#!/bin/bash

# Ensure at least one Security List ID was provided
if [ $# -lt 1 ]; then
  echo "Error: Missing Security List IDs."
  echo "Usage: $0 <LIST_ID_1> [LIST_ID_2] [LIST_ID_3] ..."
  exit 1
fi

SECURITY_LISTS=("$@")
CLOUD_URL="https://www.gstatic.com/ipranges/cloud.json"
MAX_RULES=200

# Define temporary files for safe staging
TEMP_GOOGLE_RAW=$(mktemp)
TEMP_EXISTING=$(mktemp)
TEMP_SLICED=$(mktemp)

# Clean up temporary files on exit
trap 'rm -f "$TEMP_GOOGLE_RAW" "$TEMP_EXISTING" "$TEMP_SLICED"' EXIT

echo "1. Fetching and filtering Google Cloud ranges (US & IPv4 Only)..."
curl -s "$CLOUD_URL" | jq '.prefixes | map(select(.scope | startswith("us-")) | .ipv4Prefix | select(. != null)) | unique | .[]' > "$TEMP_GOOGLE_RAW"

TOTAL_IPS=$(wc -l < "$TEMP_GOOGLE_RAW" | tr -d ' ')
echo "→ Found $TOTAL_IPS unique US IPv4 ranges to distribute."

CURRENT_LINE=1

# Loop through each provided security list
for i in "${!SECURITY_LISTS[@]}"; do
  LIST_ID="${SECURITY_LISTS[$i]}"

  # Check if we have processed all IPs
  if [ "$CURRENT_LINE" -gt "$TOTAL_IPS" ]; then
    echo "--------------------------------------------------------"
    echo "Info: All IP ranges successfully distributed. Done!"
    break
  fi

  echo "--------------------------------------------------------"
  echo "Processing Security List $((i+1))/${#SECURITY_LISTS[@]}: $LIST_ID"

  # 2. Fetch existing rules for this specific list
  echo "→ Fetching current rules..."
  RAW_EXISTING=$(oci network security-list get --security-list-id "$LIST_ID" 2>/dev/null | jq '.data["ingress-security-rules"]')

  if [ $? -ne 0 ] || [ "$RAW_EXISTING" == "null" ]; then
    echo "Warning: Failed to fetch rules for list $LIST_ID. Skipping."
    continue
  fi

  # 3. Normalize current rules
  echo "→ Normalizing existing keys..."
  echo "$RAW_EXISTING" | jq '[
    .[] | {
      description: (.description // null),
      icmpOptions: (."icmp-options" // null),
      isStateless: (."is-stateless" // false),
      protocol: .protocol,
      source: .source,
      sourceType: (."source-type" // "CIDR_BLOCK"),
      tcpOptions: (."tcp-options" // null),
      udpOptions: (."udp-options" // null)
    }
  ]' > "$TEMP_EXISTING"

  EXISTING_COUNT=$(jq '. | length' "$TEMP_EXISTING")
  echo "→ This list currently has $EXISTING_COUNT rules."

  # 4. Calculate available capacity for this specific list
  ALLOWED_NEW_RULES=$(( MAX_RULES - EXISTING_COUNT ))

  if [ "$ALLOWED_NEW_RULES" -le 0 ]; then
    echo "Warning: This list is already at or above the $MAX_RULES rule limit. Skipping to next list."
    continue
  fi

  echo "→ Room available for $ALLOWED_NEW_RULES new rules before hitting the $MAX_RULES max limit."

  # 5. Extract only the number of IPs that will fit in this specific list
  END_LINE=$(( CURRENT_LINE + ALLOWED_NEW_RULES - 1 ))
  sed -n "${CURRENT_LINE},${END_LINE}p" "$TEMP_GOOGLE_RAW" > "$TEMP_SLICED"

  CHUNK_COUNT=$(wc -l < "$TEMP_SLICED" | tr -d ' ')

  if [ "$CHUNK_COUNT" -eq 0 ]; then
     echo "Info: No more rules left to map."
     break
  fi

  echo "→ Slicing $CHUNK_COUNT IP blocks (Lines $CURRENT_LINE to $((CURRENT_LINE + CHUNK_COUNT - 1)))."

  # 6. Build the OCI formatted rule objects for this slice
  echo "→ Packaging new rules for TCP 443..."
  jq -s '[
    .[] | {
      "description": "Automated Google Cloud US IPv4 (HTTPS)",
      "icmpOptions": null,
      "isStateless": false,
      "protocol": "6",
      "source": .,
      "sourceType": "CIDR_BLOCK",
      "tcpOptions": {
        "destinationPortRange": {
          "max": 443,
          "min": 443
        },
        "sourcePortRange": null
      },
      "udpOptions": null
    }
  ]' "$TEMP_SLICED" > "${TEMP_SLICED}.final"

  # 7. Merge, minify and apply updates
  echo "→ Merging and minifying rules..."
  jq -s -c '.[0] + .[1]' "$TEMP_EXISTING" "${TEMP_SLICED}.final" > ingress_updated.json

  FINAL_RULE_COUNT=$(jq '. | length' ingress_updated.json)
  echo "→ Total rules being applied to this list: $FINAL_RULE_COUNT (Max: $MAX_RULES)"

  oci network security-list update \
    --security-list-id "$LIST_ID" \
    --ingress-security-rules "file://./ingress_updated.json" \
    --force

  rm -f "${TEMP_SLICED}.final" ingress_updated.json

  # Advance our position counter by the number of IPs we successfully consumed
  CURRENT_LINE=$(( CURRENT_LINE + CHUNK_COUNT ))
done

echo "--------------------------------------------------------"
if [ "$CURRENT_LINE" -le "$TOTAL_IPS" ]; then
  REMAINING=$(( TOTAL_IPS - CURRENT_LINE + 1 ))
  echo "Warning: Ran out of security lists! $REMAINING IP ranges could not be mapped. Please provide more list IDs."
else
  echo "Success: All IP ranges mapped without exceeding individual list limits!"
fi

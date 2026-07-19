# Oracle Cloud Infrastructure

- Project uses the free tier of OCI. Allows for 2 core / 12 GB ARM based servers.
- Leverage the firewall to whitelist specific source IPs.
- Following commands are meant to simplify the act of adding a new IP address.

## OCI CLI

```bash
bash -c "$(curl -L https://raw.githubusercontent.com/oracle/oci-cli/master/scripts/install/install.sh)"
```

Log In:

```bash
oci -i
oci iam region list --config-file /home/jpai/.oci/config --profile DEFAULT
```

## Update Security List

List out Security List information.

```bash
oci network security-list get --security-list-id ${SECURITY_LIST_OCID}
```

Update Ingress Rules

```bash
oci network security-list update --security-list-id ${SECURITY_LIST_OCID} --ingress-security-rules "${INGRESS_RULES_JSON_ARRAY}"
```

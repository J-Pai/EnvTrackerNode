# Generating Self-Signed SSL Keys and Certificates

Source `gen_certs.sh` to generate the Root CA. After that, invoke the
`generate_key_cert` bash function in the same shell.

```bash
source gen_certs.sh
generate_key_cert server $DOMAIN_NAME
generate_key_cert client $DOMAIN_NAME
```

Leave `$DOMAIN_NAME` blank if you want to just target localhost.

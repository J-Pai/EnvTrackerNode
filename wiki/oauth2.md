# OAuth2 Notes

- Must be TLS / HTTPS
- https://sslip.io/
  - Can be used to bypass GCP OAuth2 Authorized Redirect URIs limitation on
    no IP addresses.

## TLS Extfile

Signing a CA cert with nip.io or sslip.io.

```shell
-extfile <(printf "[SAN]\nsubjectAltName = DNS:111.111.111.111.nip.io, IP:111.111.111.111")
```

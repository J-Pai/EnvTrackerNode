# Control Node
Contains a Rust based RESET API server for handling interactions between the
Console node and the Core node.

Will also contain support for Wake-On-Lan and LUKS Unlock.

## Generating Self-Signed Certificates

Commands pulled from: https://deliciousbrains.com/ssl-certificate-authority-for-local-https-development/.

```shell
mkdir -p certificates
cd certificates

# Create control-node.ext with the following contents:

authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost

# Generate private key:
openssl genrsa -des3 -out myCA.key 2048
# Generate CA Certificate:
openssl req -x509 -new -nodes -key myCA.key -sha256 -days 1825 -out myCA.pem

# Create certificate signing request.
openssl genrsa -out control-node.key 2048
openssl req -new -key control-node.key -out control-node.csr

# Sign certificate with generated CA certificate.
openssl x509 -req -in control-node.csr -CA myCA.pem -CAkey myCA.key -CAcreateserial -out control-node.crt -days 365 -sha256 -extfile control-node.ext
```

Cert is `control-node.crt` and Key is `control-node.key`.

myCA.pem is what should be installed on remote clients.

Launching node with custom CA:

```shell
env NODE_EXTRA_CA_CERTS=certificates/myCA.pem npm run dev
```

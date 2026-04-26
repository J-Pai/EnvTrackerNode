# [TLS](https://learn.microsoft.com/en-us/azure/application-gateway/self-signed-certificates)

## Root CA Certificate

```shell
sudo dnf install openssl-devel
openssl ecparam -out ca.key -name prime256v1 -genkey
openssl req -new -sha256 -key ca.key -out ca.csr
openssl x509 -req -sha256 -days 365 -in ca.csr -signkey ca.key -out ca.crt
```

## Server Certificate

```shell
openssl ecparam -out server.key -name prime256v1 -genkey
openssl req -new -sha256 -key server.key -out server.csr
openssl x509 -req -sha256 -CA ca.crt -CAkey ca.key -CAcreateserial -days 365 \
    -in server.csr -out server.crt \
    -extensions SAN \
    -extfile <(printf "[SAN]\nsubjectAltName = DNS:example.com, email:me@gmail.com, IP:1.1.1.1")
```

## Check Certificate

```shell
openssl x509 -in server.crt -text -noout
```

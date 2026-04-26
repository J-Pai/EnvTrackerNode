# Environment Tracker Node

### Environment Setup

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel
cargo install trunk --locked
cargo install cross --git https://github.com/cross-rs/cross
```

### Kasa Core

- Commands: https://docs.rs/kasa-core/0.6.0/kasa_core/commands/index.html#constants.

### Launch

```shell
./launch.sh
cross build --target armv7-unknown-linux-musleabihf
```

### TLS ([ref](https://learn.microsoft.com/en-us/azure/application-gateway/self-signed-certificates))

- Root CA Certificate

```shell
sudo dnf install openssl-devel
openssl ecparam -out ca.key -name prime256v1 -genkey
openssl req -new -sha256 -key ca.key -out ca.csr
openssl x509 -req -sha256 -days 365 -in ca.csr -signkey ca.key -out ca.crt
```

- Server Certificate

```shell
openssl ecparam -out server.key -name prime256v1 -genkey
openssl req -new -sha256 -key server.key -out server.csr
openssl x509 -req -sha256 -CA ca.crt -CAkey ca.key -CAcreateserial -days 365 \
    -in server.csr -out server.crt \
    -extensions SAN \
    -extfile <(printf "[SAN]\nsubjectAltName = DNS:example.com, email:me@gmail.com, IP:1.1.1.1")
```

- Check Certificate

```shell
openssl x509 -in server.crt -text -noout
```

#### PiKVM

Certificates for kvmd-nginx is located at:

```shell
$ ls -al /etc/kvmd/nginx/ssl
total 16
drwxr-xr-x 2 root       root       4096 Aug 18  2024 .
drwxr-xr-x 3 root       root       4096 Apr 18 07:53 ..
-r-------- 1 kvmd-nginx kvmd-nginx  867 Apr 26 01:09 server.crt
-r-------- 1 kvmd-nginx kvmd-nginx  302 Apr 26 01:09 server.key
```

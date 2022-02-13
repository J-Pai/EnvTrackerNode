# Core Node
Contains the central gRPC server for interacting with the Tracker and Sense Hat
nodes.

Core Node is written in C++

## Build Steps
Please make sure to setup your system based on the root directory's
[README's Installation and Setup steps](../../README.md#installation-and-setup).

```bash
cd core_node
mkdir -p cmake/build
cd cmake/build
cmake ../.. [-DCMAKE_EXPORT_COMPILE_COMMANDS=1]
make

# Following is optional. clangd uses the compile_commands.json file to configure
# the LSP on a per-project basis.
ln -sfn `pwd`/compile_commands.json ../..
```

**NOTE**: Use `-DCMAKE_EXPORT_COMPILE_COMMANDS=1` if you want clangd LSP support.
This only works if you are developing on an x86_64 machine as clangd does not
release an arm64 version.

## Execution
From inside of cmake/build:

### Terminal 1

```bash
./core_server
```

### Terminal 2

```bash
./basic_client
```

## Enabling SSL/TLS
For both the server and client, specify the following environment variables:

```bash
export SSL_KEY=/path/to/private/ssl/key
export SSL_CERT=/path/to/private/ssl/certificate
export SSL_ROOT_CERT=/path/to/private/ssl/ca/root/certificate
```

You can generate a set of self-signed SSL certificates and private keys using
the helper script found [here](../../scripts/ssl/)

## Request OAuth2
Specify the path to the client secret JSON with the following environment
variable:

```bash
export CLIENT_SECRET_JSON=/path/to/client/secret.json
```

## Validate OAuth2 Endpoints

Get token information:

```
https://www.googleapis.com/oauth2/v3/tokeninfo?access_token=ACCESS_TOKEN
```

Get user profile information:

```
https://www.googleapis.com/oauth2/v3/userinfo?access_token=ACCESS_TOKEN
```

## Revoke OAuth2 Token

```
curl -d -X -POST --header "Content-type:application/x-www-form-urlencoded" \
        https://oauth2.googleapis.com/revoke?token={token}
```

## Using ngrok
ngrok.yml:

```yaml
tunnels:
  env_tracker_node:
    proto: http
    addr: "8080"
    bind_tls: "true"
```

Staring ngrok:

```bash
ngrok start --all    \
  --config=ngrok.yml \
  --log=stdout
```

Obtain currently opened endpoints:

```bash
curl -s localhost:4040/api/tunnels
```

## Packet/Port Sniffing

```bash
sudo tcpflow -i any -C -g port 50051
```

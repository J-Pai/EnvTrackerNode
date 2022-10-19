# Core Node
Contains the central gRPC server for interacting with the Tracker and Sense Hat
nodes.

Core Node is written in C++

## Build Steps

```
bazel build //core_node:all --config=arm64
```

## Execution

### Terminal 1

```bash
GLOG_logtostderr=1 ./core_server
```

### Terminal 2

```bash
GLOG_logtostderr=1 ./demo_client
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

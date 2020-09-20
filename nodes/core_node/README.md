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
cmake ../.. -DCMAKE_EXPORT_COMPILE_COMMANDS=1
ln -sfn `pwd`/compile_commands.json ../..
make
```

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

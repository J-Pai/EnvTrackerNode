# Core Node
Contains the central gRPC server for interacting with the Tracker and Sense Hat
nodes.

Core Node is written in C++ and Node.

## Build Steps
Please make sure to setup your system based on the root directory's
[README's Installation and Setup steps](../../README.md#installation-and-setup).

```
cd core_node
mkdir -p cmake/build
cd cmake/build
cmake ../..
```

## Using ngrok
ngrok.yml:

```
tunnels:
  env_tracker_node:
    proto: http
    addr: "8080"
    bind_tls: "true"
```

Staring ngrok:

```
ngrok start --all    \
  --config=ngrok.yml \
  --log=stdout
```

Obtain currently opened endpoints:

```
curl -s localhost:4040/api/tunnels
```

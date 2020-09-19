# Core Node
Contains the central gRPC server for interacting with the Tracker and Sense Hat
nodes.

Core Node is written in C++ and Node.

## Build Steps
Please make sure to setup your system based on the root directory's
[README's Installation and Setup steps](../../README.md#installation-and-setup).

```bash
cd core_node
mkdir -p cmake/build
cd cmake/build
cmake ../.. -DCMAKE_EXPORT_COMPILE_COMMANDS=1
ln -sfn `pwd`/compile_commands.json ../..
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

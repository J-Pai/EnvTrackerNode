# Core Node
Contains the central gRPC server for interacting with the Tracker and Sense Hat
nodes.

Core Node is written in C++ and Node.

# Using ngrok
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

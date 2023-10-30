# Console Node
Provides a UI for interacting with the Control Node.

## Launching with Custom CA

See [Control Node README](../control_node/README.md) for information on
generating myCA.pem.

```shell
env NODE_EXTRA_CA_CERTS=certificates/myCA.pem npm run dev
```

## Logging SSL Secrets for Wireshark PCAP Analysis

```shell
env NODE_EXTRA_CA_CERTS=certificates/myCA.pem \
    NODE_OPTIONS="--tls-keylog=certificates/sslkeys.log" \
    npm run dev
```

In Wireshark:

```
Edit > Preferences > Protocols > TLS > (Pre)-Master-Secret log filename
```

Set to the path provided by `--tls-keylog`.

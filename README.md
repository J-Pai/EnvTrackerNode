# Environment Tracker Node

## Web Node

### Environment Setup

```shell
sudo dnf install perl openssl-devel
curl https://sh.rustup.rs -sSf | sh
cargo install --locked cargo-leptos 
rustup target add wasm32-unknown-unknown
```

### Development

```shell
cd envtrackernode_web
cargo leptos watch
```

## Kasa Node

```shell
virtualenv venv
. venv/bin/activate
pip install -r requirements.txt
```

### Development

```shell
pip freeze -r requirements.txt
```


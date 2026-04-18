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
cargo run
```

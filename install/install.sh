#!/bin/bash

# Builds the Installer Packages
#
# Compiles the release version of the binary and creates packages to push
# to remote servers.
#
# Currently assumes remote / server is arm64 (OCI ARM or Raspberry Pi).

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR

../launch.sh build-release

rm -rf *.toml
rm -rf *.zip
rm -rf release
mkdir -p release
pushd ..

rsync -aP ./install/install_node.sh install/release
rsync -aP ./install/config.toml install/release
rsync -aP ./target/aarch64-unknown-linux-musl/release/node install/release
rsync -aP --exclude ./dist/.stage ./dist install/release

popd

zip -r envtrackernode.zip release

scp -r envtrackernode.zip $1

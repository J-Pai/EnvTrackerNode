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

rm -rf release
mkdir -p release
zip -r release/envtrackernode.zip \
	./install_node.sh \
	../dist \
	../target/aarch64-unknown-linux-gnu/release/node


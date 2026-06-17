#!/bin/bash

# Installer
#
# Compiles the release version of the binary and pushes / installs / starts
# the service on the remote side.
#
# Currently assumes remote / server is arm64 (OCI ARM or Raspberry Pi).

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR

../launch.sh build-release

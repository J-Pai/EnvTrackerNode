#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

RELEASE_FLAG=""

if [[ "$1" == "release" ]]; then
	RELEASE_FLAG="--release"
fi

cd $SCRIPT_DIR
trunk build $RELEASE_FLAG
cargo run $RELEASE_FLAG $@

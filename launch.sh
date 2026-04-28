#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR

RELEASE_FLAG=""
TRUNK_ACTION="build"

if [[ "$1" == "release" ]]; then
	RELEASE_FLAG="--release"
elif [[ "$1" == "watch" ]]; then
	TRUNK_ACTION="watch"
fi

trunk $TRUNK_ACTION $RELEASE_FLAG

rc=$?

if [[ "$rc" != 0 || "${TRUNK_ACTION}" == "watch" ]]; then
	exit $rc
fi

cargo run $RELEASE_FLAG $@

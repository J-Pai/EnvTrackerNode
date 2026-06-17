#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR

pkill -INT trunk

RELEASE_FLAG=""
TRUNK_ACTION="build"
CARGO_ACTION="run"

while [ $# -gt 0 ]; do
	case $1 in
		release)
			RELEASE_FLAG="--release"
			;;
		build-release)
			RELEASE_FLAG="--release"
			CARGO_ACTION="build"
			;;
		watch)
			TRUNK_ACTION="watch"
			;;
		serve)
			TRUNK_ACTION="serve"
			;;
		--)
			# Everything after -- goes to cargo.
			shift
			break
			;;
		*)
			;;
	esac
	shift
done

echo "=== TRUNK ==="
trunk $TRUNK_ACTION $RELEASE_FLAG

rc=$?

if [[ "$rc" != 0 || "${TRUNK_ACTION}" != "build" ]]; then
	exit $rc
fi

echo "=== Cargo RUN ==="

cargo $CARGO_ACTION $RELEASE_FLAG -- $@

#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR

RELEASE_FLAG=""
TRUNK_ACTION="build"

while [ $# -gt 0 ]; do
	case $1 in
		release)
			RELEASE_FLAG="--release"
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

if [[ "$rc" != 0 || "${TRUNK_ACTION}" == "watch" ]]; then
	exit $rc
fi

echo "=== Cargo RUN ==="

cargo run $RELEASE_FLAG -- $@

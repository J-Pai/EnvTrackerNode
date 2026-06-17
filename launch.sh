#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR

pkill -INT trunk

RELEASE_FLAG=""
TRUNK_ACTION="build"
CARGO_BINARY="cargo"
CARGO_ACTION="run"

function clean_workspace() {
	trunk clean
	rm -rf target_trunk
	cargo clean
}

if [[ "$1" == "build-release" ]]; then
	cargo install cross --git https://github.com/cross-rs/cross
	cargo install --locked trunk
	CARGO_BINARY="cross"
	CARGO_ACTION="build"
	RELEASE_FLAG="--release"
	shift
	set -x
fi

while [ $# -gt 0 ] && [[ "$1" != "build-release" ]]; do
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
		clean)
			clean_workspace
			exit
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
CARGO_TARGET_DIR="target_trunk" trunk $TRUNK_ACTION $RELEASE_FLAG &
[[ "$CARGO_ACTION" == "run" ]] && {
	echo "Waiting for trunk to complete"; wait;
	rc=$?
	if [[ "$rc" != "0" || "${TRUNK_ACTION}" != "build" ]]; then
		exit $rc
	fi
}

echo "=== Cargo RUN ==="
$CARGO_BINARY $CARGO_ACTION $RELEASE_FLAG -- $@

#!/bin/bash

SCRIPTPATH="$( cd "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
COMPILED_FILEPATH="${SCRIPTPATH}/target/release/rockery"

if !(test -f COMPILED_FILEPATH); then
	echo "[+] File ${COMPILED_FILEPATH} does not exist! Compiling..."
	cargo build --release
fi

./target/release/rockery

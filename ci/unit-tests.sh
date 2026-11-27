#!/bin/bash
# Build and collect unit test binaries for another runner:
#
#     ci/unit-tests.sh DIRECTORY [CARGO TEST'S ARGUMENTS...]
#
# Runs 'cargo test --no-run' with any trailing arguments and copies each
# test binary into DIRECTORY/PACKAGE-DIR/NAME so ci/run-tests.sh can match
# Cargo's working directory for each package.
set -euo pipefail
[ $# -ge 1 ] || { sed -n '2,8p' "$0" >&2; exit 2; }
dest=$1
shift
cargo test --no-run --message-format=json "$@" | python3 -c '
import json, os, shutil, sys

dest, root = sys.argv[1], os.getcwd()
for line in sys.stdin:
	try:
		message = json.loads(line)
	except ValueError:
		continue
	if message.get("reason") != "compiler-artifact" or not message["profile"]["test"] or not message.get("executable"):
		continue
	package = os.path.relpath(os.path.dirname(message["manifest_path"]), root)
	name = message["target"]["name"] + os.path.splitext(message["executable"])[1]
	os.makedirs(os.path.join(dest, package), exist_ok=True)
	shutil.copy(message["executable"], os.path.join(dest, package, name))
	print("unit tests:", os.path.join(package, name))
' "$dest"

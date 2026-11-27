#!/bin/sh
# Restore timestamps on unchanged files so Cargo can reuse cached builds:
#
#     ci/source-times.sh LIST
#
# Cargo rebuilds a crate whenever its source files or directories are newer
# than its build artifacts, and a fresh checkout resets every timestamp. LIST
# (cached alongside Cargo's build directory) records the contents and
# timestamp of each file and directory in HEAD from the previous run.
#
# Run in a checkout of HEAD before invoking Cargo. Any entry whose contents
# still match LIST gets its recorded timestamp back, changed entries keep
# their new timestamps, and LIST is rewritten for the current checkout (or
# created if it does not exist).
set -eu
[ $# -eq 1 ] || { sed -n '2,14p' "$0" >&2; exit 2; }
list=$1
tab=$(printf '\t')
mkdir -p "$(dirname "$list")"
git ls-tree -r -t HEAD >"$list.files"
if [ -f "$list" ]; then
	awk -F '\t' 'NR == FNR { time[$2 FS $3] = $1; next } $0 in time { print time[$0] FS $2 }' "$list" "$list.files" |
		while IFS=$tab read -r time file; do
			touch -c -d "@$time" -- "$file"
		done
fi
git ls-tree -r -t -z --name-only HEAD | xargs -0 stat -c "%Y$tab%n" >"$list.times"
awk -F '\t' 'NR == FNR { time[$2] = $1; next } { print time[$2] FS $0 }' "$list.times" "$list.files" >"$list"
rm "$list.files" "$list.times"

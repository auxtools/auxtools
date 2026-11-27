#!/bin/bash
# Run BYOND integration tests using prebuilt binaries:
#
#     ci/run-tests.sh PROGRAMS BYOND [TEST...]
#
# PROGRAMS contains the test binaries, the launcher, and the debug library
# (built with --features debug_server/test-commands so the suites can inspect
# worlds through test console commands). BYOND is the installation directory
# containing bin/. Run this script from the repository root.
#
# Set WINE=wine to run Windows binaries on Linux. Wine uses WINEPREFIX if set,
# or defaults to a dedicated prefix in target/wine-prefix.
#
# Suites run in parallel, each in target/run-tests/TEST with its own copy of
# the test worlds, printing output when finished. Any suite still running
# after TIMEOUT seconds [default: 900] is terminated and marked failed. Set
# SERIAL=1 to run suites one at a time from the repository root with no
# timeout.
#
# With no TEST arguments, runs all integration suites and, if the directory
# PROGRAMS/unit exists, also runs every unit test binary under it one at a
# time from the matching repository directory. Exits with 1 if any test fails.
set -u
[ $# -ge 2 ] || { sed -n '2,22p' "$0" >&2; exit 2; }
programs=$(cd "$1" && pwd) || exit 2
byond=$(cd "$2" && pwd) || exit 2
shift 2
tests=${*:-test_runner debug_test hot_reload_test launcher_test dap_test}

exe=
case "$(uname -s)" in
	MINGW* | MSYS* | CYGWIN*)
		exe=.exe
		BYOND_PATH=$(cygpath -w "$byond")
		;;
	*)
		if [ -n "${WINE:-}" ]; then
			exe=.exe
			BYOND_PATH="Z:$(printf '%s' "$byond" | tr / '\\')"
			export WINEDEBUG="${WINEDEBUG:--all}"
		else
			BYOND_PATH=$byond
			export LD_LIBRARY_PATH="$byond/bin${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
		fi
		;;
esac
export BYOND_PATH

work=$PWD/target/run-tests

panicked() {
	grep -q "^thread '[^']*'.* panicked at" "$1"
}

failed=
report() {
	if [ "$2" -eq 0 ]; then
		echo "passed: $1 ($3 s)"
	else
		echo "FAILED: $1 (exit status $2, $3 s)"
		failed="$failed $1"
	fi
}

run() {
	name=$1
	shift
	echo "::group::$name"
	start=$(date +%s)
	mkdir -p "$work"
	${WINE:+"$WINE"} "$@" 2>&1 | tee "$work/output.log"
	status=${PIPESTATUS[0]}
	if [ "$status" -eq 0 ] && panicked "$work/output.log"; then
		echo "$name panicked, but its exit status was 0"
		status=1
	fi
	echo "::endgroup::"
	report "$name" $status $(($(date +%s) - start))
}

if [ -n "${WINE:-}" ]; then
	own_prefix=
	if [ -z "${WINEPREFIX:-}" ]; then
		mkdir -p target
		export WINEPREFIX=$PWD/target/wine-prefix
		own_prefix=1
	fi
	if ! "$WINE" reg query 'HKCU\Software\Wine\Drivers' /v Graphics 2>/dev/null | grep -q null; then
		if [ -n "$own_prefix" ]; then
			"$WINE" reg add 'HKCU\Software\Wine\Drivers' /v Graphics /d null /f >/dev/null
			"${WINESERVER:-wineserver}" -w
		else
			echo "warning: the Wine prefix $WINEPREFIX doesn't use Wine's null graphics driver, so without a display a world may exit at its start. To set it: $WINE reg add 'HKCU\Software\Wine\Drivers' /v Graphics /d null /f" >&2
		fi
	fi
fi

if [ -n "${SERIAL:-}" ]; then
	for test in $tests; do
		run "$test" "$programs/$test$exe"
	done
else
	rm -rf "$work"
	declare -A pids
	trap 'kill "${pids[@]}" 2>/dev/null; exit 130' INT TERM
	start=$(date +%s)
	for test in $tests; do
		mkdir -p "$work/$test/tests"
		cp -r tests/*_host "$work/$test/tests/"
		(cd "$work/$test" && exec ${WINE:+"$WINE"} "$programs/$test$exe") >"$work/$test.log" 2>&1 &
		pids[$test]=$!
	done
	left=$tests
	while [ -n "$left" ]; do
		sleep 1
		seconds=$(($(date +%s) - start))
		running=
		for test in $left; do
			if kill -0 "${pids[$test]}" 2>/dev/null; then
				if [ $seconds -lt "${TIMEOUT:-900}" ]; then
					running="$running $test"
					continue
				fi
				kill "${pids[$test]}"
				echo "$test was still running after $seconds s" >>"$work/$test.log"
			fi
			wait "${pids[$test]}"
			status=$?
			unset "pids[$test]"
			if [ $status -eq 0 ] && panicked "$work/$test.log"; then
				echo "$test panicked, but its exit status was 0" >>"$work/$test.log"
				status=1
			fi
			echo "::group::$test"
			cat "$work/$test.log"
			echo "::endgroup::"
			report "$test" $status $seconds
		done
		left=$running
	done
	trap - INT TERM
fi

if [ -d "$programs/unit" ] && [ $# -eq 0 ]; then
	root=$PWD
	for test in $(cd "$programs/unit" && find . -type f | sort); do
		cd "$(dirname "$test")" || exit 2
		run "unit tests: ${test#./}" "$programs/unit/$test"
		cd "$root" || exit 2
	done
fi

if [ -n "$failed" ]; then
	echo "failed:$failed"
	exit 1
fi
echo "all passed"

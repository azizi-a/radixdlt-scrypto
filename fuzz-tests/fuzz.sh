#!/bin/bash

#set -x

# defaults
DFLT_FUZZER=simple
DFLT_CMD=run
DFLT_RUN_CMD_ARG=10
DFLT_TARGET=transaction

# available fuzzers: libfuzzer, afl, simple
fuzzer=${1:-$DFLT_FUZZER}
# available commands:
# - run, build, init
cmd=${2:-$DFLT_CMD}


run_args=""
run_cmd_arg=
# run_cmd_arg might be in seconds or 'inf', when fuzzing infinitely
if [ $cmd = "run" ] ; then
    run_cmd_arg=${3:-$DFLT_RUN_CMD_ARG}
fi
#variant="release"
target=$DFLT_TARGET

function usage() {
    echo "$0 [FUZZER] [COMMANDS] [COMMAND-ARGS]"
    echo "Available fuzzers:"
    echo "  libfuzzer - 'cargo fuzz' wrapper"
    echo "  afl       - 'cargo afl' wrapper"
    echo "  simple    - simple fuzzer (default)"
    echo "Available commands:"
    echo "  init      - Take sample input ('./fuzz_input/<target>') for given test,"
    echo "              minimize the test corpus and put the result into 'corpus/<target>',"
    echo "              which is used by 'libfuzzer' as initial input."
    echo "              Applicable only for 'libfuzzer'."
    echo "  build     - Build fuzz test for given fuzzer."
    echo "              Binaries are built in 'release' format."
    echo "  run       - Run fuzz test for given fuzzer (default command)"
    echo "              Available command options:"
    echo "              <run_cmd_arg>  If points to the existing file,"
    echo "                             then run once with the input from file (useful for crash reproduction),"
    echo "                             else run for given number of seconds or infinitely if 'inf' specified."
    echo "                             Set to '$DFLT_RUN_CMD_ARG' by default."
    echo "              Currently in case of:"
    echo "              - libfuzzer - fuzzing stops when crash/oom/timeout occurs"
    echo "              - afl       - fuzzing continues when crash/oom/timeout occurs (reproduction data is saved to file)"
    echo "Examples:"i
    echo "  - build AFL fuzz tests"
    echo "    $0 afl build"
    echo "  - run AFL fuzz tests for 1h"
    echo "    $0 afl run 3600"
    echo "  - reproduce some crash discovered by 'libfuzzer'"
    echo "    $0 libfuzzer run ./artifacts/transaction/crash-ec25d9d2a8c3d401d84da65fd2321cda289d"
    echo "  - reproduce some crash discovered by 'libfuzzer' using 'simple-fuzzer'"
    echo "    $0 simple run ./artifacts/transaction/crash-ec25d9d2a8c3d401d84da65fd2321cda289d"
}

if [ $fuzzer = "libfuzzer" ] ; then
    if [ "$cmd" = "run" ] ; then
        if [ "$run_cmd_arg" != "" -a -s $run_cmd_arg ] ; then
            run_args+="$run_cmd_arg "
        else
            run_args="-- "
            run_args+="-create_missing_dirs=1 "
            if [ "$run_cmd_arg" != "inf" ] ; then
                run_args+="-max_total_time=${run_cmd_arg} "
            fi
        fi

    elif [ "$cmd" = "init" ] ; then
        # initial setup:
        # - minimize the corpus:
        #    https://llvm.org/docs/LibFuzzer.html#id25
        #
        #   cargo +nightly fuzz $target  --fuzz-dir radix-engine-fuzz \
        #      --no-cfg-fuzzing --target-dir target-libfuzzer $target -- \
        #      -merge=1 corpus/$target <INTERESTING_INPUTS_DIR/FULL_CORPUS_DIR>
        #
        cmd=run
        run_args="-- -merge=1 corpus/${target} fuzz_input/${target} "
    fi
    # Unset cfg=fuzzing by --no-cfg-fuzzing.
    # "secp256k1" uses some stubs instead of true cryptography if "fuzzing" is set.
    # see: https://github.com/rust-bitcoin/rust-secp256k1/#fuzzing
    set -x
    cargo +nightly fuzz $cmd \
        --release \
        --no-default-features --features std,libfuzzer-sys \
        --fuzz-dir . \
        --no-cfg-fuzzing \
        --target-dir target-libfuzzer \
        $target \
        $run_args

elif [ $fuzzer = "afl" ] ; then
    if [ $cmd = "build" ] ; then
        cargo afl build --release \
            --no-default-features --features std,afl \
            --target-dir target-afl
    else
        if [ "$run_cmd_arg" != "inf" ] ; then
            run_args="-V ${run_cmd_arg}"
        fi
        mkdir -p afl/${target}/out
        set -x
        cargo afl fuzz -i fuzz_input/${target} -o afl/${target}/out $run_args target-afl/release/${target}
    fi

elif [ $fuzzer = "simple" ] ; then
    if [ "$cmd" = "run" ] ; then
        if [ "$run_cmd_arg" != "" -a -s $run_cmd_arg ] ; then
            export RUST_BACKTRACE=full
            run_args+="$run_cmd_arg "
        elif [ "$run_cmd_arg" != "inf" ] ; then
            run_args="-- --duration ${run_cmd_arg}"
        fi
    fi
    set -x
    cargo $cmd --release \
        --no-default-features --features std,simple-fuzzer \
        --bin $target \
        $run_args
else
    if [ $fuzzer != "help" -a $fuzzer != "h" ] ; then
        echo "invalid fuzzer '$fuzzer' specified"
    fi
    usage
fi

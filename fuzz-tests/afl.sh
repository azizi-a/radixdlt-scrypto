#!/bin/bash

#set -x
set -e
set -u

DFLT_CPUS=1
DFLT_INTERVAL=60
# At the moment this is the only supported test
DFLT_TARGET=transaction
DFLT_AFL_TIMEOUT=1000

function usage() {
    echo "$0 [COMMAND] [COMMAND-ARGS]"
    echo "Commands:"
    echo "    run <duration> <instances> [timeout]"
    echo "            Run given number of AFL instances (default: $DFLT_CPUS) in screen sessions"
    echo "            for a given number of seconds."
    echo "            For 'instances' one can specify"
    echo "              all      - to run as many instances as CPU cores available"
    echo "              <number> - to run <number> of instances"
    echo "            'timeout' is an AFL timeout in ms"
    echo "    watch <interval>"
    echo "            Monitor AFL instances until they are finished."
    echo "            One can specify interval (default: $DFLT_INTERVAL) to output the status"
}

function error() {
    local msg=$1
    echo "error - $msg"
    usage
    exit 1
}

function get_cpus() {
    local uname="$(uname -s)"
    if [ $uname = "Linux" ] ; then
        cat /proc/cpuinfo  | grep processor | wc -l
    elif [ $uname = "Darwin" ] ; then
        sysctl -n hw.ncpu
    else
        echo "OS $uname not supported"
        exit 1
    fi
}

function humanize_seconds()
{
   local t=$1
   local d=$((t / 60 / 60 / 24))
   local h=$((t / 60 / 60 % 24))
   local m=$((t / 60 % 60))
   local s=$((t % 60))

   if [ $d -ne 0 ] ; then
      printf '%d days %02d hours %02d minutes %02d seconds' $d $h $m $s
   else
      printf '%02d hours %02d minutes %02d seconds' $h $m $s
   fi
}

target=$DFLT_TARGET
cmd=${1:-watch}
shift

if [ $cmd = "run" ] ; then
    if [ $# -lt 1 ] ; then
        error "duration parameter is missing"
    fi
    duration=${1}
    if ! [[ $duration =~ ^[0-9]+$ ]] ; then
        error "given duration '$duration' is not a number"
    fi
    cpus=${2:-1}
    timeout=${3:-$DFLT_AFL_TIMEOUT}
    if [ $cpus = "all" ] ; then
        cpus=$(get_cpus)
        echo "CPU cores available: $cpus"
    fi
    if ! [[ $cpus =~ ^[0-9]+$ ]] ; then
        error "given instances '$cpus' is not a number or 'all'"
    fi
    if ! [[ $timeout =~ ^[0-9]+$ ]] ; then
        error "given timeout '$timeout' is not a number"
    fi
    echo "Running $cpus AFL instances for $duration seconds"
    mkdir -p afl

    # Remove dead screen sessions.
    # Such sessions might remain if the previous run was cancelled.
    screen -wipe || true

    for (( i=0; i<$cpus; i++ )) ; do
        if [ $i -eq 0 ] ; then
            name=${target}_main_$i
            # main fuzzer
            fuzzer="-M $name"
        else
            name=${target}_secondary_$i
            # secondary fuzzer
            fuzzer="-S $name"
        fi
        # TODO: use different fuzzing variants per instance
        screen -dmS afl_$name \
            bash -c "{ ./fuzz.sh afl run -V $duration $fuzzer -T $name -t $timeout >afl/$name.log 2>afl/$name.err ; echo \$? > afl/$name.status; }"
    done
    echo "Started below screen sessions with AFL instances"
    # adding 'true', because screen returns always error, when listing sessions.
    screen -ls afl_ || true

    echo "started=$(date +%s)" > afl/${target}_info
    echo "duration=$duration" >> afl/${target}_info

elif [ $cmd = "watch" ] ; then
    interval=${1:-$DFLT_INTERVAL}
    duration=
    started=$(date +%s)
    # afl/info should include most accurate info on duration and start time
    if [ -f afl/${target}_info ] ; then
        source afl/${target}_info
    fi
    # if no start time given, then get current time (it's better than nothing)
    if [ $started = "none" ] ; then
        stared=$(date +%s)
    fi
    while ! screen -ls afl_${target} | grep "No Sockets found" ; do
        sleep $interval
        # afl folder structure created with some delay after fuzz startup
        if [ -d afl/$target ] ; then
            cargo afl whatsup -d afl/$target
        fi
        now=$(date +%s)
        run_time=$(( now - started ))
        echo "Fuzzing duration : $(humanize_seconds $run_time)"
        if [ $duration != "" ] ; then
            time_left=$(( duration - run_time ))
            if [ $time_left -lt 0 ] ; then
                time_left=0
            fi
            echo "Fuzzing ends in  : $(humanize_seconds $time_left)"
        fi
    done
    echo "AFL instances status (0 means 'ok'):"
    find afl -name "${target}_*.status" | xargs grep -H -v "*"
else
    error "Command '$cmd' not supported"
fi


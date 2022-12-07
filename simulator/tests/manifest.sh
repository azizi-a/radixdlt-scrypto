#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

resim="cargo run --bin resim $@ --"

$resim reset

export account=`$resim new-account | awk '/Account component address:/ {print $NF}'`
export owner_badge=`$resim new-simple-badge --name 'OwnerBadge' | awk '/NFAddress:/ {print $NF}'`
export package=`$resim publish ../examples/hello-world --owner-badge $owner_badge | awk '/Package:/ {print $NF}'`

export xrd=resource_sim1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95sqjjpwr

output=`$resim run ./tests/m1.rtm | awk '/Component:|Resource:/ {print $NF}'`
export component=`echo $output | cut -d " " -f1`
export resource=`echo $output | cut -d " " -f2`

$resim run ./tests/m2.rtm

$resim show-ledger
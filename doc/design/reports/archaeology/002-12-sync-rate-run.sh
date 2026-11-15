#!/usr/bin/env bash
# 002-12-sync-rate-run.sh <sync-hz> <duration-s> <out-dir> <group-port> [poc-dir]
#
# One run of the four-module graph of `hadropoc loopback`, each module as its
# own process with its own --metrics file, so every metrics file has a single
# writer and the POC needs no code change. The module wiring, the --iface and
# the simulated crystals (+250 ms and +35 ppm per module index on top of
# 3 ms / 20 ppm) are those `loopback` builds in-process (src/main.rs, Loopback
# arm). Modules start in loopback's order: synth1, synth2, mixer, sender.
# <poc-dir> is a pristine, built extraction of the poc-untouched tag:
#   mkdir -p /tmp/opencode/002-12-pristine && cd /tmp/opencode/002-12-pristine &&
#   git -C /home/dorian/workspace/hadropoc archive poc-untouched | tar -x -C . &&
#   cargo build --release
# Never point <poc-dir> at the frozen tree itself. Outputs go under /tmp/opencode.
set -eu
HZ=$1; DUR=$2; OUT=$3; PORT=$4; POC=${5:-/tmp/opencode/002-12-pristine}
BIN=$POC/target/release/hadropoc
G="--group 239.72.68.44:$PORT --iface 127.0.0.1 --no-tui"
mkdir -p "$OUT"
"$BIN" synth  --name synth1 $G --sync-hz "$HZ" --duration "$DUR" --sim-offset-ms 253 --sim-drift-ppm 55  --connect audio=mixer.in1 --metrics "$OUT/synth1.csv" > "$OUT/synth1.log" 2>&1 &
"$BIN" synth  --name synth2 $G --sync-hz "$HZ" --duration "$DUR" --sim-offset-ms 503 --sim-drift-ppm 90  --connect audio=mixer.in2 --metrics "$OUT/synth2.csv" > "$OUT/synth2.log" 2>&1 &
"$BIN" mixer  --name mixer  $G --sync-hz "$HZ" --duration "$DUR" --sim-offset-ms 753 --sim-drift-ppm 125 --connect mix=sender.return --metrics "$OUT/mixer.csv" > "$OUT/mixer.log" 2>&1 &
"$BIN" sender --name sender $G --sync-hz "$HZ" --duration "$DUR" --sim-offset-ms 3   --sim-drift-ppm 20  --connect events=synth1.events --connect events=synth2.events --connect audio=mixer.feed --metrics "$OUT/sender.csv" > "$OUT/sender.log" 2>&1 &
wait

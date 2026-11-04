#!/bin/bash
# Programmatic audio-health check for the Fireface UCX hwdep driver.
#
# Polls the driver's transport counters (empty-URB queue, short packets,
# status errors, recoveries) and PipeWire's ALSA xrun counter once a second.
# Prints a per-second line and a summary; exits 1 if any error counter moved.
#
#   scripts/audio-health.sh [seconds]
#
# Run a load in parallel, e.g.:
#   scripts/audio-health.sh 60 &
#   touch ui/totalcontrol/totalcontrol-gui/src/app.rs
#   taskset -c 0-3,6-8,10-15 nice -n 19 cargo build --profile profiling -p totalcontrol-gui
set -u
secs=${1:-30}
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
bin=${TMPDIR:-/tmp}/totalcontrol-ffuc-counters
gcc -O2 -I"$root/kernel" -o "$bin" "$root/tools/ffuc-counters.c" || exit 1
node=/dev/snd/hwC2D0
get() { "$bin" "$node" 2>/dev/null | sed -n "s/.*[[:space:]]$1=\([0-9][0-9]*\).*/\1/p"; }
xrun() { pw-top -b -n 2 2>/dev/null | grep -E '^R .*Fireface.*pro-output' | tail -1 | awk '{print $9}'; }
state() { awk '/^state:/ {print $2}' /proc/asound/card2/pcm0p/sub0/status 2>/dev/null; }
q() { grep -E 'period_size|buffer_size' /proc/asound/card2/pcm0p/sub0/hw_params 2>/dev/null | tr '\n' ' '; }

fields="out_gaps out_shorts out_status_errors urb_errors recoveries in_gaps in_shorts in_status_errors"
declare -A prev
for f in $fields; do prev[$f]=$(get "$f"); done
prev_x=$(xrun)
fail=0
echo "time      xrun  deltas (out_gaps/out_short/out_stat/urb/recov/in_gaps/in_short/in_stat)  quantum state"
for i in $(seq 1 "$secs"); do
    sleep 1
    x=$(xrun); dx=$((x - prev_x)); prev_x=$x
    line=""
    for f in $fields; do
        v=$(get "$f"); line+=" $((v - ${prev[$f]}))"; prev[$f]=$v
    done
    set -- $line
    for v in "$@"; do [ "$v" -ne 0 ] && fail=1; done
    [ "$dx" -ne 0 ] && fail=1
    printf '%s %5d %s  %-24s %s\n' \
        "$(date +%H:%M:%S)" "$dx" "$line" "$(q)" "$(state)"
done
echo "result: $([ "$fail" -eq 0 ] && echo CLEAN || echo ERRORS)"
exit "$fail"

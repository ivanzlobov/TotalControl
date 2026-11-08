#!/usr/bin/env bash
# Create the TotalControl 5.1 and 7.1 PipeWire sinks.
#
# The Fireface/MR18 Pro Audio nodes expose 18 playback channels as AUX0..17.
# Desktop players send a surround stream, which WirePlumber cannot map onto
# those AUX ports, so only FL/FR reach the device.  These loopback sinks give
# the players real surround layouts and forward the channels to the first
# surround SW playback channels.
set -euo pipefail

RUN_DIR="${XDG_RUNTIME_DIR:-/tmp}/totalcontrol-surround"
mkdir -p "$RUN_DIR"

usage() {
    cat <<'EOF'
Usage: totalcontrol-surround.sh COMMAND [target-sink]

Commands:
  start        create both the 5.1 and 7.1 sinks
  stop         remove both sinks
  restart      remove and recreate both
  start51      create only the 5.1 sink
  stop51       remove only the 5.1 sink
  restart51    recreate only the 5.1 sink
  start71      create only the 7.1 sink
  stop71       remove only the 7.1 sink
  restart71    recreate only the 7.1 sink
  status       show the sinks and their target

If target-sink is omitted, the default sink is used when it is a
Fireface/MR18 Pro Audio node; otherwise the first matching Pro Audio node
is selected.
EOF
}

target_sink() {
    local explicit="${1:-}"
    if [[ -n "$explicit" ]]; then
        printf '%s\n' "$explicit"
        return 0
    fi
    local default
    default="$(pactl get-default-sink 2>/dev/null || true)"
    if [[ "$default" =~ (Fireface|MR18).*pro-output-0 ]]; then
        printf '%s\n' "$default"
        return 0
    fi
    pactl list short sinks 2>/dev/null |
        awk '{print $2}' |
        grep -E '(Fireface|MR18).*pro-output-0' |
        head -n 1
}

kind_name() {
    case "$1" in
        51) printf '%s\n' totalcontrol-surround51 ;;
        71) printf '%s\n' totalcontrol-surround71 ;;
        *) return 1 ;;
    esac
}

kind_channels() {
    case "$1" in
        51) printf '%s\n' 'FL FR FC LFE SL SR' ;;
        71) printf '%s\n' 'FL FR FC LFE RL RR SL SR' ;;
        *) return 1 ;;
    esac
}

pid_file() { printf '%s/%s.pid\n' "$RUN_DIR" "$1"; }
target_file() { printf '%s/%s.target\n' "$RUN_DIR" "$1"; }

stop_one() {
    local kind="$1" pfile tfile pid
    pfile="$(pid_file "$kind")"
    tfile="$(target_file "$kind")"
    if [[ -f "$pfile" ]]; then
        read -r pid <"$pfile" || true
        if [[ -n "${pid:-}" ]]; then
            kill "$pid" 2>/dev/null || true
        fi
        rm -f "$pfile"
    fi
    rm -f "$tfile"
    local node
    node="$(kind_name "$kind")"
    local _i
    for _i in $(seq 1 20); do
        pactl list short sinks 2>/dev/null | awk '{print $2}' | grep -qx "$node" || break
        sleep 0.1
    done
}

wait_for_output() {
    local node="$1" channel="$2"
    local _i
    for _i in $(seq 1 50); do
        if pw-link -o 2>/dev/null | grep -qx "output.${node}:output_${channel}"; then
            return 0
        fi
        sleep 0.1
    done
    return 1
}

start_one() {
    local kind="$1" target="$2"
    local node channels channel idx address pid
    node="$(kind_name "$kind")"
    channels="$(kind_channels "$kind")"
    read -r -a channel_list <<<"$channels"
    local count="${#channel_list[@]}"

    if ! pactl list short sinks 2>/dev/null | awk '{print $2}' | grep -qx "$target"; then
        echo "Sink not found: $target" >&2
        exit 1
    fi
    stop_one "$kind"
    pw-loopback -n "$node" -c "$count" \
        -m "[ ${channel_list[*]} ]" \
        -i "media.class=Audio/Sink node.name=$node node.description=\"TotalControl Surround ${kind:0:1}.${kind:1:1}\"" \
        -P 0 -o 'node.autoconnect=false' &
    pid=$!
    printf '%s\n' "$pid" >"$(pid_file "$kind")"

    sleep 0.5
    if ! kill -0 "$pid" 2>/dev/null; then
        echo "The ${kind} sink failed to start" >&2
        stop_one "$kind"
        exit 1
    fi

    idx=0
    for channel in "${channel_list[@]}"; do
        wait_for_output "$node" "$channel" || {
            echo "The ${kind} sink did not create its ports" >&2
            stop_one "$kind"
            exit 1
        }
        address="playback_AUX$idx"
        pw-link "output.$node:output_$channel" "$target:$address" 2>/dev/null || true
        idx=$((idx + 1))
    done
    printf '%s\n' "$target" >"$(target_file "$kind")"
}

start_all() {
    local target
    target="$(target_sink "${1:-}")"
    if [[ -z "$target" ]]; then
        echo "No Fireface/MR18 Pro Audio sink found" >&2
        exit 1
    fi
    if ! pactl list short sinks 2>/dev/null | awk '{print $2}' | grep -qx "$target"; then
        echo "Sink not found: $target" >&2
        exit 1
    fi
    start_one 51 "$target"
    start_one 71 "$target"
    echo "TotalControl surround sinks are ready on $target"
    echo "  TotalControl Surround 5.1"
    echo "  TotalControl Surround 7.1"
}

stop_all() {
    stop_one 51
    stop_one 71
}

status() {
    local kind tfile
    for kind in 51 71; do
        tfile="$(target_file "$kind")"
        if [[ -f "$tfile" ]]; then
            echo "Target ${kind}: $(cat "$tfile")"
        fi
    done
    pactl list short sinks 2>/dev/null | grep 'totalcontrol-surround' || true
}

cmd="${1:-start}"
case "$cmd" in
    start)
        start_all "${2:-}"
        ;;
    stop)
        stop_all
        echo "TotalControl surround sinks stopped"
        ;;
    restart)
        stop_all
        start_all "${2:-}"
        ;;
    start51)
        target="$(target_sink "${2:-}")"
        [ -n "$target" ] || { echo "No Fireface/MR18 Pro Audio sink found" >&2; exit 1; }
        start_one 51 "$target"
        ;;
    stop51)
        stop_one 51
        ;;
    restart51)
        stop_one 51
        target="$(target_sink "${2:-}")"
        [ -n "$target" ] || { echo "No Fireface/MR18 Pro Audio sink found" >&2; exit 1; }
        start_one 51 "$target"
        ;;
    start71)
        target="$(target_sink "${2:-}")"
        [ -n "$target" ] || { echo "No Fireface/MR18 Pro Audio sink found" >&2; exit 1; }
        start_one 71 "$target"
        ;;
    stop71)
        stop_one 71
        ;;
    restart71)
        stop_one 71
        target="$(target_sink "${2:-}")"
        [ -n "$target" ] || { echo "No Fireface/MR18 Pro Audio sink found" >&2; exit 1; }
        start_one 71 "$target"
        ;;
    status)
        status
        ;;
    -h|--help|help)
        usage
        ;;
    *)
        usage >&2
        exit 2
        ;;
esac

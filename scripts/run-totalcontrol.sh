#!/usr/bin/env bash
# Build the current TotalControl sources and start that binary.
#
# The desktop entry points here so a menu launch never starts a stale
# development build. The release profile is the default; a running instance of
# any profile in this checkout is stopped after the new build succeeds, because
# the application keeps one instance per session.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKSPACE="$ROOT/ui/totalcontrol"
# Release by default: a menu launch is a daily-use run, and the dev profile
# builds this crate at opt-level 1, which draws a mixer several times more
# slowly than the release the GUI is shipped as. TOTALCONTROL_BUILD_PROFILE=dev
# restores the fast-to-build development binary.
PROFILE="${TOTALCONTROL_BUILD_PROFILE:-release}"
case "$PROFILE" in
    dev) OUT_DIR="debug" ;;
    *) OUT_DIR="$PROFILE" ;;
esac
BIN="$WORKSPACE/target/$OUT_DIR/totalcontrol-gui"

CARGO="${CARGO:-}"
if [ -z "$CARGO" ]; then
    CARGO="$(command -v cargo || true)"
fi
if [ -z "$CARGO" ] && [ -x "$HOME/.cargo/bin/cargo" ]; then
    CARGO="$HOME/.cargo/bin/cargo"
fi
if [ -z "$CARGO" ]; then
    "${NOTIFY_SEND:-notify-send}" "TotalControl" "cargo was not found" 2>/dev/null || true
    exit 1
fi

cd "$WORKSPACE"
if ! "$CARGO" build -p totalcontrol-gui --profile "$PROFILE" --quiet; then
    "${NOTIFY_SEND:-notify-send}" "TotalControl" "Build failed; see the terminal" 2>/dev/null || true
    exit 1
fi

# The single-instance guard would otherwise ask the stale process to show
# itself instead of starting the build we just made. Match every profile in
# this checkout, so switching between them does not leave one running.
pkill -f "$WORKSPACE/target/[^/]*/totalcontrol-gui" 2>/dev/null || true
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
    pgrep -f "$BIN" >/dev/null 2>&1 || break
    sleep 0.1
done

exec "$BIN" "$@"

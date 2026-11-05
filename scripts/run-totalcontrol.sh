#!/usr/bin/env bash
# Build the current TotalControl sources and start that binary.
#
# The desktop entry points here so a menu launch never starts a stale
# development build. A running debug instance is stopped after the new build
# succeeds, because the application keeps one instance per session.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKSPACE="$ROOT/ui/totalcontrol"
BIN="$WORKSPACE/target/debug/totalcontrol-gui"

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
if ! "$CARGO" build -p totalcontrol-gui --quiet; then
    "${NOTIFY_SEND:-notify-send}" "TotalControl" "Build failed; see the terminal" 2>/dev/null || true
    exit 1
fi

# The single-instance guard would otherwise ask the stale process to show
# itself instead of starting the build we just made.
pkill -f "$BIN" 2>/dev/null || true
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
    pgrep -f "$BIN" >/dev/null 2>&1 || break
    sleep 0.1
done

exec "$BIN" "$@"

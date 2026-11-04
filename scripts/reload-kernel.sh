#!/bin/sh
# Reload the locally built transport module and show the card/hwdep nodes.
#
# rmmod can fail for two reasons:
#   - PipeWire/wireplumber has the ALSA control/PCM open;
#   - the ALSA card itself holds a module reference until the USB interface
#     is disconnected.
# Stop the user audio services first if possible.  If rmmod still fails, this
# script unbinds the USB interface, which calls the driver's disconnect and
# frees the card.
set -eu
cd "$(dirname "$0")/.."

if lsmod | grep -q '^snd_fireface_uc '; then
	if ! sudo rmmod snd_fireface_uc; then
		echo "module still in use; checking holders..." >&2
		echo "  refcnt: $(cat /sys/module/snd_fireface_uc/refcnt 2>/dev/null || echo '?')" >&2
		# An unplugged unit can leave a process holding the removed hwdep
		# node (TotalControl is the usual one), keeping the module reference.
		lsof +L1 2>/dev/null | grep -E 'snd/(hwC|pcmC)' >&2 || true
		IFACE=$(ls -d /sys/bus/usb/drivers/snd-fireface-uc/*:*.* 2>/dev/null |
			head -1 | xargs -r -n1 basename)
		if [ -z "$IFACE" ]; then
			echo "no bound USB interface to unbind." >&2
			echo "Close TotalControl, or stop the audio services:" >&2
			echo "  systemctl --user stop wireplumber pipewire-pulse pipewire" >&2
			echo "then rerun this script." >&2
			exit 1
		fi
		echo "unbinding $IFACE" >&2
		echo -n "$IFACE" | sudo tee /sys/bus/usb/drivers/snd-fireface-uc/unbind >/dev/null
		if ! sudo rmmod snd_fireface_uc; then
			echo "module is still in use; close TotalControl or unplug the Fireface, then rerun." >&2
			exit 1
		fi
	fi
fi

sudo modprobe snd-pcm
# Any extra arguments are passed to insmod, e.g.
#   ./scripts/reload-kernel.sh urb_frames=48 nurbs=8 ucx_if5=Y
sudo insmod kernel/snd-fireface-uc.ko "$@"
echo '--- fireface log ---'
sudo dmesg | grep -i fireface | tail -3 || true
echo '--- card ---'
aplay -l | grep -A2 -i fireface || true
echo '--- hwdep control node ---'
# The kernel exposes raw vendor/register/meter transports here; mixer state
# lives in TotalControl (see docs/HWDEP-ABI.md).
ls -l /dev/snd/hwC*D* 2>/dev/null || true

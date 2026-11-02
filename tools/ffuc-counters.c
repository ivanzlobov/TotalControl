// ffuc-counters - read the snd-fireface-uc hwdep transport counters.
//
// Build: gcc -O2 -I../kernel -o ffuc-counters ffuc-counters.c
// Run:   ./ffuc-counters [/dev/snd/hwC2D0]
//
// The counters are cumulative since the module loaded.  scripts/audio-health.sh
// samples them together with PipeWire's ALSA xrun count and prints deltas.

#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <linux/types.h>
#include "ffuc-uapi.h"
int main(int argc, char **argv) {
    const char *path = argc > 1 ? argv[1] : "/dev/snd/hwC2D0";
    int fd = open(path, O_RDONLY);
    if (fd < 0) { perror(path); return 1; }
    struct ffuc_hwdep_counters c;
    if (ioctl(fd, FFUC_HWDEP_IOCTL_COUNTERS, &c) < 0) { perror("ioctl"); return 1; }
    printf("out_urbs=%u out_gaps=%u out_shorts=%u out_status_errors=%u urb_errors=%u recoveries=%u events_dropped=%u meter_urbs=%u state_urbs=%u status_urbs=%u\n",
        c.out_urbs, c.out_gaps, c.out_shorts, c.out_status_errors, c.urb_errors,
        c.recoveries, c.events_dropped, c.meter_urbs, c.state_urbs, c.status_urbs);
    printf("in_urbs=%u in_gaps=%u in_shorts=%u in_status_errors=%u\n",
        c.in_urbs, c.in_gaps, c.in_shorts, c.in_status_errors);
    close(fd);
    return 0;
}

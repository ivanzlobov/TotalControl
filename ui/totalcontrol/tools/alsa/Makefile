# ALSA plugin for the TotalControl Babyface Pro FS proprietary driver.
#
# Build:       make
# Install:     make install   (needs root: copies the .so + the ALSA conf)
# Uninstall:   make uninstall
#
# The Rust side must be built first (release):  cargo build -p totalcontrol-sys --release

CC      ?= gcc
TARGET   = libasound_module_pcm_totalcontrol.so
RELEASE  = ../../target/release
CFLAGS  += -O2 -Wall -Wextra -fPIC -DPIC -I../../totalcontrol-sys/include $(shell pkg-config --cflags alsa)
LIBS    += -shared -lasound -L$(RELEASE) -ltotalcontrol_sys
# For local testing: the plugin finds libtotalcontrol_sys.so in the cargo
# target dir. For a system install, copy libtotalcontrol_sys.so to /usr/lib
# and drop the -rpath (see install).
RPATH   := -Wl,-rpath,$(abspath $(RELEASE))

all: $(TARGET)

$(TARGET): pcm_totalcontrol.c ../../totalcontrol-sys/include/totalcontrol_sys.h
	$(CC) $(CFLAGS) -o $@ pcm_totalcontrol.c $(LIBS) $(RPATH)

install: $(TARGET)
	install -D -m 644 $(TARGET) /usr/lib/alsa-lib/$(TARGET)
	install -D -m 644 ../../totalcontrol-sys/include/totalcontrol_sys.h /usr/include/totalcontrol_sys.h
	install -D -m 644 $(RELEASE)/libtotalcontrol_sys.so /usr/lib/libtotalcontrol_sys.so
	install -D -m 644 50-totalcontrol.conf /etc/alsa/conf.d/50-totalcontrol.conf
	ldconfig

uninstall:
	rm -f /usr/lib/alsa-lib/$(TARGET)
	rm -f /usr/include/totalcontrol_sys.h
	rm -f /usr/lib/libtotalcontrol_sys.so
	rm -f /etc/alsa/conf.d/50-totalcontrol.conf

clean:
	rm -f $(TARGET)

.PHONY: all install uninstall clean

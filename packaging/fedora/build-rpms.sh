#!/bin/sh
# Build the Fedora release RPMs locally.
#
# Requirements on Fedora:
#   sudo dnf install rpm-build rpmdevtools cargo rust gcc gcc-c++ make \
#       pkgconf-pkg-config alsa-lib-devel mesa-libGL-devel libX11-devel \
#       libxkbcommon-devel wayland-devel systemd-devel fontconfig-devel \
#       freetype-devel openssl-devel desktop-file-utils git tar gzip
#
# The build uses the checked-out commit as its source.  Outputs are written
# to packaging/fedora/out/ (git-ignored).

set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
VERSION=$(sed -n 's/^version = "\(.*\)"$/\1/p' \
    "$ROOT/ui/totalcontrol/totalcontrol-gui/Cargo.toml" | head -n 1)
[ -n "$VERSION" ] || {
    echo "could not read the application version" >&2
    exit 1
}

TOP=${RPM_TOPDIR:-"$ROOT/packaging/fedora/.rpmbuild"}
OUT="$ROOT/packaging/fedora/out"
SPECS="$ROOT/packaging/fedora"

rm -rf "$TOP" "$OUT"
mkdir -p "$TOP"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS} "$OUT"

git -C "$ROOT" archive --format=tar.gz \
    --prefix="TotalControl-$VERSION/" HEAD \
    > "$TOP/SOURCES/TotalControl-$VERSION.tar.gz"

cp "$SPECS/totalcontrol.spec" "$SPECS/totalcontrol-kmod.spec" "$TOP/SPECS/"

rpmbuild --define "_topdir $TOP" --define "app_version $VERSION" \
    -ba "$TOP/SPECS/totalcontrol.spec"
rpmbuild --define "_topdir $TOP" --define "app_version $VERSION" \
    -ba "$TOP/SPECS/totalcontrol-kmod.spec"

find "$TOP/RPMS" -type f -name '*.rpm' -exec cp -p {} "$OUT/" \;
echo "RPMs written to $OUT:"
find "$OUT" -maxdepth 1 -type f -name '*.rpm' -printf '  %f\n' | sort

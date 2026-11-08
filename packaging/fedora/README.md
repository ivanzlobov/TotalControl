# Fedora RPM packages

These are convenience packages for Fedora, built from a tagged source archive
and published as GitHub release assets.  They are not part of the official
Fedora repositories.

Two packages are produced:

| Package | Contents |
|---|---|
| `totalcontrol` | `totalcontrol` GUI, `totalcontrol-tui`, desktop entry and hicolor icons |
| `totalcontrol-kmod` | `snd-fireface-uc` source installed through DKMS, plus its boot-time load service |

The kernel module is separate because it must be rebuilt for each kernel.  It
requires `dkms`, `kernel-devel`, `gcc` and `make`; matching headers for the
running kernel must be installed.  On Fedora with UEFI Secure Boot, sign the
DKMS module and enrol the key in MOK before it can load.

## Build locally

On Fedora:

```sh
sudo dnf install rpm-build cargo rust gcc gcc-c++ make \
    pkgconf-pkg-config alsa-lib-devel mesa-libGL-devel libX11-devel \
    libxkbcommon-devel wayland-devel systemd-devel fontconfig-devel \
    freetype-devel openssl-devel desktop-file-utils git tar gzip

./packaging/fedora/build-rpms.sh
```

The RPMs are written to `packaging/fedora/out/`.

Install the module first, then the application:

```sh
sudo dnf install ./packaging/fedora/out/totalcontrol-kmod-*.rpm
sudo dnf install ./packaging/fedora/out/totalcontrol-[0-9]*.rpm
```

After installing matching kernel headers later, retry the module build with:

```sh
sudo dkms autoinstall
```

## GitHub release

`.github/workflows/fedora-rpms.yml` builds the RPMs in a Fedora container.
On a published GitHub release it attaches the two primary RPMs to that
release; on a manual workflow run it uploads all RPMs as workflow artifacts.

The workflow is pinned to Fedora 44.  After moving to Fedora 45, change the
`container:` image and publish a new release from the updated `main`; the RPM
release suffix changes from `.fc44` to `.fc45` automatically.  Publish the
release rather than leaving it as a draft so the workflow runs.

The application version comes from
`ui/totalcontrol/totalcontrol-gui/Cargo.toml`; the kernel module version comes
from `kernel/dkms.conf`.  Keep the two spec files in step when either changes.

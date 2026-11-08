# RPM spec for the TotalControl user-space application.
#
# These are convenience packages hosted as GitHub release assets, not an
# official Fedora package.  Build with packaging/fedora/build-rpms.sh.

%{!?app_version:%global app_version 0.3.1}

Name:           totalcontrol
Version:        %{app_version}
Release:        1%{?dist}
Summary:        TotalMix-compatible mixer for RME Fireface interfaces
License:        GPL-3.0-only
URL:            https://github.com/jspx-projects/TotalControl
Source0:        TotalControl-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gcc
BuildRequires:  gcc-c++
BuildRequires:  make
BuildRequires:  pkgconf-pkg-config
BuildRequires:  alsa-lib-devel
BuildRequires:  mesa-libGL-devel
BuildRequires:  libX11-devel
BuildRequires:  libxkbcommon-devel
BuildRequires:  wayland-devel
BuildRequires:  systemd-devel
BuildRequires:  fontconfig-devel
BuildRequires:  freetype-devel
BuildRequires:  openssl-devel
BuildRequires:  desktop-file-utils
BuildRequires:  tar
BuildRequires:  gzip

Requires:       hicolor-icon-theme

%description
TotalControl is a Linux control surface for RME Fireface UC and UCX
interfaces in normal vendor mode.  It replaces the vendor mixer application
with direct control of routing, gains, EQ, dynamics, FX, snapshots and the
Control Room.

This package installs the GUI and TUI binaries, desktop entry and icons.
The kernel transport module is provided by the separate totalcontrol-kmod
package.

%prep
%autosetup -n TotalControl-%{version}

%build
cd ui/totalcontrol
cargo build --release --locked -p totalcontrol-gui -p totalcontrol-tui

%install
install -Dm755 ui/totalcontrol/target/release/totalcontrol-gui %{buildroot}%{_bindir}/totalcontrol
install -Dm755 ui/totalcontrol/target/release/totalcontrol-tui %{buildroot}%{_bindir}/totalcontrol-tui
install -Dm644 ui/totalcontrol/totalcontrol-gui/assets/totalcontrol.desktop \
    %{buildroot}%{_datadir}/applications/totalcontrol.desktop
install -Dm644 ui/totalcontrol/totalcontrol-gui/assets/totalcontrol.svg \
    %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/totalcontrol.svg
install -Dm644 ui/totalcontrol/totalcontrol-gui/assets/totalcontrol-128.png \
    %{buildroot}%{_datadir}/icons/hicolor/128x128/apps/totalcontrol.png
install -Dm644 ui/totalcontrol/totalcontrol-gui/assets/totalcontrol-256.png \
    %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/totalcontrol.png
install -Dm644 LICENSE %{buildroot}%{_licensedir}/totalcontrol/LICENSE
install -Dm644 README.md %{buildroot}%{_docdir}/totalcontrol/README.md

%check
desktop-file-validate ui/totalcontrol/totalcontrol-gui/assets/totalcontrol.desktop

%post
update-desktop-database &>/dev/null || :
touch --no-create %{_datadir}/icons/hicolor &>/dev/null || :

%postun
update-desktop-database &>/dev/null || :
touch --no-create %{_datadir}/icons/hicolor &>/dev/null || :

%files
%{_bindir}/totalcontrol
%{_bindir}/totalcontrol-tui
%{_datadir}/applications/totalcontrol.desktop
%{_datadir}/icons/hicolor/scalable/apps/totalcontrol.svg
%{_datadir}/icons/hicolor/128x128/apps/totalcontrol.png
%{_datadir}/icons/hicolor/256x256/apps/totalcontrol.png
%license %{_licensedir}/totalcontrol/LICENSE
%doc %{_docdir}/totalcontrol/README.md

%changelog
* Sun Nov 08 2026 TotalControl contributors <noreply@localhost> - 0.3.1-1
- Initial Fedora convenience package.

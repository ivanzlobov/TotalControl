# RPM spec for the TotalControl DKMS kernel module.
#
# The module is out-of-tree and must be rebuilt for every kernel.  A DKMS
# package keeps that maintenance on the target machine; installing it without
# matching kernel headers leaves the source in /usr/src and dkms.service
# retries the build after the headers are installed.

%global kmod_name     snd-fireface-uc
%global kmod_version  0.1
%{!?app_version:%global app_version 0.3.1}

Name:           totalcontrol-kmod
Version:        %{kmod_version}
Release:        1%{?dist}
Summary:        DKMS kernel transport for RME Fireface UC/UCX
License:        GPL-2.0-only
URL:            https://github.com/jspx-projects/TotalControl
Source0:        TotalControl-%{app_version}.tar.gz

BuildArch:      noarch

Requires:       dkms
Requires:       kernel-devel
Requires:       gcc
Requires:       make
Requires:       systemd

%description
This package installs the snd-fireface-uc kernel transport source through
DKMS.  The module provides the ALSA PCM device and the hwdep control node used
by TotalControl; it does not contain the mixer application itself.

Matching kernel headers and DKMS are required.  With UEFI Secure Boot enabled,
the generated module must be signed and the signing key enrolled in MOK before
the kernel will load it.

%prep
%autosetup -n TotalControl-%{app_version}

%build

%install
install -d %{buildroot}%{_usrsrc}/%{kmod_name}-%{kmod_version}
cp -a kernel/. %{buildroot}%{_usrsrc}/%{kmod_name}-%{kmod_version}/
find %{buildroot}%{_usrsrc}/%{kmod_name}-%{kmod_version} \
    -maxdepth 1 -type f \
    \( -name '*.o' -o -name '*.ko' -o -name '*.mod' -o -name '*.mod.c' \
       -o -name 'modules.order' -o -name 'Module.symvers' \) -delete
rm -rf %{buildroot}%{_usrsrc}/%{kmod_name}-%{kmod_version}/.tmp_versions
install -Dm644 packaging/systemd/snd-fireface-uc-load.service \
    %{buildroot}%{_unitdir}/snd-fireface-uc-load.service

%post
if [ -x /usr/sbin/dkms ]; then
    /usr/sbin/dkms add -m %{kmod_name} -v %{kmod_version} --force || :
    /usr/sbin/dkms build -m %{kmod_name} -v %{kmod_version} -k "$(uname -r)" --force || :
    /usr/sbin/dkms install -m %{kmod_name} -v %{kmod_version} -k "$(uname -r)" --force || :
fi
systemctl daemon-reload || :
systemctl enable snd-fireface-uc-load.service || :

%preun
if [ "$1" -eq 0 ]; then
    systemctl disable --now snd-fireface-uc-load.service || :
    if [ -x /usr/sbin/dkms ]; then
        /usr/sbin/dkms remove -m %{kmod_name} -v %{kmod_version} --all || :
    fi
fi

%postun
systemctl daemon-reload || :

%files
%{_usrsrc}/%{kmod_name}-%{kmod_version}
%{_unitdir}/snd-fireface-uc-load.service

%changelog
* Sun Nov 08 2026 TotalControl contributors <noreply@localhost> - 0.1-1
- Initial DKMS Fedora convenience package.

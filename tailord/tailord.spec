Name:           tailord
Version:        0.2.5
Release:        1%{?dist}
Summary:        Tailord hardware control deamon

License:        GPLv2
URL:            https://github.com/AaronErhardt/tuxedo-rs
Source0:        tailord.tar.gz

Requires:       systemd
BuildRequires:  systemd
BuildRequires:  meson
BuildRequires:  ninja-build
BuildRequires:  rust-packaging >= 21

%description
Tailord hardware control deamon. Part of tuxedo-rs.

%prep
%autosetup -c

%build
cd tailord
%meson
%meson_build

%install
cd tailord
%meson_install

%post
systemctl daemon-reload
systemctl enable tailord.service
systemctl start tailord.service

%preun
systemctl stop tailord.service
systemctl disable tailord.service
systemctl daemon-reload

%files
%license LICENSE
/usr/bin/tailord
/usr/lib/systemd/system/tailord.service
/usr/share/dbus-1/system.d/com.tux.Tailor.conf

%changelog
%autochangelog

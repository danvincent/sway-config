Name:           sway-config
Version:        0.1.0
Release:        1%{?dist}
Summary:        GTK4/libadwaita GUI for configuring Sway and Waybar
License:        MIT
URL:            https://github.com/danvincent/sway-config
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  pkgconfig(gtk4)
BuildRequires:  pkgconfig(libadwaita-1)
BuildRequires:  pkgconfig(pango)
BuildRequires:  pkgconfig(cairo)
BuildRequires:  pkgconfig(gdk-pixbuf-2.0)
BuildRequires:  pkgconfig(graphene-1.0)

Requires:       sway
Requires:       waybar

%description
Rust + GTK4/libadwaita application for configuring Sway and Waybar.

%prep
%autosetup

%build
cargo build --release --locked

%install
install -Dpm0755 target/release/sway-config %{buildroot}%{_bindir}/sway-config
install -Dpm0644 packaging/sway-config.desktop %{buildroot}%{_datadir}/applications/sway-config.desktop
install -Dpm0644 packaging/sway-config.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/sway-config.svg
install -Dpm0644 README.md %{buildroot}%{_docdir}/%{name}/README.md
install -Dpm0644 LICENSE %{buildroot}%{_docdir}/%{name}/LICENSE

%files
%license LICENSE
%doc README.md
%{_bindir}/sway-config
%{_datadir}/applications/sway-config.desktop
%{_datadir}/icons/hicolor/scalable/apps/sway-config.svg

%changelog
* Fri May 22 2026 Daniel Vincent <daniel@example.com> - 0.1.0-1
- Initial RPM packaging

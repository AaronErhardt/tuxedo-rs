<div align="center">
  <img alt="Tuxedo-rs" src="assets/tuxedo-rs-mascots.png#gh-light-mode-only" width="132" />
  <img alt="Tuxedo-rs" src="assets/tuxedo-rs-mascots-dark.png#gh-dark-mode-only" width="132" />
  <h1>Tuxedo-rs 🐧+🦀=❤️</h1>
  <p>
    <strong>
      Rust libraries for interacting with hardware from <a href="https://www.tuxedocomputers.com">TUXEDO Computers</a>.
	</strong>
  </p>
</div>

> Tuxedo-rs is a community project. It is not developed nor supported by TUXEDO Computers.  
> Although we only supports TUXEDO hardware for now, other vendors are free to contribute their own bindings.

## Motivation

The TUXEDO Control Center (TCC) is a neat application that allows you to control different parts of your hardware, such as fans, webcam and performance profiles.
However, TCC and its tccd service rely on Node.js which makes it slow, memory hungry and hard to package.

Also, tuxedo-rs is modular and contains several crates with different levels of abstraction, which makes it easy for everyone to built their own tools on top.

### Why Rust?

- ~~All software should be rewritten in Rust~~
- Very robust code
- Strong compile-time guarantees
- High performance
- Easy to package (no additional runtime or dependencies)

## Structure

<img alt="Project structure" src="assets/structure_light.png#gh-light-mode-only" width="480" />
<img alt="Project structure" src="assets/structure_dark.png#gh-dark-mode-only" width="480" />

## Tested hardware

This list includes devices that were successfully tested with tuxedo-rs.
Since I have limited access to hardware, please consider adding your device(s) to the list.

+ TUXEDO Aura 15 Gen1 and Gen2
+ TUXEDO Pulse 15 Gen1
+ TUXEDO Polaris 17 Gen3

## Installation

Currently, tuxedo-rs isn't available from package archives so you have to install it from source.

### Tuxedo driver modules

If you use a distribution that doesn't package the required tuxedo hardware drivers, you can install the from [source](https://github.com/tuxedocomputers/tuxedo-keyboard).

```sh
git clone https://github.com/tuxedocomputers/tuxedo-keyboard.git
cd tuxedo-keyboard
git checkout release
sudo make dkmsinstall
```

### Tailord

Tailord is the system service that runs in the background and interacts with the driver modules.
It exposes a D-BUS interface that can be used by applications to configure the hardware.

```sh
cd tailord
meson setup --prefix=/usr _build
ninja -C _build
ninja -C _build install
```

If you have the TUXEDO Control Center (TCC) and its daemons installed, make sure to deactivate them first.

```sh
sudo systemctl disable tccd.service 
sudo systemctl stop tccd.service 
sudo systemctl disable tccd-sleep.service 
sudo systemctl stop tccd-sleep.service 
```

Then, enable tailord with the following commands:

```sh
sudo systemctl enable tailord.service 
sudo systemctl start tailord.service 
```

### Tailor GUI

Tailor GUI will soon be available as flatpak package. 
In the meantime, you can build the app from source.
Usually, building the app only requires GNOME Builder or the flatpak extension for VSCode.
Yet, you can use the command line if your prefer it:

```sh
flatpak install org.gnome.Sdk//44 org.freedesktop.Sdk.Extension.rust-stable//22.08 org.gnome.Platform//44 runtime/org.freedesktop.Sdk.Extension.llvm15//22.08
flatpak-builder --user flatpak_app tailor_gui/build-aux/com.github.aaronerhardt.Tailor.json 
flatpak-builder --run flatpak_app tailor_gui/build-aux/com.github.aaronerhardt.Tailor.json tailor_gui
```

If you don't want to use flatpak-builder, make sure you have the following dependencies installed on your system.

Ubuntu 23.04:

```sh
sudo apt install meson libadwaita-1-dev libgtk-4-dev
```

Arch Linux:

```sh
sudo pacman -S meson libadwaita gtk4
```

Fedora 38:

```sh
sudo dnf -y install meson libadwaita-devel gtk4-devel
```

Then build and install Tailor GUI with meson:

```sh
cd tailor_gui
meson setup --prefix=/usr _build
ninja -C _build
ninja -C _build install
```

### On NixOS

To install tailor-rs on NixOS, with flakes enabled, add
the following to your flake.nix:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    tuxedo-rs.url = "github:AaronErhardt/tuxedo-rs";
  }

  outputs = {
    nixpkgs,
    tuxedo-rs,
   ...
  } : {
    nixosConfigurations.myConfiguration = nixpkgs.lib.nixosSystem {
      modules = [
        tuxedo-rs.nixosModules.default
      ];

      services.tuxedo-rs = {
        enable = true;
        tailor_gui.enable = true;
        tailor_cli.enable = true;
      };
    };
  };
}
```

## Roadmap

- [x] Ioctl abstraction for tuxedo_io
- [x] Sysfs abstraction for tuxedo_keyboard
- [x] Support for hardware from both clevo and uniwill
- [x] Daemon with DBus interface for user space application
- [x] Client library for interacting with the daemon
- [x] CLI that interacts with the daemon
- [x] Native GUI that interacts with the daemon
- [ ] OPTIONAL: Rewrite various tuxedo kernel modules in Rust

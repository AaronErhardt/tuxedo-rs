# Tuxedo-rs 🐧+🦀=❤️

**Rust libraries for interacting with hardware from [TUXEDO Computers](https://www.tuxedocomputers.com).**

> This is a community project. It is not developed or supported by TUXEDO Computers.

> Although this is tailored towards TUXEDO hardware for now, other vendors are free to contribute their own bindings.

## Motivation

The TUXEDO Control Center (TCC) is a neat application that allows you to control different parts of your hardware, such as fans, webcam and performance profiles.
However, TCC and its tccd service rely on Node.js which makes it slow, memory hungry and hard to package.

Also, we keep the hardware abstractions and other utilities in individual crates to allow others to build their own applications on top.

### Why Rust?

- ~~All software should be rewritten in Rust~~
- Very robust code
- Strong compile-time guarantees
- High performance
- Easy to package (no additional runtime or dependencies)

## Roadmap

- [x] Ioctl abstraction for tuxedo_io
- [x] Additional hardware abstractions (just limited features)
- [x] Deamon with DBus interface for user space application (just limited features)
- [ ] CLI that interacts with the deamon
- [ ] Native GUI that interacts with the deamon
- [ ] OPTIONAL: Rewrite various tuxedo kernel modules in Rust
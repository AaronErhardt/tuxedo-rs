# tuxedo_fancontrol

Yet another tool to control your TUXEDO Computers fans.

This is a **community project**. It is **not** developed or supported by TUXEDO Computers. It is based on [the work of AaronErhardt](https://github.com/AaronErhardt/tuxedo-rs) licensed under GPLv2.

This software *may* work on other Clevo-branded laptops like system76.

### Why?

Their laptops does not regulate their fan properly.

I have experienced the following irregularities with my model (Aura 15 Gen1):
- The fan is loud when idle (it defaults at 30% speed at all times).
- The fan sometimes does not start when it should (it stays at 30% speed even if there’s heavy load and the temperature raises at 80°C)
- Fan control is quite binary: when the fan actually *starts*, it kinda shifts between 30% and 90% speed.

TUXEDO has released [a tool](https://github.com/tuxedocomputers/tuxedo-control-center) to control fan speed, it consumes approximately 5% CPU and lots of RAM when idle. It is also written in TypeScript (along with an electron frontend), and I feel personally quite uncomfortable running bloated JavaScript applications with full root rights in background.

So here’s a Rust™ app.

### Disclaimer

**I am not responsible for any damage this software may cause to your computer.**

Be aware that **this software will run as root** and interact with low-level APIs to control your computer fans.

A bad configuration or a some bug **may cause overheats and permanent hardware damage**.

The software is in **development** state (not even alpha). It is suited **for testing purposes only**.

### How to use

1. `cd tuxedo_fancontrol`
2. Edit `config.toml` to adjust it to your needs. Don’t edit if you don’t know what you are doing.
3. `cargo run --release` and just leave it alone.

There are no instructions to run it in background with a systemd service yet (feel free to contribute).

### The algorithm

I suck at math, so I did not use any standard / existing math algorithm to control fans. I reinvented the wheel again. Just [read the code](https://git.42l.fr/neil/tuxedo-fancontrol/src/branch/main/tuxedo_fancontrol/src/fan.rs#L116).

If you’re willing to implement some trusty fan control algorithm, here are some hints:
- https://en.wikipedia.org/wiki/Hysteresis
- https://en.wikipedia.org/wiki/PID_controller

The current algorithm uses some magic nonsensical numbers as key indicators. It is meant to be improved. Feel free to edit it yourself.

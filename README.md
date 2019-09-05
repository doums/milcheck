[![Build Status](https://travis-ci.org/doums/milcheck.svg?branch=master)](https://travis-ci.org/doums/milcheck)

## milcheck

###### for **MI**rror**L**ist **Check**

![milcheck](https://image.petitmur.beer/milcheck.png)

A small binary to display status of your pacman mirrorlist in your terminal, written in [Rust](https://www.rust-lang.org/)

#### How ?

milcheck reads your `/etc/pacman.d/mirrorlist` and retrieves the corresponding data from the official [mirror status page](https://www.archlinux.org/mirrors/status/)

#### Why ?

As explained in the [mirror doc](https://wiki.archlinux.org/index.php/Mirrors), before a system upgrade i.e. `sudo pacman -Syu`, you have to check that the mirrors in your mirrorlist are up to date e.g. not out of sync.

#### Ready to shake..

Rust is a language that compiles to native code and by default statically links all dependencies. All you have to do is simply download the latest release of the precompiled binary [here](https://github.com/doums/milcheck/releases) and use it!
(do not forget to make it executable `chmod 755 milcheck`)

#### Install from [crates.io](https://crates.io/crates/milcheck)

install Rust -> https://www.rust-lang.org/tools/install
```
cargo install milcheck
```

#### Build from sources

install Rust -> https://www.rust-lang.org/tools/install
```
git clone https://github.com/doums/milcheck.git
cd milcheck
cargo build
```
to build for release
```
cargo build --release
```
the binary is located under `target/debug` or `target/release`

#### Usage

```
milcheck
```

#### License
Mozilla Public License 2.0

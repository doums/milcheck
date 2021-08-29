[![milcheck](https://img.shields.io/github/workflow/status/doums/milcheck/Rust?color=0D0D0D&logoColor=BFBFBF&labelColor=404040&logo=github&style=for-the-badge)](https://github.com/doums/milcheck/actions?query=workflow%3ARust)
[![milcheck](https://img.shields.io/aur/version/milcheck?color=0D0D0D&logoColor=BFBFBF&labelColor=404040&logo=arch-linux&style=for-the-badge)](https://aur.archlinux.org/packages/milcheck/)

## milcheck

:tea: _**MI**rror**L**ist **Check**_

![milcheck](https://raw.githubusercontent.com/doums/milcheck/master/public/milcheck.png)

A small binary that displays the status of your pacman mirrorlist in your terminal and optionally the lastest news

### How ?

milcheck just reads your `/etc/pacman.d/mirrorlist` and retrieves the corresponding data from the official [mirror status page](https://www.archlinux.org/mirrors/status/)

### Why ?

As explained in the [mirror doc](https://wiki.archlinux.org/index.php/Mirrors),\
before a system upgrade i.e. `sudo pacman -Syu`, you must check that the mirrors in your mirrorlist are up to date e.g. not out of sync.

### It's not

..an additional mirrorlist ranking utility

### Install

Rust is a language that compiles to native code and by default statically links all dependencies.\
Simply download the latest [release](https://github.com/doums/milcheck/releases) of the precompiled binary and use it!
(do not forget to make it executable `chmod 755 milcheck`)

### Install from [crates.io](https://crates.io/crates/milcheck)

install Rust -> https://www.rust-lang.org/tools/install
```
cargo install milcheck
```

### Arch Linux AUR package

milcheck is present as a [package](https://aur.archlinux.org/packages/milcheck) in the Arch User Repository.

### Build from sources

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

### Usage

```
milcheck
```

you can print the lastest news, handy to stay informed
```
milcheck -n
```

### License
Mozilla Public License 2.0

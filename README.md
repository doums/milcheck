[![milcheck](https://img.shields.io/github/actions/workflow/status/doums/milcheck/rust.yml?color=0D0D0D&logoColor=BFBFBF&labelColor=404040&logo=github&style=for-the-badge)](https://github.com/doums/milcheck/actions?query=workflow%3ARust)
[![milcheck](https://img.shields.io/aur/version/milcheck?color=0D0D0D&logoColor=BFBFBF&labelColor=404040&logo=arch-linux&style=for-the-badge)](https://aur.archlinux.org/packages/milcheck/)

## milcheck

:tea: _**MI**rror**L**ist **Check**_

![milcheck](https://raw.githubusercontent.com/doums/milcheck/master/public/milcheck.png)

A simple CLI that displays the status of your pacman mirrorlist
and the Arch Linux latest news right in the terminal

### How ?

Milcheck just reads your `/etc/pacman.d/mirrorlist` and retrieves
the corresponding data from the official
[mirror status page](https://www.archlinux.org/mirrors/status/).

The latest news are directly scraped from https://archlinux.org/.

### Why ?

As explained in the
[mirror doc](https://wiki.archlinux.org/index.php/Mirrors), before
a system upgrade i.e. `sudo pacman -Syu`, you should check that
the mirrors in your mirrorlist are up to date e.g. not out of
sync.

### It's not

..an additional mirrorlist ranking utility.

### Install from [crates.io](https://crates.io/crates/milcheck)

Install Rust -> https://www.rust-lang.org/tools/install

```
cargo install milcheck
```

### Arch Linux AUR package

Milcheck is present as a
[package](https://aur.archlinux.org/packages/milcheck) in the Arch
User Repository.

### Build from sources

Install Rust -> https://www.rust-lang.org/tools/install

```
git clone https://github.com/doums/milcheck.git
cd milcheck
cargo build
```

To build for release

```
cargo build --release
```

The binary is located under `target/debug` or `target/release`.

### Pre-built binary

Rust is a language that compiles to native code and by default
statically links. Simply download the pre-built binary from latest
[release](https://github.com/doums/milcheck/releases/latest).

### Usage

```
milcheck
```

In addition to the mirrorlist output you can print the Arch Linux
[lastest news](https://archlinux.org/), handy to stay informed:

```
milcheck -n
```

### License

Mozilla Public License 2.0

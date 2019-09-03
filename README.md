## milcheck

###### for **MI**rror**L**ist **Check**

!(milcheck)[https://image.petitmur.beer/milcheck.png]

A small binary to display status of your pacman mirrorlist in your terminal, written in [Rust](https://www.rust-lang.org/)

#### How ?

milcheck reads your `/etc/pacman.d/mirrorlist` and retrieves the corresponding data from the official [mirror status list](https://www.archlinux.org/mirrors/status/)

#### Why ?

As explained in the [mirror doc](https://wiki.archlinux.org/index.php/Mirrors) before a system upgrade i.e. `sudo pacman -Syu`, you have to be sure that the mirrors in your mirrorlist are up to date i.e. not out of sync.

#### Install from [crates.io](https://crates.io/crates/milcheck)

install Rust -> https://www.rust-lang.org/tools/install
```
cargo install milcheck
milcheck
```

#### Build from sources

install Rust -> https://www.rust-lang.org/tools/install
```
git clone https://github.com/doums/milcheck.git
cd milcheck
cargo build
```
or to build for release
```
cargo build --release
```
the binary is located under `target/debug` or `./target/release`

#### Usage

```
milcheck
```

#### License
Mozilla Public License 2.0

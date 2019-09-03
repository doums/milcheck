## milcheck

###### for **MI**rror**L**ist **Check**

A small binary to display status of your pacman mirrorlist in your terminal, written in [Rust](https://www.rust-lang.org/)

#### How ?

milcheck reads your `/etc/pacman.d/mirrorlist` and retrieves the corresponding data from the official [mirror status list](https://www.archlinux.org/mirrors/status/)

#### Why ?

As explained in the [mirror doc](https://wiki.archlinux.org/index.php/Mirrors) before a system upgrade i.e. `sudo pacman -Syu`, you have to be sure that the mirrors in your mirrorlist are up to date i.e. not out of sync.

#### Build

install Rust -> https://www.rust-lang.org/tools/install
then

```
git clone https://github.com/doums/milcheck.git
cd milcheck
cargo build --release
```
You can find the binary in `./target/release/milcheck`

#### Usage

```
milcheck
```

#### License
Mozilla Public License 2.0

[![milcheck](https://img.shields.io/github/actions/workflow/status/doums/milcheck/test.yml?color=0D0D0D&logoColor=BFBFBF&labelColor=404040&logo=github&style=for-the-badge)](https://github.com/doums/milcheck/actions?query=workflow%3ATest)
[![milcheck](https://img.shields.io/aur/version/milcheck?color=0D0D0D&logoColor=BFBFBF&labelColor=404040&logo=arch-linux&style=for-the-badge)](https://aur.archlinux.org/packages/milcheck/)

## milcheck

:tea: _**MI**rror**L**ist **Check**_

![milcheck](https://github.com/doums/milcheck/assets/6359431/939c72ac-72f8-4ade-8155-ec8f66ba3c0b)

A CLI that displays the status of your pacman mirrorlist
and the Arch Linux latest news right in the terminal

### How ?

Milcheck just reads your `/etc/pacman.d/mirrorlist` and retrieves
the corresponding data from the official
[mirror status page](https://www.archlinux.org/mirrors/status/).

The last news are directly scraped from https://archlinux.org/.

### Why ?

As explained in the
[mirror doc](https://wiki.archlinux.org/index.php/Mirrors), before
a system upgrade i.e. `sudo pacman -Syu`, you should check that
the mirrors in your mirrorlist are up-to-date e.g. not out of
sync.

### It's not

..an additional mirrorlist ranking utility.

### Install

- latest [release](https://github.com/doums/milcheck/releases/latest)
- AUR [package](https://aur.archlinux.org/packages/milcheck)

### Usage

By default, milcheck prints the mirrorlist status

```
milcheck
```

In addition, it can fetch and print the Arch Linux
[latest news](https://archlinux.org/), handy to stay informed:

```
milcheck -n5
```

Example: print both mirrorlist and the latest news

```
milcheck -m -n1
```

### License

Mozilla Public License 2.0

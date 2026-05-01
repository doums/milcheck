[![milcheck](https://img.shields.io/github/actions/workflow/status/doums/milcheck/test.yml?color=0D0D0D&logoColor=BFBFBF&labelColor=404040&logo=github&style=for-the-badge)](https://github.com/doums/milcheck/actions?query=workflow%3ATest)
[![milcheck](https://img.shields.io/aur/version/milcheck?color=0D0D0D&logoColor=BFBFBF&labelColor=404040&logo=arch-linux&style=for-the-badge)](https://aur.archlinux.org/packages/milcheck/)

## milcheck

:tea: _**MI**rror**L**ist **Check**_

![milcheck](https://github.com/doums/milcheck/assets/6359431/939c72ac-72f8-4ade-8155-ec8f66ba3c0b)

A CLI to get the status of the local pacman mirrorlist
and the Arch Linux latest news

### How ?

Milcheck reads `/etc/pacman.d/mirrorlist` and retrieves
the corresponding mirror data from the official
[status page](https://www.archlinux.org/mirrors/status/).

The last news are fetched from the [RSS feed](https://archlinux.org/feeds/news/).

### Why ?

As explained in the
[mirror doc](https://wiki.archlinux.org/title/Mirrors), before
a system upgrade i.e. `sudo pacman -Syu`, you should check that
the mirrors set in your mirrorlist are up-to-date e.g. not out of
sync.

### It's not

..an additional mirror list ranking utility.

### Install

- latest [release](https://github.com/doums/milcheck/releases/latest)
- AUR [package](https://aur.archlinux.org/packages/milcheck)

### Usage

Prints the mirrors status

```
milcheck
```

Print the Arch Linux [latest news](https://archlinux.org/)

```
milcheck -n
```

Print both mirrors and latest news (most recent one)

```
milcheck -m -n1
```

### License

Mozilla Public License 2.0

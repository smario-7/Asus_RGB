# The .deb package: what it installs and what it does not do

**English** | [Polski](PACKAGE.pl.md)

## Contents

| Path on the system | Source in the repo | Role |
|---|---|---|
| `/usr/bin/a-rgb` | musl binary (static) | the `a-rgb` command |
| `/usr/lib/udev/rules.d/99-a-rgb.rules` | `packaging/99-a-rgb.rules` | gives the `plugdev` group write access to `brightness` and `kbd_rgb_mode` |
| `/usr/lib/systemd/user/a-rgb-restore.service` | `packaging/a-rgb-restore.service` | runs `a-rgb restore` once after login; `build-deb.sh` replaces its `ExecStart` with `/usr/bin/a-rgb restore` |
| `/usr/share/doc/a-rgb/{README.md,README.pl.md,copyright}` | `README.md`, `README.pl.md`, `LICENSE` | documentation (English and Polish) and the license (MIT, DEP-5 format) |

Metadata (`control`): package `a-rgb`, architecture `amd64`, dependency `udev`. The binary is static, so the package does not depend on the glibc version.

## Maintainer scripts

| Script | When | What it does |
|---|---|---|
| `postinst` | after install and upgrade (`configure`) | `udevadm control --reload`, `udevadm trigger` for `asus::kbd_backlight` (the rule takes effect immediately, no reboot), `systemctl --global enable a-rgb-restore.service` (the `restore` service for every user) |
| `prerm` | before removal (`remove`) | `systemctl --global disable a-rgb-restore.service` |
| `postrm` | after removal and `purge` | `udevadm control --reload` |

Every command has `|| true`, so installation does not fail in an environment without systemd or udev (e.g. a container).

## What the package does NOT do

These things cannot be done safely from package scripts:

1. **It does not add the user to the `plugdev` group.** Without it, writing to sysfs fails. `postinst` runs as root and does not know which user is installing, so it only prints a reminder. The command that adds you to the group only when needed: `id -nG | grep -qw plugdev || sudo usermod -aG plugdev "$USER"`, then log in again (a group only takes effect in a new session). The first user created by the Ubuntu installer is usually already in `plugdev`; accounts added later probably are not (unverified).
2. **It does not register GNOME shortcuts** (Super+F5/F6/F7, Fn+←/→). Shortcuts are user settings; see the [README](../README.md#keyboard-shortcuts-gnome).
3. **It does not change the xkb key map** for Fn+← on the FX506LHB — that is user configuration, described in the README.
4. **It does not change the color on the login screen or at system start.** The firmware sets it; `restore` only runs after login.

## Verifying after installation

```bash
dpkg -L a-rgb                                   # list of files
a-rgb --version
ls -l /sys/class/leds/asus::kbd_backlight/kbd_rgb_mode   # plugdev group, group write permission
systemctl --user is-enabled a-rgb-restore.service        # after logging in again
```

## Uninstalling

```bash
sudo apt remove a-rgb
```

This removes all the files in the table and disables the service. The `~/.config/a-rgb/last` file (the last setting) stays in the user's directory. The permissions on the `/sys` files set by the rule most likely revert to the defaults only after a reboot (not verified).

## Conflicts

If `asusd` (asusctl) is running on the system, both programs control the same LED. Do not use them together.

# a-rgb

**English** | [Polski](README.pl.md)

A small CLI tool (Rust, standard library only) for controlling the keyboard backlight of the ASUS TUF Gaming F15 FX506LHB on Ubuntu. No daemon, no GUI: it starts, sets the color and exits (about 2 ms).

## Compatibility

| Scope | Status |
|---|---|
| ASUS TUF Gaming F15 FX506LHB, Ubuntu 26.04, kernel 7.0, GNOME 50 / Wayland | **tested** |
| Other TUF laptops with a `kbd_rgb_mode` interface in sysfs | untested; should work if the file from the check below exists |
| Other Ubuntu versions | untested on hardware; the `.deb` package installs and removes correctly in Ubuntu 22.04 and 24.04 containers, and the packaged binary is static (no glibc dependency); building from source needs Rust ≥ 1.85 |
| Desktop environments other than GNOME | the commands work, shortcuts have to be assigned manually |
| ROG laptops with an Aura keyboard (zones / per-key, USB HID) | **not supported** |

Check whether your laptop has the required interface:

```bash
ls /sys/class/leds/asus::kbd_backlight/kbd_rgb_mode
```

If the file does not exist, `a-rgb` reports that the laptop does not support color (brightness may still work). The keyboard has a single color for the whole surface. Do not run `a-rgb` together with `asusd`: both control the same LED.

## Installation

Ready-made packages are on the [Releases](https://github.com/smario-7/Asus_RGB/releases) page. Every Release includes `SHA256SUMS` checksums.

### From the .deb package (Ubuntu / Debian, amd64) — recommended

```bash
sha256sum -c SHA256SUMS --ignore-missing
sudo apt install ./a-rgb_VERSION_amd64.deb
id -nG | grep -qw plugdev || sudo usermod -aG plugdev "$USER"   # log in again after being added to the group
a-rgb get
```

Setting the color without `sudo` requires membership of the `plugdev` group. The first user created by the Ubuntu installer is usually already in it (then the group command does nothing). Accounts added later probably are not (unverified) — they need `usermod` and a new login, because a group only takes effect in a new session.

The package installs the binary, the udev rule and the `restore` service (enabled for all users). What the package does **not** do (the `plugdev` group, GNOME shortcuts, the Fn+← key map) and exactly what it installs: [docs/PACKAGE.md](docs/PACKAGE.md). To uninstall: `sudo apt remove a-rgb`.

### From the tar.gz archive (other distributions)

A static binary with no glibc dependency:

```bash
tar -xzf a-rgb-VERSION-x86_64-linux.tar.gz
cd a-rgb-VERSION-x86_64-linux
sudo ./install.sh              # to uninstall: sudo ./install.sh --uninstall
```

The script installs `a-rgb` in `/usr/local/bin`, the udev rule and the `restore` service; check and complete your `plugdev` membership with the same command as above (on distributions without a `plugdev` group, change the group in the udev rule).

### From source (cargo)

Requirements: `build-essential`, Rust ≥ 1.85 (via `rustup`), and your user in the `plugdev` group.

```bash
cargo install --path .
```

The binary is installed to `~/.cargo/bin/a-rgb`. You then install the udev rule and the `restore` service manually:

#### Access without root (udev rule)

Writing to sysfs requires root, so the rule gives the `plugdev` group write access to `brightness` and `kbd_rgb_mode`:

```bash
sudo install -m 644 packaging/99-a-rgb.rules /etc/udev/rules.d/99-a-rgb.rules
sudo udevadm control --reload
sudo udevadm trigger --action=change --subsystem-match=leds --sysname-match='asus::kbd_backlight'
```

To check: `ls -l /sys/class/leds/asus::kbd_backlight/kbd_rgb_mode` should show the `plugdev` group and group write permission. To undo: remove the rule file and run `sudo udevadm control --reload`.

#### Restoring the color after login

`restore` applies the last setting (one-shot, no background process):

```bash
mkdir -p ~/.config/systemd/user
install -m 644 packaging/a-rgb-restore.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable a-rgb-restore.service
```

The unit calls `~/.cargo/bin/a-rgb restore`. Before login (system start, login screen) the keyboard shows the color set by the firmware.

## Usage

```
a-rgb [COLOR] [MODE] [SPEED]     any order; omitted fields come from the last setting
a-rgb COMMAND [arguments]
```

| Command | Action |
|---|---|
| `a-rgb red`, `a-rgb ff8800 breathe fast` | set color / mode / speed |
| `a-rgb b`, `b 2`, `b +`, `b -` | show / set / change brightness by 1 (0 = off) |
| `a-rgb preset gaming\|work\|night\|off` | ready-made settings |
| `a-rgb off` | turn the backlight off (brightness 0) |
| `a-rgb next-color`, `prev-color` | next / previous color from the palette |
| `a-rgb next-mode` | next mode |
| `a-rgb next-preset`, `prev-preset` | next / previous preset (gaming → work → night) |
| `a-rgb get` | brightness and last setting |
| `a-rgb restore` | restore the last setting |
| `a-rgb -h`, `a-rgb help COMMAND` | help |

Colors: `red orange yellow green cyan blue purple pink white` or hex `RRGGBB` / `#RRGGBB`. Speeds: `slow mid fast`.

### Modes (verified on the FX506LHB)

| Mode | Effect | Speed |
|---|---|---|
| `static` | solid color | – |
| `breathe` | the chosen color breathes | works |
| `cycle` | smooth cycling through all colors (the color is ignored) | works |
| `pulse` | fast breathing of the chosen color | has no effect |

Other mode values accepted by the kernel (3–8, 11) produced a solid color on this laptop, so they are not exposed.

### Presets

| Preset | Look |
|---|---|
| `gaming` | red, breathing, speed mid |
| `work` | white, solid |
| `night` | warm orange `ff9329`, solid, brightness 1 |
| `off` | brightness 0 |

`next-preset` / `prev-preset` switch only the look and do not change brightness.

### Exit codes

`0` success, `1` runtime error (e.g. missing permissions), `2` invalid arguments.

## Keyboard shortcuts (GNOME)

Shortcuts are ordinary custom shortcuts that call `a-rgb`. You can add them under Settings → Keyboard → Shortcuts, or with `gsettings`. Used on the FX506LHB:

| Keys | Command |
|---|---|
| Super+F5 / Super+F6 | `a-rgb prev-color` / `a-rgb next-color` |
| Super+F7 | `a-rgb next-mode` |
| Fn+← (`XF86Launch3`) | `a-rgb prev-preset` |
| Fn+→ (`XF86Launch4`) | `a-rgb next-preset` |

Use the full path in the shortcut commands (`/usr/bin/a-rgb` for the package, or `/home/USER/.cargo/bin/a-rgb` for a cargo install), because the GNOME session may not have `~/.cargo/bin` in its `PATH`.

Fn+↑/↓ is handled by GNOME (brightness) and `a-rgb` does not replace it. Brightness is remembered by systemd (`systemd-backlight`), which is why `restore` does not touch it. The "Keyboard" slider in the quick settings menu is also a GNOME element that works on the same `brightness` file.

### Fn+← on the FX506LHB

This step is optional and only needed if you want to use Fn+← as a shortcut. Fn+→ works without it.

**What is established** (checked by listening to input events on the FX506LHB): Fn+← sends scancode `0xb2`, which the kernel reports as `KEY_UNKNOWN` (240), and Fn+→ sends `0xb3`, reported as `KEY_PROG4` (`XF86Launch4`). The kernel does not let you remap `0xb2` through hwdb (`EINVAL`), so the only way is an xkb map to the `XF86Launch3` symbol.

**What Fn+← does on your system depends on your configuration.** On a clean installation I expect it to do nothing, because GNOME does not know this code (I did not check this). On the test machine, whose system had been moved over from another laptop, the key acted like Super (it opened the workspace overview) because a custom xkb option in `~/.config/xkb` mapped the same code (`<I248>`) to `Super_L`. I do not know where that option came from.

**First check whether something already maps this key**, so you do not add a second option for the same code:

```bash
gsettings get org.gnome.desktop.input-sources xkb-options
ls ~/.config/xkb
```

If you already have a custom option for `<I248>`, change its symbol instead of adding a new one. Do not do this if you use Fn+← for something else (e.g. as a substitute Super key). The option name in the example below comes from the test machine and can be anything.

`~/.config/xkb/symbols/custom`:

```
partial
xkb_symbols "fix_super" {
    key <I248> { [ XF86Launch3 ] };
};
```

`~/.config/xkb/rules/evdev`:

```
! option = symbols
  custom:fix_super = +custom(fix_super)

! include %S/evdev
```

Then add the option in GNOME and log in again (GNOME reads the xkb files at session start):

```bash
gsettings set org.gnome.desktop.input-sources xkb-options "['custom:fix_super']"
```

If you already have other xkb options, append this one to the existing list instead of replacing it (`gsettings get org.gnome.desktop.input-sources xkb-options`).

## How it works

`a-rgb` writes the line `cmd mode r g b speed` to `/sys/class/leds/asus::kbd_backlight/kbd_rgb_mode`. It always uses `cmd=0` (set) and never `cmd=1` (save to BIOS), so it does not write to non-volatile memory. The kernel does not validate the r, g, b values, so `a-rgb` enforces the 0–255 range. The file is write-only: the current color cannot be read back from the hardware, so `get` and `restore` use the last setting stored in `~/.config/a-rgb/last`.

The `A_RGB_SYSFS` variable overrides the LED directory (testing without hardware), and `XDG_CONFIG_HOME` overrides the configuration directory.

## Known limitations

- The color on the login screen and at system start is set by the firmware. Changing it would require saving to the BIOS (`cmd=1`) or a system service, and it is not implemented.
- The boot / awake / sleep backlight states (`kbd_rgb_state`) are not supported.
- The program output (`a-rgb -h`, messages) is in English only; the Polish documentation is in [README.pl.md](README.pl.md).

## Development

```bash
cargo test
cargo clippy --all-targets
cargo build --release
```

Unit tests do not need hardware. To test on the device: set a color and confirm it visually, because `kbd_rgb_mode` is write-only. Building packages and releasing versions: [docs/RELEASING.md](docs/RELEASING.md).

## Uninstalling

```bash
systemctl --user disable --now a-rgb-restore.service
rm ~/.config/systemd/user/a-rgb-restore.service ~/.cargo/bin/a-rgb
rm -r ~/.config/a-rgb
sudo rm /etc/udev/rules.d/99-a-rgb.rules && sudo udevadm control --reload
```

Remove the GNOME shortcuts under Settings → Keyboard → Shortcuts (the "a-rgb: …" entries).

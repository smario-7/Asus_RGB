//! Domain types: color, mode, speed and their serialization.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b }
}

/// Named colors; the order is the palette for `next-color` / `prev-color`.
pub const NAMED: [(&str, Color); 9] = [
    ("red", rgb(255, 0, 0)),
    ("orange", rgb(255, 100, 0)),
    ("yellow", rgb(255, 200, 0)),
    ("green", rgb(0, 255, 0)),
    ("cyan", rgb(0, 255, 255)),
    ("blue", rgb(0, 0, 255)),
    ("purple", rgb(140, 0, 255)),
    ("pink", rgb(255, 0, 140)),
    ("white", rgb(255, 255, 255)),
];

impl Color {
    /// Nazwa z `NAMED` albo hex `RRGGBB` / `#RRGGBB`.
    pub fn parse(s: &str) -> Result<Color, String> {
        let lower = s.to_ascii_lowercase();
        if let Some((_, c)) = NAMED.iter().find(|(n, _)| *n == lower) {
            return Ok(*c);
        }
        let hex = lower.strip_prefix('#').unwrap_or(&lower);
        if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!(
                "unknown argument '{s}' (expected a color: a name or hex RRGGBB, a mode or a speed)"
            ));
        }
        let v = u32::from_str_radix(hex, 16).map_err(|e| e.to_string())?;
        Ok(rgb((v >> 16) as u8, (v >> 8) as u8, v as u8))
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// Modes verified on the FX506LHB; the value is the `mode` code in `kbd_rgb_mode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Static,
    Breathe,
    Cycle,
    Pulse,
}

impl Mode {
    pub const ALL: [Mode; 4] = [Mode::Static, Mode::Breathe, Mode::Cycle, Mode::Pulse];

    pub fn code(self) -> u8 {
        match self {
            Mode::Static => 0,
            Mode::Breathe => 1,
            Mode::Cycle => 2,
            Mode::Pulse => 10,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Mode::Static => "static",
            Mode::Breathe => "breathe",
            Mode::Cycle => "cycle",
            Mode::Pulse => "pulse",
        }
    }

    pub fn parse(s: &str) -> Option<Mode> {
        let lower = s.to_ascii_lowercase();
        Mode::ALL.into_iter().find(|m| m.name() == lower)
    }

    pub fn next(self) -> Mode {
        let i = Mode::ALL.iter().position(|m| *m == self).unwrap_or(0);
        Mode::ALL[(i + 1) % Mode::ALL.len()]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Speed {
    Slow,
    Mid,
    Fast,
}

impl Speed {
    pub const ALL: [Speed; 3] = [Speed::Slow, Speed::Mid, Speed::Fast];

    pub fn code(self) -> u8 {
        match self {
            Speed::Slow => 0,
            Speed::Mid => 1,
            Speed::Fast => 2,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Speed::Slow => "slow",
            Speed::Mid => "mid",
            Speed::Fast => "fast",
        }
    }

    pub fn parse(s: &str) -> Option<Speed> {
        let lower = s.to_ascii_lowercase();
        Speed::ALL.into_iter().find(|v| v.name() == lower)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub color: Color,
    pub mode: Mode,
    pub speed: Speed,
}

impl Settings {
    /// Initial state when nothing has been saved yet (the current color cannot be read back from the hardware).
    pub const DEFAULT: Settings = Settings {
        color: rgb(255, 255, 255),
        mode: Mode::Static,
        speed: Speed::Mid,
    };

    /// Line for `kbd_rgb_mode`: `cmd mode r g b speed`; cmd=0 (set, no save to the BIOS).
    pub fn sysfs_line(&self) -> String {
        format!(
            "0 {} {} {} {} {}",
            self.mode.code(),
            self.color.r,
            self.color.g,
            self.color.b,
            self.speed.code()
        )
    }

    pub fn serialize(&self) -> String {
        format!(
            "mode={}\ncolor={}\nspeed={}\n",
            self.mode.name(),
            self.color,
            self.speed.name()
        )
    }

    pub fn deserialize(text: &str) -> Result<Settings, String> {
        let (mut mode, mut color, mut speed) = (None, None, None);
        for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
            let (k, v) = line
                .split_once('=')
                .ok_or_else(|| format!("bad state line: '{line}'"))?;
            match k {
                "mode" => mode = Some(Mode::parse(v).ok_or_else(|| format!("bad mode: '{v}'"))?),
                "color" => color = Some(Color::parse(v)?),
                "speed" => {
                    speed = Some(Speed::parse(v).ok_or_else(|| format!("bad speed: '{v}'"))?)
                }
                _ => {}
            }
        }
        Ok(Settings {
            mode: mode.ok_or("missing key: mode")?,
            color: color.ok_or("missing key: color")?,
            speed: speed.ok_or("missing key: speed")?,
        })
    }

    /// The next (`step` > 0) or previous (`step` < 0) color in the `NAMED` palette.
    pub fn stepped_color(&self, step: isize) -> Color {
        let n = NAMED.len() as isize;
        let pos = NAMED.iter().position(|(_, c)| *c == self.color);
        let next = match pos {
            Some(p) => (p as isize + step).rem_euclid(n),
            None if step > 0 => 0,
            None => n - 1,
        };
        NAMED[next as usize].1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_named_and_hex_colors() {
        assert_eq!(Color::parse("Red").unwrap(), rgb(255, 0, 0));
        assert_eq!(Color::parse("ff8800").unwrap(), rgb(255, 136, 0));
        assert_eq!(Color::parse("#00FF7f").unwrap(), rgb(0, 255, 127));
    }

    #[test]
    fn rejects_bad_colors() {
        for bad in ["", "ff88", "gg0000", "ff88001", "#12345", "reddish"] {
            assert!(Color::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn sysfs_line_matches_kernel_format() {
        let s = Settings {
            color: rgb(1, 2, 3),
            mode: Mode::Pulse,
            speed: Speed::Fast,
        };
        assert_eq!(s.sysfs_line(), "0 10 1 2 3 2");
    }

    #[test]
    fn state_roundtrip() {
        for mode in Mode::ALL {
            for speed in Speed::ALL {
                let s = Settings {
                    color: rgb(18, 52, 86),
                    mode,
                    speed,
                };
                assert_eq!(Settings::deserialize(&s.serialize()).unwrap(), s);
            }
        }
    }

    #[test]
    fn state_rejects_incomplete_or_garbage() {
        assert!(Settings::deserialize("mode=static\ncolor=ff0000\n").is_err());
        assert!(Settings::deserialize("garbage").is_err());
        assert!(Settings::deserialize("mode=x\ncolor=ff0000\nspeed=mid\n").is_err());
    }

    #[test]
    fn color_stepping_wraps_and_handles_custom() {
        let mut s = Settings::DEFAULT; // white = ostatni w palecie
        assert_eq!(s.stepped_color(1), NAMED[0].1);
        s.color = NAMED[0].1;
        assert_eq!(s.stepped_color(-1), NAMED[8].1);
        s.color = rgb(1, 2, 3); // spoza palety
        assert_eq!(s.stepped_color(1), NAMED[0].1);
        assert_eq!(s.stepped_color(-1), NAMED[8].1);
    }

    #[test]
    fn mode_cycle_covers_all_and_wraps() {
        assert_eq!(Mode::Pulse.next(), Mode::Static);
        assert_eq!(Mode::Static.next(), Mode::Breathe);
    }
}

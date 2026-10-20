//! Argument parsing and help texts.

use crate::model::{Color, Mode, NAMED, Settings, Speed};

#[derive(Debug, PartialEq, Eq)]
pub enum Brightness {
    Show,
    Set(u8),
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    Gaming,
    Work,
    Night,
    Off,
}

pub const PRESETS: [(Preset, &str, &str); 4] = [
    (
        Preset::Gaming,
        "gaming",
        "red, breathing (breathe), speed mid",
    ),
    (Preset::Work, "work", "white, solid (static)"),
    (
        Preset::Night,
        "night",
        "warm orange ff9329, solid, brightness 1",
    ),
    (Preset::Off, "off", "turns the backlight off (brightness 0)"),
];

/// Presets cycled by `next-preset` / `prev-preset` (looks only, brightness is untouched).
pub const CYCLE: [Preset; 3] = [Preset::Gaming, Preset::Work, Preset::Night];

impl Preset {
    /// The look to apply (if the preset changes it) and the target brightness (if it changes that).
    pub fn spec(self) -> (Option<Settings>, Option<u8>) {
        let look = |color: Color, mode: Mode| Settings {
            color,
            mode,
            speed: Speed::Mid,
        };
        match self {
            Preset::Gaming => (Some(look(NAMED[0].1, Mode::Breathe)), None),
            Preset::Work => (Some(look(NAMED[8].1, Mode::Static)), None),
            Preset::Night => (
                Some(look(
                    Color {
                        r: 0xff,
                        g: 0x93,
                        b: 0x29,
                    },
                    Mode::Static,
                )),
                Some(1),
            ),
            Preset::Off => (None, Some(0)),
        }
    }
}

/// The look of the next (`step` > 0) or previous (`step` < 0) preset in `CYCLE`;
/// from an unknown setting it starts at the first (forward) or the last (backward).
pub fn cycle_preset(current: Option<Settings>, step: isize) -> Settings {
    let n = CYCLE.len() as isize;
    let pos = current.and_then(|s| CYCLE.iter().position(|p| p.spec().0 == Some(s)));
    let i = match pos {
        Some(p) => (p as isize + step).rem_euclid(n),
        None if step > 0 => 0,
        None => n - 1,
    };
    CYCLE[i as usize].spec().0.unwrap_or(Settings::DEFAULT)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Appearance {
        color: Option<Color>,
        mode: Option<Mode>,
        speed: Option<Speed>,
    },
    Brightness(Brightness),
    Preset(Preset),
    NextColor,
    PrevColor,
    NextMode,
    NextPreset,
    PrevPreset,
    Get,
    Restore,
    Help(Option<String>),
    Version,
}

pub fn parse(args: &[String]) -> Result<Command, String> {
    let Some(first) = args.first() else {
        return Ok(Command::Help(None));
    };
    let rest = &args[1..];
    let no_args = |cmd: Command| {
        if rest.is_empty() {
            Ok(cmd)
        } else {
            Err(format!("'{first}' takes no arguments"))
        }
    };
    match first.to_ascii_lowercase().as_str() {
        "-h" | "--help" | "help" => Ok(Command::Help(rest.first().cloned())),
        "-v" | "-V" | "--version" | "version" => no_args(Command::Version),
        "b" | "brightness" => parse_brightness(rest),
        "preset" => parse_preset(rest),
        "off" => no_args(Command::Preset(Preset::Off)),
        "next-color" => no_args(Command::NextColor),
        "prev-color" => no_args(Command::PrevColor),
        "next-mode" => no_args(Command::NextMode),
        "next-preset" => no_args(Command::NextPreset),
        "prev-preset" => no_args(Command::PrevPreset),
        "get" => no_args(Command::Get),
        "restore" => no_args(Command::Restore),
        _ => parse_appearance(args),
    }
}

fn parse_brightness(rest: &[String]) -> Result<Command, String> {
    let b = match rest {
        [] => Brightness::Show,
        [a] if a == "+" => Brightness::Up,
        [a] if a == "-" => Brightness::Down,
        [a] => Brightness::Set(
            a.parse()
                .map_err(|_| format!("brightness: expected a number, '+' or '-', got '{a}'"))?,
        ),
        _ => return Err("b takes one argument: a number, '+' or '-'".into()),
    };
    Ok(Command::Brightness(b))
}

fn parse_preset(rest: &[String]) -> Result<Command, String> {
    let [name] = rest else {
        return Err(format!("preset needs a name: {}", preset_names()));
    };
    PRESETS
        .iter()
        .find(|(_, n, _)| n.eq_ignore_ascii_case(name))
        .map(|(p, _, _)| Command::Preset(*p))
        .ok_or_else(|| format!("unknown preset '{name}' (available: {})", preset_names()))
}

fn preset_names() -> String {
    PRESETS
        .iter()
        .map(|(_, n, _)| *n)
        .collect::<Vec<_>>()
        .join(", ")
}

/// `[color] [mode] [speed]` in any order; missing fields are taken from the saved state.
fn parse_appearance(args: &[String]) -> Result<Command, String> {
    let (mut color, mut mode, mut speed) = (None, None, None);
    for a in args {
        if let Some(m) = Mode::parse(a) {
            if mode.replace(m).is_some() {
                return Err("two modes given".into());
            }
        } else if let Some(s) = Speed::parse(a) {
            if speed.replace(s).is_some() {
                return Err("two speeds given".into());
            }
        } else if color.replace(Color::parse(a)?).is_some() {
            return Err("two colors given".into());
        }
    }
    Ok(Command::Appearance { color, mode, speed })
}

struct CmdDoc {
    name: &'static str,
    usage: &'static str,
    short: &'static str,
    long: &'static str,
}

const COMMANDS: &[CmdDoc] = &[
    CmdDoc {
        name: "color",
        usage: "a-rgb [COLOR] [MODE] [SPEED]",
        short: "set color, mode and speed (any order)",
        long: "Omitted fields keep the values of the last setting (a-rgb get);\n\
               with no saved setting the start is white, static, mid.\n\
               COLOR  name (red orange yellow green cyan blue purple pink white) or hex RRGGBB / #RRGGBB\n\
               MODE   static | breathe | cycle | pulse\n\
               SPEED  slow | mid | fast\n\
               Notes: cycle changes colors by itself (COLOR is ignored); pulse is fast breathing,\n\
               SPEED has no effect on it. Speed works in breathe and cycle.\n\
               Examples: a-rgb red | a-rgb ff8800 breathe fast | a-rgb cycle slow | a-rgb static",
    },
    CmdDoc {
        name: "b",
        usage: "a-rgb b [0..MAX | + | -]",
        short: "keyboard brightness (show / set / raise / lower)",
        long: "With no argument it shows the brightness. A number sets it (0 = off, MAX is usually 3),\n\
               '+' and '-' change it by 1 within 0..MAX. It uses the same setting as Fn+↑/↓,\n\
               and the system remembers brightness (systemd-backlight). Alias: brightness.",
    },
    CmdDoc {
        name: "preset",
        usage: "a-rgb preset NAME",
        short: "ready-made settings: gaming, work, night, off",
        long: "gaming  red, breathing, speed mid\n\
               work    white, solid\n\
               night   warm orange ff9329, solid, brightness 1\n\
               off     turns the backlight off (brightness 0); also: a-rgb off",
    },
    CmdDoc {
        name: "off",
        usage: "a-rgb off",
        short: "turn the backlight off (brightness 0)",
        long: "Shortcut for 'preset off'. To turn it back on: a-rgb b 1 (or Fn+↑). The color and mode stay remembered.",
    },
    CmdDoc {
        name: "next-color",
        usage: "a-rgb next-color",
        short: "next color in the palette (for a keyboard shortcut)",
        long: "Palette: red orange yellow green cyan blue purple pink white (then back to the start).\n\
               Mode and speed are unchanged. A color outside the palette starts from the beginning of the palette.",
    },
    CmdDoc {
        name: "prev-color",
        usage: "a-rgb prev-color",
        short: "previous color in the palette (for a keyboard shortcut)",
        long: "The reverse of next-color.",
    },
    CmdDoc {
        name: "next-mode",
        usage: "a-rgb next-mode",
        short: "next mode: static → breathe → cycle → pulse (for a keyboard shortcut)",
        long: "Color and speed are unchanged. After pulse it wraps around to static.",
    },
    CmdDoc {
        name: "next-preset",
        usage: "a-rgb next-preset",
        short: "next ready-made look: gaming → work → night (for a keyboard shortcut)",
        long: "Switches only the looks (color, mode, speed); brightness is unchanged.\n\
               If the current setting is not one of the presets, it starts from gaming.",
    },
    CmdDoc {
        name: "prev-preset",
        usage: "a-rgb prev-preset",
        short: "previous ready-made look: night → work → gaming (for a keyboard shortcut)",
        long: "The reverse of next-preset. From an unknown setting it starts from night.",
    },
    CmdDoc {
        name: "get",
        usage: "a-rgb get",
        short: "show the brightness and the last saved setting",
        long: "The current color cannot be read from the hardware — what is shown is the last setting\n\
               made by a-rgb (file ~/.config/a-rgb/last).",
    },
    CmdDoc {
        name: "restore",
        usage: "a-rgb restore",
        short: "restore the last setting (for autostart)",
        long: "Applies the saved color, mode and speed. With no saved setting it does nothing (exit code 0).\n\
               It does not touch brightness — systemd-backlight restores that.",
    },
    CmdDoc {
        name: "help",
        usage: "a-rgb help [COMMAND]",
        short: "general help, or details for a command (also -h, --help)",
        long: "Example: a-rgb help preset",
    },
];

pub fn help_text() -> String {
    let mut s = String::from(
        "a-rgb — keyboard backlight control for the ASUS TUF Gaming F15 (FX506LHB)\n\n\
         USAGE\n  a-rgb [COLOR] [MODE] [SPEED]\n  a-rgb COMMAND [arguments]\n\n\
         COMMANDS\n",
    );
    for c in COMMANDS {
        s.push_str(&format!("  {:<11} {}\n", c.name, c.short));
    }
    s.push_str("\nDetails: a-rgb help COMMAND\n\nCOLORS\n  ");
    s.push_str(&NAMED.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(" "));
    s.push_str(" or hex RRGGBB (e.g. ff8800, #00ff7f)\n\nMODES\n");
    s.push_str(
        "  static   solid color\n  breathe  the chosen color breathes (speed applies)\n  \
         cycle    smooth change through all colors (speed applies, color ignored)\n  \
         pulse    fast breathing of the chosen color (speed has no effect)\n\n\
         SPEED\n  slow mid fast\n\nPRESETS\n",
    );
    for (_, n, d) in PRESETS {
        s.push_str(&format!("  {n:<8} {d}\n"));
    }
    s.push_str(
        "\nEXAMPLES\n  a-rgb red\n  a-rgb ff8800 breathe fast\n  a-rgb cycle slow\n  a-rgb b +\n  \
         a-rgb preset night\n\nEXIT CODES\n  0 success   1 runtime error (e.g. missing permissions)   2 invalid arguments\n\n\
         FILES\n  ~/.config/a-rgb/last   last setting (used by restore, next-*)\n  \
         /sys/class/leds/asus::kbd_backlight   the LED being controlled (writing requires the udev rule)\n",
    );
    s
}

pub fn command_help(name: &str) -> Result<String, String> {
    let key = if name.eq_ignore_ascii_case("brightness") {
        "b"
    } else {
        name
    };
    COMMANDS
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(key))
        .map(|c| format!("{}\n\n{}\n\n{}\n", c.usage, c.short, c.long))
        .ok_or_else(|| format!("no help for '{name}' (list: a-rgb -h)"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(a: &[&str]) -> Result<Command, String> {
        parse(&a.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn appearance_any_order_and_partial() {
        let want = Command::Appearance {
            color: Some(Color::parse("ff8800").unwrap()),
            mode: Some(Mode::Breathe),
            speed: Some(Speed::Fast),
        };
        assert_eq!(p(&["ff8800", "breathe", "fast"]).unwrap(), want);
        assert_eq!(p(&["fast", "BREATHE", "#ff8800"]).unwrap(), want);
        assert_eq!(
            p(&["cycle"]).unwrap(),
            Command::Appearance {
                color: None,
                mode: Some(Mode::Cycle),
                speed: None
            }
        );
    }

    #[test]
    fn appearance_rejects_duplicates_and_garbage() {
        assert!(p(&["red", "blue"]).is_err());
        assert!(p(&["static", "pulse"]).is_err());
        assert!(p(&["slow", "fast"]).is_err());
        assert!(p(&["nonsense"]).is_err());
    }

    #[test]
    fn brightness_forms() {
        assert_eq!(p(&["b"]).unwrap(), Command::Brightness(Brightness::Show));
        assert_eq!(p(&["b", "+"]).unwrap(), Command::Brightness(Brightness::Up));
        assert_eq!(
            p(&["b", "-"]).unwrap(),
            Command::Brightness(Brightness::Down)
        );
        assert_eq!(
            p(&["brightness", "2"]).unwrap(),
            Command::Brightness(Brightness::Set(2))
        );
        assert!(p(&["b", "x"]).is_err());
        assert!(p(&["b", "1", "2"]).is_err());
        assert!(p(&["b", "-1"]).is_err());
    }

    #[test]
    fn presets_and_simple_commands() {
        assert_eq!(
            p(&["preset", "gaming"]).unwrap(),
            Command::Preset(Preset::Gaming)
        );
        assert_eq!(p(&["off"]).unwrap(), Command::Preset(Preset::Off));
        assert!(p(&["preset"]).is_err());
        assert!(p(&["preset", "nope"]).is_err());
        assert!(p(&["off", "x"]).is_err());
        assert_eq!(p(&[]).unwrap(), Command::Help(None));
        assert_eq!(p(&["-h"]).unwrap(), Command::Help(None));
        assert_eq!(
            p(&["help", "preset"]).unwrap(),
            Command::Help(Some("preset".into()))
        );
    }

    #[test]
    fn preset_cycling_wraps_both_ways() {
        let look = |p: Preset| p.spec().0.unwrap();
        let (g, w, n) = (
            look(Preset::Gaming),
            look(Preset::Work),
            look(Preset::Night),
        );
        assert_eq!(cycle_preset(Some(g), 1), w);
        assert_eq!(cycle_preset(Some(w), 1), n);
        assert_eq!(cycle_preset(Some(n), 1), g);
        assert_eq!(cycle_preset(Some(g), -1), n);
        assert_eq!(cycle_preset(Some(n), -1), w);
        assert_eq!(cycle_preset(None, 1), g);
        assert_eq!(cycle_preset(None, -1), n);
        assert_eq!(cycle_preset(Some(Settings::DEFAULT), 1), n); // DEFAULT = the 'work' look
        let custom = Settings {
            color: Color::parse("0000ff").unwrap(),
            ..w
        };
        assert_eq!(cycle_preset(Some(custom), 1), g);
        assert_eq!(cycle_preset(Some(custom), -1), n);
        assert_eq!(p(&["next-preset"]).unwrap(), Command::NextPreset);
        assert_eq!(p(&["prev-preset"]).unwrap(), Command::PrevPreset);
        assert!(p(&["next-preset", "x"]).is_err());
    }

    #[test]
    fn every_command_is_documented_and_listed_in_help() {
        let help = help_text();
        for c in COMMANDS {
            assert!(
                !c.usage.is_empty() && !c.short.is_empty() && !c.long.is_empty(),
                "{}",
                c.name
            );
            assert!(
                help.contains(c.name) && help.contains(c.short),
                "{}",
                c.name
            );
            assert!(command_help(c.name).is_ok());
        }
        for (_, n, _) in PRESETS {
            assert!(help.contains(n), "{n}");
        }
        for m in Mode::ALL {
            assert!(help.contains(m.name()), "{}", m.name());
        }
        for (n, _) in NAMED {
            assert!(help.contains(n), "{n}");
        }
        assert!(command_help("brightness").is_ok());
        assert!(command_help("zzz").is_err());
    }
}

//! Parsowanie argumentów i teksty pomocy.

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
        "czerwony, oddychanie (breathe), prędkość mid",
    ),
    (Preset::Work, "work", "biały, stały (static)"),
    (
        Preset::Night,
        "night",
        "ciepły pomarańczowy ff9329, stały, jasność 1",
    ),
    (Preset::Off, "off", "wyłącza podświetlenie (jasność 0)"),
];

/// Presety przełączane przez `next-preset` / `prev-preset` (same wyglądy, bez zmiany jasności).
pub const CYCLE: [Preset; 3] = [Preset::Gaming, Preset::Work, Preset::Night];

impl Preset {
    /// Wygląd do ustawienia (jeśli preset go zmienia) i docelowa jasność (jeśli ją zmienia).
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

/// Wygląd następnego (`step` > 0) lub poprzedniego (`step` < 0) presetu z `CYCLE`;
/// z nieznanego ustawienia zaczyna od pierwszego (do przodu) lub ostatniego (do tyłu).
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
            Err(format!("'{first}' nie przyjmuje argumentów"))
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
                .map_err(|_| format!("jasność: oczekiwano liczby, '+' lub '-', podano '{a}'"))?,
        ),
        _ => return Err("b przyjmuje jeden argument: liczbę, '+' lub '-'".into()),
    };
    Ok(Command::Brightness(b))
}

fn parse_preset(rest: &[String]) -> Result<Command, String> {
    let [name] = rest else {
        return Err(format!("preset wymaga nazwy: {}", preset_names()));
    };
    PRESETS
        .iter()
        .find(|(_, n, _)| n.eq_ignore_ascii_case(name))
        .map(|(p, _, _)| Command::Preset(*p))
        .ok_or_else(|| format!("nieznany preset '{name}' (dostępne: {})", preset_names()))
}

fn preset_names() -> String {
    PRESETS
        .iter()
        .map(|(_, n, _)| *n)
        .collect::<Vec<_>>()
        .join(", ")
}

/// `[kolor] [tryb] [prędkość]` w dowolnej kolejności; brakujące pola bierzemy z zapisanego stanu.
fn parse_appearance(args: &[String]) -> Result<Command, String> {
    let (mut color, mut mode, mut speed) = (None, None, None);
    for a in args {
        if let Some(m) = Mode::parse(a) {
            if mode.replace(m).is_some() {
                return Err("podano dwa tryby".into());
            }
        } else if let Some(s) = Speed::parse(a) {
            if speed.replace(s).is_some() {
                return Err("podano dwie prędkości".into());
            }
        } else if color.replace(Color::parse(a)?).is_some() {
            return Err("podano dwa kolory".into());
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
        name: "kolor",
        usage: "a-rgb [KOLOR] [TRYB] [PRĘDKOŚĆ]",
        short: "ustaw kolor, tryb i prędkość (kolejność dowolna)",
        long: "Pominięte pola przejmują wartości z ostatniego ustawienia (a-rgb get);\n\
               bez zapisanego stanu start to biały, static, mid.\n\
               KOLOR  nazwa (red orange yellow green cyan blue purple pink white) lub hex RRGGBB / #RRGGBB\n\
               TRYB   static | breathe | cycle | pulse\n\
               PRĘDKOŚĆ  slow | mid | fast\n\
               Uwagi: cycle sam zmienia kolory (KOLOR jest ignorowany); pulse to szybkie oddychanie,\n\
               PRĘDKOŚĆ nie ma na nie wpływu. Prędkość działa w breathe i cycle.\n\
               Przykłady: a-rgb red | a-rgb ff8800 breathe fast | a-rgb cycle slow | a-rgb static",
    },
    CmdDoc {
        name: "b",
        usage: "a-rgb b [0..MAX | + | -]",
        short: "jasność klawiatury (pokaż / ustaw / zwiększ / zmniejsz)",
        long: "Bez argumentu pokazuje jasność. Liczba ustawia ją (0 = wyłączone, MAX zwykle 3),\n\
               '+' i '-' zmieniają o 1 w granicach 0..MAX. Używa tego samego ustawienia co Fn+↑/↓,\n\
               jasność zapamiętuje system (systemd-backlight). Alias: brightness.",
    },
    CmdDoc {
        name: "preset",
        usage: "a-rgb preset NAZWA",
        short: "gotowe ustawienia: gaming, work, night, off",
        long: "gaming  czerwony, oddychanie, prędkość mid\n\
               work    biały, stały\n\
               night   ciepły pomarańczowy ff9329, stały, jasność 1\n\
               off     wyłącza podświetlenie (jasność 0); też: a-rgb off",
    },
    CmdDoc {
        name: "off",
        usage: "a-rgb off",
        short: "wyłącz podświetlenie (jasność 0)",
        long: "Skrót do 'preset off'. Włączenie: a-rgb b 1 (albo Fn+↑). Kolor i tryb pozostają zapamiętane.",
    },
    CmdDoc {
        name: "next-color",
        usage: "a-rgb next-color",
        short: "następny kolor z palety (skrót klawiszowy)",
        long: "Paleta: red orange yellow green cyan blue purple pink white (potem od początku).\n\
               Tryb i prędkość bez zmian. Kolor spoza palety zaczyna od początku palety.",
    },
    CmdDoc {
        name: "prev-color",
        usage: "a-rgb prev-color",
        short: "poprzedni kolor z palety (skrót klawiszowy)",
        long: "Odwrotnie niż next-color.",
    },
    CmdDoc {
        name: "next-mode",
        usage: "a-rgb next-mode",
        short: "następny tryb: static → breathe → cycle → pulse (skrót klawiszowy)",
        long: "Kolor i prędkość bez zmian. Po pulse wraca do static.",
    },
    CmdDoc {
        name: "next-preset",
        usage: "a-rgb next-preset",
        short: "następny gotowy tryb: gaming → work → night (skrót klawiszowy)",
        long: "Przełącza same wyglądy (kolor, tryb, prędkość); jasności nie zmienia.\n\
               Jeśli aktualne ustawienie nie jest żadnym z presetów, zaczyna od gaming.",
    },
    CmdDoc {
        name: "prev-preset",
        usage: "a-rgb prev-preset",
        short: "poprzedni gotowy tryb: night → work → gaming (skrót klawiszowy)",
        long: "Odwrotnie niż next-preset. Z nieznanego ustawienia zaczyna od night.",
    },
    CmdDoc {
        name: "get",
        usage: "a-rgb get",
        short: "pokaż jasność i ostatnio zapisane ustawienie",
        long: "Aktualnego koloru nie da się odczytać ze sprzętu — pokazywane jest ostatnie ustawienie\n\
               wykonane przez a-rgb (plik ~/.config/a-rgb/last).",
    },
    CmdDoc {
        name: "restore",
        usage: "a-rgb restore",
        short: "przywróć ostatnie ustawienie (do autostartu)",
        long: "Stosuje zapisany kolor, tryb i prędkość. Bez zapisanego stanu nic nie robi (kod 0).\n\
               Jasności nie rusza — przywraca ją systemd-backlight.",
    },
    CmdDoc {
        name: "help",
        usage: "a-rgb help [POLECENIE]",
        short: "pomoc ogólna lub szczegółowa dla polecenia (też -h, --help)",
        long: "Przykład: a-rgb help preset",
    },
];

pub fn help_text() -> String {
    let mut s = String::from(
        "a-rgb — sterowanie podświetleniem klawiatury ASUS TUF Gaming F15 (FX506LHB)\n\n\
         UŻYCIE\n  a-rgb [KOLOR] [TRYB] [PRĘDKOŚĆ]\n  a-rgb POLECENIE [argumenty]\n\n\
         POLECENIA\n",
    );
    for c in COMMANDS {
        s.push_str(&format!("  {:<11} {}\n", c.name, c.short));
    }
    s.push_str("\nSzczegóły: a-rgb help POLECENIE\n\nKOLORY\n  ");
    s.push_str(&NAMED.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(" "));
    s.push_str(" lub hex RRGGBB (np. ff8800, #00ff7f)\n\nTRYBY\n");
    s.push_str(
        "  static   stały kolor\n  breathe  oddychanie wybranego koloru (działa prędkość)\n  \
         cycle    płynna zmiana wszystkich kolorów (działa prędkość, kolor ignorowany)\n  \
         pulse    szybkie oddychanie wybranego koloru (prędkość bez wpływu)\n\n\
         PRĘDKOŚĆ\n  slow mid fast\n\nPRESETY\n",
    );
    for (_, n, d) in PRESETS {
        s.push_str(&format!("  {n:<8} {d}\n"));
    }
    s.push_str(
        "\nPRZYKŁADY\n  a-rgb red\n  a-rgb ff8800 breathe fast\n  a-rgb cycle slow\n  a-rgb b +\n  \
         a-rgb preset night\n\nKODY WYJŚCIA\n  0 sukces   1 błąd wykonania (np. brak uprawnień)   2 błędne argumenty\n\n\
         PLIKI\n  ~/.config/a-rgb/last   ostatnie ustawienie (używane przez restore, next-*)\n  \
         /sys/class/leds/asus::kbd_backlight   sterowany LED (zapis wymaga reguły udev)\n",
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
        .ok_or_else(|| format!("brak pomocy dla '{name}' (lista: a-rgb -h)"))
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
        assert_eq!(cycle_preset(Some(Settings::DEFAULT), 1), n); // DEFAULT = wygląd 'work'
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

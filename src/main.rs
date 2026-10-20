mod cli;
mod model;
mod state;
mod sysfs;

use cli::{Brightness, Command};
use model::Settings;
use std::process::ExitCode;
use sysfs::Led;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = match cli::parse(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("a-rgb: {e}\nPomoc: a-rgb -h");
            return ExitCode::from(2);
        }
    };
    match run(cmd) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("a-rgb: {e}");
            ExitCode::from(1)
        }
    }
}

fn run(cmd: Command) -> Result<(), String> {
    match cmd {
        Command::Help(None) => print!("{}", cli::help_text()),
        Command::Help(Some(name)) => print!("{}", cli::command_help(&name)?),
        Command::Version => println!("a-rgb {}", env!("CARGO_PKG_VERSION")),
        Command::Brightness(b) => brightness(&Led::open()?, b)?,
        Command::Preset(p) => {
            let led = Led::open()?;
            let (look, level) = p.spec();
            if let Some(s) = look {
                apply(&led, &s)?;
            }
            if let Some(v) = level {
                led.set_brightness(v.min(led.max_brightness()?))?;
            }
        }
        Command::Appearance { color, mode, speed } => {
            let led = Led::open()?;
            let base = state::load().unwrap_or(Settings::DEFAULT);
            apply(
                &led,
                &Settings {
                    color: color.unwrap_or(base.color),
                    mode: mode.unwrap_or(base.mode),
                    speed: speed.unwrap_or(base.speed),
                },
            )?;
        }
        Command::NextColor | Command::PrevColor => {
            let led = Led::open()?;
            let base = state::load().unwrap_or(Settings::DEFAULT);
            let step = if cmd == Command::NextColor { 1 } else { -1 };
            apply(
                &led,
                &Settings {
                    color: base.stepped_color(step),
                    ..base
                },
            )?;
        }
        Command::NextMode => {
            let led = Led::open()?;
            let base = state::load().unwrap_or(Settings::DEFAULT);
            apply(
                &led,
                &Settings {
                    mode: base.mode.next(),
                    ..base
                },
            )?;
        }
        Command::NextPreset | Command::PrevPreset => {
            let led = Led::open()?;
            let step = if cmd == Command::NextPreset { 1 } else { -1 };
            apply(&led, &cli::cycle_preset(state::load(), step))?;
        }
        Command::Get => {
            let led = Led::open()?;
            println!("jasność: {}/{}", led.brightness()?, led.max_brightness()?);
            match state::load() {
                Some(s) => println!(
                    "ostatnie ustawienie: {} {} {}",
                    s.color,
                    s.mode.name(),
                    s.speed.name()
                ),
                None => println!("ostatnie ustawienie: brak (jeszcze nic nie ustawiono)"),
            }
        }
        Command::Restore => match state::load() {
            Some(s) => Led::open()?.set_mode(&s)?,
            None => eprintln!("a-rgb: brak zapisanego ustawienia, nic do przywrócenia"),
        },
    }
    Ok(())
}

/// Zapis do sprzętu, potem zapamiętanie stanu (błąd zapisu stanu nie jest krytyczny).
fn apply(led: &Led, s: &Settings) -> Result<(), String> {
    led.set_mode(s)?;
    if let Err(e) = state::save(s) {
        eprintln!("a-rgb: ostrzeżenie: nie zapisano stanu: {e}");
    }
    println!("{} {} {}", s.color, s.mode.name(), s.speed.name());
    Ok(())
}

fn brightness(led: &Led, b: Brightness) -> Result<(), String> {
    let max = led.max_brightness()?;
    let cur = led.brightness()?;
    let target = match b {
        Brightness::Show => {
            println!("{cur}/{max}");
            return Ok(());
        }
        Brightness::Set(v) if v > max => return Err(format!("jasność: maksimum to {max}")),
        Brightness::Set(v) => v,
        Brightness::Up => (cur + 1).min(max),
        Brightness::Down => cur.saturating_sub(1),
    };
    led.set_brightness(target)?;
    println!("{target}/{max}");
    Ok(())
}

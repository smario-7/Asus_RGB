mod cli;
mod model;
mod state;
mod sysfs;

use cli::{Brightness, Command};
use model::Settings;
use std::io::{self, Write};
use std::process::ExitCode;
use sysfs::Led;

/// Prints to stdout without panicking: a closed pipe (`a-rgb get | head -1`) or a full disk
/// must not abort the process (with `panic = "abort"` it would end in SIGABRT).
fn emit(text: &str) {
    let _ = io::stdout().write_all(text.as_bytes());
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = match cli::parse(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("a-rgb: {e}\nHelp: a-rgb -h");
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
        Command::Help(None) => emit(&cli::help_text()),
        Command::Help(Some(name)) => emit(&cli::command_help(&name)?),
        Command::Version => emit(&format!("a-rgb {}\n", env!("CARGO_PKG_VERSION"))),
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
            emit(&format!(
                "brightness: {}/{}\n",
                led.brightness()?,
                led.max_brightness()?
            ));
            match state::load() {
                Some(s) => emit(&format!(
                    "last setting: {} {} {}\n",
                    s.color,
                    s.mode.name(),
                    s.speed.name()
                )),
                None => emit("last setting: none (nothing has been set yet)\n"),
            }
        }
        Command::Restore => match state::load() {
            Some(s) => Led::open()?.set_mode(&s)?,
            None => eprintln!("a-rgb: no saved setting, nothing to restore"),
        },
    }
    Ok(())
}

/// Write to the hardware, then remember the state (failing to save the state is not fatal).
fn apply(led: &Led, s: &Settings) -> Result<(), String> {
    led.set_mode(s)?;
    if let Err(e) = state::save(s) {
        eprintln!("a-rgb: warning: could not save the state: {e}");
    }
    emit(&format!(
        "{} {} {}\n",
        s.color,
        s.mode.name(),
        s.speed.name()
    ));
    Ok(())
}

fn brightness(led: &Led, b: Brightness) -> Result<(), String> {
    let max = led.max_brightness()?;
    let cur = led.brightness()?;
    let target = match b {
        Brightness::Show => {
            emit(&format!("{cur}/{max}\n"));
            return Ok(());
        }
        Brightness::Set(v) if v > max => return Err(format!("brightness: the maximum is {max}")),
        Brightness::Set(v) => v,
        Brightness::Up => (cur + 1).min(max),
        Brightness::Down => cur.saturating_sub(1),
    };
    led.set_brightness(target)?;
    emit(&format!("{target}/{max}\n"));
    Ok(())
}

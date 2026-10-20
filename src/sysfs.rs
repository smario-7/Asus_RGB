//! Access to the keyboard LED in sysfs (`/sys/class/leds/asus::kbd_backlight`).

use crate::model::Settings;
use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};

const DEFAULT_DIR: &str = "/sys/class/leds/asus::kbd_backlight";

pub struct Led {
    dir: PathBuf,
}

impl Led {
    /// The directory can be overridden with the `A_RGB_SYSFS` variable (testing without hardware).
    pub fn open() -> Result<Led, String> {
        let dir = env::var_os("A_RGB_SYSFS")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_DIR));
        Led::at(dir)
    }

    pub fn at(dir: PathBuf) -> Result<Led, String> {
        if !dir.is_dir() {
            return Err(format!(
                "{} not found — is this an ASUS TUF with the asus-nb-wmi driver loaded?",
                dir.display()
            ));
        }
        Ok(Led { dir })
    }

    pub fn set_mode(&self, s: &Settings) -> Result<(), String> {
        self.write("kbd_rgb_mode", &s.sysfs_line())
    }

    pub fn brightness(&self) -> Result<u8, String> {
        self.read_u8("brightness")
    }

    pub fn max_brightness(&self) -> Result<u8, String> {
        self.read_u8("max_brightness")
    }

    pub fn set_brightness(&self, v: u8) -> Result<(), String> {
        self.write("brightness", &v.to_string())
    }

    fn write(&self, file: &str, value: &str) -> Result<(), String> {
        let path = self.dir.join(file);
        // No create: a sysfs attribute either exists or this hardware does not support it.
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&path)
            .and_then(|mut f| f.write_all(value.as_bytes()))
            .map_err(|e| describe(&e, &path))
    }

    fn read_u8(&self, file: &str) -> Result<u8, String> {
        let path = self.dir.join(file);
        let text = fs::read_to_string(&path).map_err(|e| describe(&e, &path))?;
        text.trim()
            .parse()
            .map_err(|_| format!("{}: unexpected content '{}'", path.display(), text.trim()))
    }
}

fn describe(e: &io::Error, path: &std::path::Path) -> String {
    if e.kind() == io::ErrorKind::PermissionDenied {
        format!(
            "permission denied for {} — install the udev rule (packaging/99-a-rgb.rules) and join the plugdev group",
            path.display()
        )
    } else if e.kind() == io::ErrorKind::NotFound {
        format!(
            "{} not found — this laptop or kernel does not provide this feature (color needs an ASUS TUF with the RGB interface in the asus-wmi driver)",
            path.display()
        )
    } else {
        format!("{}: {e}", path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Color, Mode, Speed};

    fn temp_led(name: &str) -> (Led, PathBuf) {
        let dir = env::temp_dir().join(format!("a-rgb-test-{}-{name}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("brightness"), "1\n").unwrap();
        fs::write(dir.join("max_brightness"), "3\n").unwrap();
        fs::write(dir.join("kbd_rgb_mode"), "").unwrap();
        (Led::at(dir.clone()).unwrap(), dir)
    }

    #[test]
    fn writes_mode_line_and_brightness() {
        let (led, dir) = temp_led("write");
        let s = Settings {
            color: Color::parse("ff8800").unwrap(),
            mode: Mode::Breathe,
            speed: Speed::Slow,
        };
        led.set_mode(&s).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("kbd_rgb_mode")).unwrap(),
            "0 1 255 136 0 0"
        );
        led.set_brightness(3).unwrap();
        assert_eq!(led.brightness().unwrap(), 3);
        assert_eq!(led.max_brightness().unwrap(), 3);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn missing_rgb_attribute_is_clear_error_and_not_created() {
        let (led, dir) = temp_led("norgb");
        fs::remove_file(dir.join("kbd_rgb_mode")).unwrap();
        let err = led.set_mode(&Settings::DEFAULT).unwrap_err();
        assert!(err.contains("does not provide this feature"), "{err}");
        assert!(!dir.join("kbd_rgb_mode").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn shorter_write_does_not_leave_tail() {
        let (led, dir) = temp_led("trunc");
        led.set_mode(&Settings::DEFAULT).unwrap();
        led.set_mode(&Settings {
            color: Color::parse("000000").unwrap(),
            ..Settings::DEFAULT
        })
        .unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("kbd_rgb_mode")).unwrap(),
            "0 0 0 0 0 1"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn missing_dir_is_error() {
        assert!(Led::at(PathBuf::from("/nonexistent/a-rgb")).is_err());
    }
}

//! The library: games found on this PC and what the launcher records about them. Built for
//! more than one title; today the Xbox Series build this project runs.
//!
//! What it shows about a game is read from the game's own folder (`MicrosoftGame.Config` and
//! the images it names), on this PC: nothing of any game is part of HybridGt.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default, Clone, Debug)]
pub struct Game {
    pub found: bool,
    pub dir: PathBuf,
    /// The title's identity name (`Identity Name`), the key of its records.
    pub id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub executable: String,
    pub archives: usize,
    pub bytes: u64,
    /// The title's own images, as its config names them.
    pub splash: Option<PathBuf>,
    pub logo: Option<PathBuf>,
}

pub fn attribute(xml: &str, element: &str, name: &str) -> Option<String> {
    let at = xml.find(&format!("<{element}"))?;
    let tag = &xml[at..at + xml[at..].find('>')?];
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = tag[start..].find('"')?;
    Some(tag[start..start + end].to_string())
}

pub fn read(dir: &str) -> Game {
    let dir = PathBuf::from(dir);
    let mut g = Game { dir: dir.clone(), ..Game::default() };
    let Ok(xml) = std::fs::read_to_string(dir.join("MicrosoftGame.Config")) else { return g };
    let exe = attribute(&xml, "Executable", "Name").unwrap_or_else(|| "game_scarlett_master.exe".into());
    if !dir.join(&exe).is_file() {
        return g;
    }
    g.found = true;
    g.executable = exe;
    g.id = attribute(&xml, "Identity", "Name").unwrap_or_default();
    g.name = attribute(&xml, "ShellVisuals", "DefaultDisplayName").unwrap_or_else(|| "Unnamed title".into());
    g.version = attribute(&xml, "Identity", "Version").unwrap_or_default();
    g.publisher = attribute(&xml, "ShellVisuals", "PublisherDisplayName").unwrap_or_default();
    let image = |key: &str| attribute(&xml, "ShellVisuals", key).map(|f| dir.join(f)).filter(|p| p.is_file());
    g.splash = image("SplashScreenImage");
    g.logo = image("Square480x480Logo").or_else(|| image("Square150x150Logo"));
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let Ok(meta) = e.metadata() else { continue };
            if meta.is_file() {
                g.bytes += meta.len();
                if e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("rpf")) {
                    g.archives += 1;
                }
            }
        }
    }
    g
}

/// An image file as RGBA (the title's own art, shown as it is).
pub fn load_image(path: &Path, max_side: u32) -> Option<([usize; 2], Vec<u8>)> {
    let bytes = std::fs::read(path).ok()?;
    let mut img = image::load_from_memory(&bytes).ok()?;
    if img.width().max(img.height()) > max_side {
        img = img.thumbnail(max_side, max_side);
    }
    let rgba = img.to_rgba8();
    Some(([rgba.width() as usize, rgba.height() as usize], rgba.into_raw()))
}

/// Time played per title, recorded while the launcher runs a game.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Played {
    pub seconds: u64,
    pub last: Option<u64>,
    pub sessions: u64,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Playtime {
    pub titles: BTreeMap<String, Played>,
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

impl Playtime {
    pub fn path() -> PathBuf {
        crate::config::app_dir().join("playtime.json")
    }

    pub fn load() -> Self {
        std::fs::read_to_string(Self::path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }

    pub fn save(&self) {
        let p = Self::path();
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        if let Ok(t) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(p, t);
        }
    }

    pub fn of(&self, id: &str) -> Played {
        self.titles.get(id).cloned().unwrap_or_default()
    }

    /// A session began now.
    pub fn begin(&mut self, id: &str) {
        let p = self.titles.entry(id.to_string()).or_default();
        p.sessions += 1;
        p.last = Some(now());
    }

    /// `secs` more of play (counted while the game runs, so a launcher closed early loses at
    /// most the last tick).
    pub fn add(&mut self, id: &str, secs: u64) {
        let p = self.titles.entry(id.to_string()).or_default();
        p.seconds += secs;
        p.last = Some(now());
    }
}

pub fn duration(secs: u64) -> String {
    match secs {
        0..=59 => "under a minute".into(),
        60..=3599 => format!("{} min", secs / 60),
        _ => format!("{} h {} min", secs / 3600, secs % 3600 / 60),
    }
}

pub fn ago(unix: u64) -> String {
    let s = now().saturating_sub(unix);
    match s {
        0..=59 => "just now".into(),
        60..=3599 => format!("{} min ago", s / 60),
        3600..=86_399 => format!("{} h ago", s / 3600),
        _ => format!("{} days ago", s / 86_400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_attributes_are_read() {
        let xml = r#"<Identity Name="A.B" Publisher="CN=x" Version="1.0.2.1" /><ShellVisuals DefaultDisplayName="Some Game" PublisherDisplayName="Some Publisher" SplashScreenImage="s.png" />"#;
        assert_eq!(attribute(xml, "Identity", "Version").as_deref(), Some("1.0.2.1"));
        assert_eq!(attribute(xml, "Identity", "Name").as_deref(), Some("A.B"));
        assert_eq!(attribute(xml, "ShellVisuals", "DefaultDisplayName").as_deref(), Some("Some Game"));
        // `Name` is not found inside `DefaultDisplayName`.
        assert_eq!(attribute(xml, "ShellVisuals", "Name"), None);
    }

    #[test]
    fn a_folder_without_the_game_is_not_found() {
        assert!(!read(&std::env::temp_dir().display().to_string()).found);
    }

    #[test]
    fn playtime_adds_up_and_counts_sessions() {
        let mut p = Playtime::default();
        p.begin("t");
        p.add("t", 90);
        p.add("t", 30);
        let t = p.of("t");
        assert_eq!((t.seconds, t.sessions), (120, 1));
        assert!(t.last.is_some());
        assert_eq!(duration(3_720), "1 h 2 min");
        let back: Playtime = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(back.of("t").seconds, 120);
    }

    #[test]
    fn an_image_is_decoded_and_scaled() {
        let dir = std::env::temp_dir().join(format!("hgt-img-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("a.png");
        image::RgbaImage::from_pixel(64, 32, image::Rgba([1, 2, 3, 255])).save(&path).unwrap();
        let ([w, h], px) = load_image(&path, 16).unwrap();
        assert_eq!((w, h, px.len()), (16, 8, 16 * 8 * 4));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

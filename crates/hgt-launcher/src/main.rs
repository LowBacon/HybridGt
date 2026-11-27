//! HybridGt launcher: one window to point at your own copy of the game, prepare its shaders,
//! pick a preset and start the runtime, with the frame rate shown while it runs.
//!
//! It ships nothing of the game: everything it shows about the game is read from the folder
//! you choose, on your PC. The look is HybridGt's own (no logos or art of the game).
#![windows_subsystem = "windows"]

mod presets;
mod scan;
mod style;

use eframe::egui::{self, Align, Color32, Layout, RichText, Vec2};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

use presets::Preset;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("HybridGt")
            .with_inner_size([1180.0, 740.0])
            .with_min_inner_size([980.0, 620.0])
            .with_icon(icon()),
        centered: true,
        ..Default::default()
    };
    eframe::run_native("HybridGt", options, Box::new(|cc| Ok(Box::new(Launcher::new(cc)))))
}

/// The window's icon: a sunset disc with a dark band, drawn here (no art of the game).
fn icon() -> egui::IconData {
    const N: u32 = 64;
    let mut rgba = Vec::with_capacity((N * N * 4) as usize);
    for y in 0..N {
        for x in 0..N {
            let (fx, fy) = (x as f32 + 0.5 - 32.0, y as f32 + 0.5 - 32.0);
            let r = (fx * fx + fy * fy).sqrt();
            let t = y as f32 / N as f32;
            let top = [255.0, 170.0, 72.0];
            let bottom = [124.0, 58.0, 237.0];
            let mid = [255.0, 79.0, 139.0];
            let c = if t < 0.5 {
                let k = t * 2.0;
                [top[0] + (mid[0] - top[0]) * k, top[1] + (mid[1] - top[1]) * k, top[2] + (mid[2] - top[2]) * k]
            } else {
                let k = (t - 0.5) * 2.0;
                [mid[0] + (bottom[0] - mid[0]) * k, mid[1] + (bottom[1] - mid[1]) * k, mid[2] + (bottom[2] - mid[2]) * k]
            };
            let band = (y as i32 - 40).rem_euclid(9) < 3 && y > 36;
            let alpha = (31.5 - r).clamp(0.0, 1.0);
            let k = if band { 0.25 } else { 1.0 };
            rgba.extend([(c[0] * k) as u8, (c[1] * k) as u8, (c[2] * k) as u8, (alpha * 255.0) as u8]);
        }
    }
    egui::IconData { rgba, width: N, height: N }
}

/// What the launcher remembers, in `hybridgt-launcher.json` beside it.
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
struct Config {
    game_dir: String,
    runtime_exe: String,
    /// The earlier runtime (fxceless's victory-gta6 with HybridGt's patch), for "Classic".
    classic_exe: String,
    classic_cache_dir: String,
    classic_data_dir: String,
    /// 2 since the runtime moved to the VIctory-NVIDIA base (0.2.0); a config without it is
    /// older (0), not the default.
    #[serde(default)]
    version: u32,
    /// Read from launchers before 0.2.0 only.
    #[serde(skip_serializing)]
    nvidia_exe: String,
    source_dir: String,
    cache_dir: String,
    data_dir: String,
    logs_dir: String,
    ffx_dir: String,
    #[serde(deserialize_with = "presets::lenient")]
    preset: Preset,
    inplace: bool,
    hash_wide: bool,
    ambient: bool,
    audio_delay_ms: u32,
}

impl Default for Config {
    fn default() -> Self {
        let home = app_dir();
        Config {
            game_dir: String::new(),
            runtime_exe: String::new(),
            classic_exe: String::new(),
            classic_cache_dir: home.join("classic-cache").display().to_string(),
            classic_data_dir: home.join("classic-data").display().to_string(),
            version: 2,
            nvidia_exe: String::new(),
            source_dir: String::new(),
            cache_dir: home.join("cache").display().to_string(),
            data_dir: home.join("data").display().to_string(),
            logs_dir: home.join("logs").display().to_string(),
            ffx_dir: String::new(),
            preset: Preset::Fast,
            inplace: false,
            hash_wide: false,
            ambient: true,
            audio_delay_ms: 0,
        }
    }
}

/// `%LOCALAPPDATA%\HybridGt`.
fn app_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("HybridGt")
}

fn config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("hybridgt-launcher.json")))
        .unwrap_or_else(|| app_dir().join("hybridgt-launcher.json"))
}

impl Config {
    fn load() -> Self {
        let mut c: Config = std::fs::read_to_string(config_path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        if c.version < 2 {
            // Before 0.2.0 the runtime was the earlier one: it and its folders become Classic's
            // (saves and warm caches kept), and the VIctory-NVIDIA build, if one was set, the main.
            let defaults = Config::default();
            c.classic_exe = std::mem::take(&mut c.runtime_exe);
            c.classic_cache_dir = std::mem::replace(&mut c.cache_dir, defaults.cache_dir);
            c.classic_data_dir = std::mem::replace(&mut c.data_dir, defaults.data_dir);
            c.runtime_exe = std::mem::take(&mut c.nvidia_exe);
            c.version = 2;
        }
        c
    }
    fn save(&self) {
        if let Ok(t) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(config_path(), t);
        }
    }
}

/// What the game folder says about itself (`MicrosoftGame.Config`), and its size.
#[derive(Default, Clone)]
struct GameInfo {
    found: bool,
    name: String,
    version: String,
    publisher: String,
    executable: String,
    archives: usize,
    bytes: u64,
}

fn attribute(xml: &str, element: &str, name: &str) -> Option<String> {
    let at = xml.find(&format!("<{element}"))?;
    let tag = &xml[at..at + xml[at..].find('>')?];
    let key = format!("{name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = tag[start..].find('"')?;
    Some(tag[start..start + end].to_string())
}

fn read_game(dir: &str) -> GameInfo {
    let dir = Path::new(dir);
    let mut info = GameInfo::default();
    let exe = dir.join("game_scarlett_master.exe");
    if !exe.is_file() {
        return info;
    }
    info.found = true;
    info.executable = "game_scarlett_master.exe".into();
    if let Ok(xml) = std::fs::read_to_string(dir.join("MicrosoftGame.Config")) {
        info.name = attribute(&xml, "ShellVisuals", "DefaultDisplayName").unwrap_or_default();
        info.version = attribute(&xml, "Identity", "Version").unwrap_or_default();
        info.publisher = attribute(&xml, "ShellVisuals", "PublisherDisplayName").unwrap_or_default();
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let Ok(meta) = e.metadata() else { continue };
            if meta.is_file() {
                info.bytes += meta.len();
                if e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("rpf")) {
                    info.archives += 1;
                }
            }
        }
    }
    info
}

/// The newest file under `dir` (saves) and how long ago it was written.
fn newest(dir: &Path) -> Option<SystemTime> {
    let mut best: Option<SystemTime> = None;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).ok()?.flatten() {
            let Ok(meta) = e.metadata() else { continue };
            if meta.is_dir() {
                stack.push(e.path());
            } else if let Ok(m) = meta.modified() {
                best = Some(best.map_or(m, |b| b.max(m)));
            }
        }
    }
    best
}

fn ago(t: SystemTime) -> String {
    let s = SystemTime::now().duration_since(t).map(|d| d.as_secs()).unwrap_or(0);
    match s {
        0..=59 => "just now".into(),
        60..=3599 => format!("{} min ago", s / 60),
        3600..=86_399 => format!("{} h ago", s / 3600),
        _ => format!("{} days ago", s / 86_400),
    }
}

/// The frame line the runtime logs every second.
#[derive(Default, Clone, Copy)]
struct Live {
    fps: f32,
    ms: f32,
    draws: u32,
}

fn parse_live(text: &str) -> Option<Live> {
    let line = text.lines().rev().find(|l| l.contains(" fps, ") && l.contains("ms/frame") && l.contains("draws/frame"))?;
    let num_before = |s: &str, key: &str| -> Option<f32> {
        let end = s.find(key)?;
        let head = s[..end].trim_end();
        let start = head.rfind(|c: char| !(c.is_ascii_digit() || c == '.')).map_or(0, |i| i + 1);
        head[start..].parse().ok()
    };
    let after_fps = &line[line.find(": ").map_or(0, |i| i + 2)..];
    Some(Live {
        fps: num_before(after_fps, " fps")?,
        ms: num_before(after_fps, " ms/frame")?,
        draws: num_before(after_fps, " draws/frame")? as u32,
    })
}

fn tail(path: &Path, bytes: u64) -> String {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut f) = std::fs::File::open(path) else { return String::new() };
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    let _ = f.seek(SeekFrom::Start(len.saturating_sub(bytes)));
    let mut buf = Vec::new();
    let _ = f.read_to_end(&mut buf);
    // The runtime colours its lines; the viewer shows plain text.
    let text = String::from_utf8_lossy(&buf).into_owned();
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            for d in chars.by_ref() {
                if d.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[derive(PartialEq, Clone, Copy)]
enum Page {
    Play,
    Game,
    Settings,
    Builds,
    Logs,
    About,
}

/// A process the launcher started: the game, a shader warm-up or a build.
struct Job {
    child: Child,
    what: &'static str,
    #[allow(dead_code)]
    log: PathBuf,
    started: Instant,
}

struct Launcher {
    config: Config,
    page: Page,
    game: GameInfo,
    last_save: Option<SystemTime>,
    job: Option<Job>,
    last_log: Option<PathBuf>,
    live: Option<Live>,
    log_text: String,
    polled: Instant,
    message: String,
    scanning: Option<std::sync::mpsc::Receiver<scan::Found>>,
    scan_seen: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl Launcher {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        style::apply(&cc.egui_ctx);
        let mut config = Config::load();
        unpack_runtime(&mut config);
        let game = read_game(&config.game_dir);
        let last_save = newest(&Path::new(&config.data_dir).join("7952987A").join("saves"));
        let missing = !scan::is_game_dir(Path::new(&config.game_dir)) || !Path::new(&config.runtime_exe).is_file();
        let mut launcher = Launcher {
            config,
            page: Page::Play,
            game,
            last_save,
            job: None,
            last_log: None,
            live: None,
            log_text: String::new(),
            polled: Instant::now(),
            message: String::new(),
            scanning: None,
            scan_seen: Default::default(),
        };
        if missing {
            launcher.start_scan();
        }
        launcher
    }

    /// Looks for the game, the runtime and the folders on a thread of its own.
    fn start_scan(&mut self) {
        if self.scanning.is_some() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let seen = self.scan_seen.clone();
        seen.store(0, std::sync::atomic::Ordering::Relaxed);
        std::thread::spawn(move || {
            let found = scan::scan(|n| seen.store(n, std::sync::atomic::Ordering::Relaxed));
            let _ = tx.send(found);
        });
        self.scanning = Some(rx);
        self.message = "Looking for the game and the runtime on this PC...".into();
    }

    /// Takes what the scan found where nothing usable was chosen.
    fn take_scan(&mut self, found: scan::Found) {
        let c = &mut self.config;
        let defaults = Config::default();
        let mut took = Vec::new();
        if let Some(g) = found.game_dir
            && !scan::is_game_dir(Path::new(&c.game_dir))
        {
            c.game_dir = g.display().to_string();
            took.push("game");
        }
        if let Some(r) = found.runtime_exe
            && !Path::new(&c.runtime_exe).is_file()
        {
            c.runtime_exe = r.display().to_string();
            took.push("runtime");
        }
        if let Some(r) = found.classic_exe
            && !Path::new(&c.classic_exe).is_file()
        {
            c.classic_exe = r.display().to_string();
        }
        if let Some(s) = found.source_dir
            && c.source_dir.is_empty()
        {
            c.source_dir = s.display().to_string();
        }
        for (slot, value, default) in [
            (&mut c.cache_dir, found.cache_dir, defaults.cache_dir),
            (&mut c.data_dir, found.data_dir, defaults.data_dir),
            (&mut c.classic_cache_dir, found.classic_cache_dir, defaults.classic_cache_dir),
            (&mut c.classic_data_dir, found.classic_data_dir, defaults.classic_data_dir),
            (&mut c.ffx_dir, found.ffx_dir, defaults.ffx_dir),
        ] {
            if let Some(v) = value
                && (*slot == default || slot.is_empty())
            {
                *slot = v;
                took.push("folders");
            }
        }
        took.dedup();
        c.save();
        self.message = if took.is_empty() {
            format!("Scan done ({} folders): nothing new to take.", found.folders_seen)
        } else {
            format!("Found and set: {}.", took.join(", "))
        };
        self.refresh_game();
    }

    fn refresh_game(&mut self) {
        self.game = read_game(&self.config.game_dir);
        self.last_save = newest(&Path::new(&self.config.data_dir).join("7952987A").join("saves"));
    }

    fn runtime_for(&self, preset: Preset) -> String {
        if preset.classic() { self.config.classic_exe.clone() } else { self.config.runtime_exe.clone() }
    }

    /// The switches a run gets: the preset's, the toggles, and the folders.
    fn environment(&self, preset: Preset) -> Vec<(String, String)> {
        let c = &self.config;
        let mut env: Vec<(String, String)> =
            preset.switches().iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        let mut set = |k: &str, v: String| {
            env.retain(|(key, _)| key != k);
            env.push((k.to_string(), v));
        };
        set("VICTORY_GAME_DIR", c.game_dir.clone());
        let (cache, data) = if preset.classic() { (&c.classic_cache_dir, &c.classic_data_dir) } else { (&c.cache_dir, &c.data_dir) };
        set("VICTORY_CACHE_DIR", cache.clone());
        set("VICTORY_DATA_DIR", data.clone());
        if preset.classic() {
            // Device method 0x218's index of the game's own shader heads, made on this PC.
            let index = Path::new(cache).join("native-index.txt");
            if index.is_file() {
                set("VICTORY_NATIVE_INDEX", index.display().to_string());
            }
            if c.inplace {
                set("VICTORY_INPLACE", "1".into());
            }
            if c.hash_wide {
                set("VICTORY_CARD_HASH_WIDE", "1".into());
            }
        }
        if !c.ambient {
            env.retain(|(k, _)| k != "VICTORY_AMBIENT" && k != "VICTORY_AMBIENT_AIR");
        }
        if c.audio_delay_ms > 0 {
            env.push(("VICTORY_AUDIO_DELAY_MS".into(), c.audio_delay_ms.to_string()));
        }
        if preset.fsr() && !c.ffx_dir.is_empty() {
            env.push(("VICTORY_FFX_DIR".into(), c.ffx_dir.clone()));
        }
        env
    }

    fn start(&mut self, what: &'static str, exe: &str, args: Vec<String>, env: Vec<(String, String)>, cwd: Option<PathBuf>) {
        if self.job.is_some() {
            self.message = "Something is already running.".into();
            return;
        }
        if !Path::new(exe).is_file() {
            self.message = format!("No program at {exe}: set it under Builds.");
            return;
        }
        let logs = PathBuf::from(&self.config.logs_dir);
        let _ = std::fs::create_dir_all(&logs);
        let log = logs.join(format!("{}-{}.log", what.replace(' ', "-"), stamp()));
        let Ok(file) = std::fs::File::create(&log) else {
            self.message = format!("Cannot write the log at {}", log.display());
            return;
        };
        let Ok(err) = file.try_clone() else { return };
        let mut cmd = Command::new(exe);
        cmd.args(&args).stdout(Stdio::from(file)).stderr(Stdio::from(err)).stdin(Stdio::null());
        for (k, _) in std::env::vars().filter(|(k, _)| k.starts_with("VICTORY_")) {
            cmd.env_remove(k);
        }
        cmd.envs(env);
        if let Some(dir) = cwd.or_else(|| Path::new(exe).parent().map(Path::to_path_buf)) {
            cmd.current_dir(dir);
        }
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        match cmd.spawn() {
            Ok(child) => {
                self.message = format!("{what}: started.");
                self.last_log = Some(log.clone());
                self.live = None;
                self.job = Some(Job { child, what, log, started: Instant::now() });
            }
            Err(e) => self.message = format!("{what}: could not start ({e})."),
        }
    }

    fn play(&mut self) {
        if !self.game.found {
            self.message = "Choose your game folder first (the one with game_scarlett_master.exe).".into();
            self.page = Page::Game;
            return;
        }
        let preset = self.config.preset;
        let exe = self.runtime_for(preset);
        let game_exe = Path::new(&self.config.game_dir).join("game_scarlett_master.exe").display().to_string();
        let mut args = vec!["run".to_string(), game_exe, "--execute".to_string()];
        if let Some(console) = preset.console() {
            args.splice(2..2, ["--console".to_string(), console.to_string()]);
        }
        let env = self.environment(preset);
        self.config.save();
        self.start("game", &exe, args, env, None);
    }

    fn warm(&mut self) {
        let preset = self.config.preset;
        let exe = self.runtime_for(preset);
        let mut env = self.environment(preset);
        env.extend([("VICTORY_HOST_WAVE".into(), "32".into()), ("VICTORY_VIEW_LIMIT".into(), "999999".into())]);
        let rpf = Path::new(&self.config.game_dir).join("shaders.rpf").display().to_string();
        self.start("shader warm-up", &exe, vec!["translate".into(), "--warm".into(), "all".into(), rpf], env, None);
    }

    fn build(&mut self) {
        let src = PathBuf::from(&self.config.source_dir);
        if !src.join("Cargo.toml").is_file() {
            self.message = "Set the runtime's source folder (with Cargo.toml) first.".into();
            return;
        }
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let exe = which(&cargo).unwrap_or(cargo);
        self.start("build", &exe, vec!["build".into(), "--release".into(), "-p".into(), "victory".into()], Vec::new(), Some(src));
    }

    fn poll(&mut self, ctx: &egui::Context) {
        if self.polled.elapsed() < Duration::from_millis(700) {
            return;
        }
        self.polled = Instant::now();
        if let Some(job) = &mut self.job
            && let Ok(Some(status)) = job.child.try_wait()
        {
            self.message = format!("{} finished ({}).", job.what, if status.success() { "ok" } else { "with an error: see Logs" });
            if job.what == "build" && status.success() {
                let built = Path::new(&self.config.source_dir).join("target").join("release").join("victory.exe");
                if built.is_file() {
                    self.config.runtime_exe = built.display().to_string();
                    self.config.save();
                }
            }
            self.job = None;
        }
        if let Some(log) = &self.last_log {
            let text = tail(log, 96 * 1024);
            if let Some(live) = parse_live(&text) {
                self.live = Some(live);
            }
            if self.page == Page::Logs {
                let lines: Vec<&str> = text.lines().collect();
                self.log_text = lines[lines.len().saturating_sub(300)..].join("\n");
            }
        }
        if self.job.is_some() {
            ctx.request_repaint_after(Duration::from_millis(700));
        }
    }
}

fn stamp() -> String {
    let s = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    format!("{s}")
}

fn which(name: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        for candidate in [dir.join(format!("{name}.exe")), dir.join(name)] {
            if candidate.is_file() {
                return Some(candidate.display().to_string());
            }
        }
    }
    let cargo_home = std::env::var_os("USERPROFILE").map(|h| PathBuf::from(h).join(".cargo").join("bin").join("cargo.exe"))?;
    cargo_home.is_file().then(|| cargo_home.display().to_string())
}

/// With a runtime inside this exe (`HGT_RUNTIME_EXE` at build time), it is unpacked to
/// `%LOCALAPPDATA%\HybridGt\runtime\victory.exe` when missing or different, and used.
fn unpack_runtime(config: &mut Config) {
    #[cfg(embedded_runtime)]
    {
        static RUNTIME: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/runtime.bin"));
        let dir = app_dir().join("runtime");
        let exe = dir.join("victory.exe");
        let same = std::fs::metadata(&exe).is_ok_and(|m| m.len() == RUNTIME.len() as u64);
        if !same {
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(&exe, RUNTIME);
        }
        if config.runtime_exe.is_empty() || !Path::new(&config.runtime_exe).is_file() {
            config.runtime_exe = exe.display().to_string();
        }
    }
    #[cfg(not(embedded_runtime))]
    let _ = config;
}

impl eframe::App for Launcher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll(ctx);
        if let Some(rx) = &self.scanning
            && let Ok(found) = rx.try_recv()
        {
            self.scanning = None;
            self.take_scan(found);
        }
        let t = ctx.input(|i| i.time) as f32;
        ctx.request_repaint_after(Duration::from_millis(33));

        egui::TopBottomPanel::top("header").exact_height(118.0).frame(egui::Frame::none()).show(ctx, |ui| {
            style::header(ui, t, self.job.as_ref().map(|j| j.what), self.live);
        });

        egui::SidePanel::left("nav")
            .exact_width(212.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(style::PANEL).inner_margin(egui::Margin::symmetric(14.0, 18.0)))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                for (page, label, icon) in [
                    (Page::Play, "Play", "▶"),
                    (Page::Game, "Game", "◆"),
                    (Page::Settings, "Settings", "⚙"),
                    (Page::Builds, "Builds", "⛭"),
                    (Page::Logs, "Logs", "☰"),
                    (Page::About, "About", "ℹ"),
                ] {
                    if style::nav_button(ui, &format!("{icon}   {label}"), self.page == page).clicked() {
                        self.page = page;
                    }
                }
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    ui.label(RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).color(style::MUTED).small());
                    if !self.message.is_empty() {
                        ui.label(RichText::new(&self.message).color(style::SOFT).small());
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(style::BACKGROUND).inner_margin(egui::Margin::same(26.0)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match self.page {
                    Page::Play => self.page_play(ui),
                    Page::Game => self.page_game(ui),
                    Page::Settings => self.page_settings(ui),
                    Page::Builds => self.page_builds(ui),
                    Page::Logs => self.page_logs(ui),
                    Page::About => page_about(ui),
                });
            });
    }
}

impl Launcher {
    fn page_play(&mut self, ui: &mut egui::Ui) {
        style::title(ui, "Ready to play");
        ui.add_space(6.0);
        style::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let name = if self.game.found && !self.game.name.is_empty() { self.game.name.clone() } else { "No game folder chosen".into() };
                    ui.label(RichText::new(name).size(24.0).strong().color(Color32::WHITE));
                    let sub = if self.game.found {
                        format!("Version {}  ·  {}  ·  {}", self.game.version, self.config.preset.label(), self.last_save.map(ago).map_or("no save yet".into(), |a| format!("last save {a}")))
                    } else {
                        "Open Game and choose the folder of your own copy.".into()
                    };
                    ui.label(RichText::new(sub).color(style::SOFT));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let running = self.job.as_ref().is_some_and(|j| j.what == "game");
                    if running {
                        if style::big_button(ui, "STOP", style::STOP).clicked()
                            && let Some(job) = &mut self.job
                        {
                            let _ = job.child.kill();
                        }
                    } else if style::big_button(ui, "PLAY", style::ACCENT).clicked() {
                        self.play();
                    }
                });
            });
        });
        ui.add_space(14.0);
        ui.label(RichText::new("Preset").color(style::SOFT).strong());
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(12.0, 12.0);
            for preset in Preset::ALL {
                if style::preset_card(ui, preset.label(), preset.blurb(), self.config.preset == preset).clicked() {
                    self.config.preset = preset;
                    self.config.save();
                }
            }
        });
        ui.add_space(14.0);
        style::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_max_width(ui.available_width() - 210.0);
                    ui.label(RichText::new("First start of a new build?").strong().color(Color32::WHITE));
                    ui.label(RichText::new("Translate every shader ahead of time so the game does not stutter while it plays (a few minutes, once per build).").color(style::SOFT));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if style::pill(ui, "Prepare shaders", false).clicked() {
                        self.warm();
                    }
                });
            });
        });
        if let Some(job) = &self.job {
            ui.add_space(14.0);
            style::card(ui, |ui| {
                let secs = job.started.elapsed().as_secs();
                ui.label(RichText::new(format!("{} running for {}:{:02}", job.what, secs / 60, secs % 60)).strong().color(Color32::WHITE));
                if let Some(live) = self.live {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        style::stat(ui, "FPS", &format!("{:.1}", live.fps));
                        style::stat(ui, "Frame", &format!("{:.0} ms", live.ms));
                        style::stat(ui, "Draws", &format!("{}", live.draws));
                    });
                }
            });
        }
    }

    fn page_game(&mut self, ui: &mut egui::Ui) {
        style::title(ui, "Your game");
        ui.horizontal(|ui| {
            if self.scanning.is_some() {
                ui.add(egui::Spinner::new().color(style::ACCENT));
                ui.label(RichText::new(format!("Scanning this PC... {} folders", self.scan_seen.load(std::sync::atomic::Ordering::Relaxed))).color(style::SOFT));
            } else if style::pill(ui, "Scan this PC", false).clicked() {
                self.start_scan();
            }
        });
        ui.add_space(8.0);
        style::card(ui, |ui| {
            ui.label(RichText::new("Game folder (the one with game_scarlett_master.exe)").color(style::SOFT));
            if style::path_field(ui, &mut self.config.game_dir) {
                self.config.save();
                self.refresh_game();
            }
        });
        ui.add_space(12.0);
        style::card(ui, |ui| {
            if !self.game.found {
                ui.label(RichText::new("No game_scarlett_master.exe in that folder.").color(style::SOFT));
                return;
            }
            egui::Grid::new("game").num_columns(2).spacing([28.0, 10.0]).show(ui, |ui| {
                for (k, v) in [
                    ("Title", self.game.name.clone()),
                    ("Version", self.game.version.clone()),
                    ("Publisher", self.game.publisher.clone()),
                    ("Program", self.game.executable.clone()),
                    ("Archives", format!("{} .rpf files", self.game.archives)),
                    ("Size", format!("{:.1} GB", self.game.bytes as f64 / 1e9)),
                    ("Last save", self.last_save.map(ago).unwrap_or_else(|| "none yet".into())),
                ] {
                    ui.label(RichText::new(k).color(style::MUTED));
                    ui.label(RichText::new(v).color(Color32::WHITE).strong());
                    ui.end_row();
                }
            });
        });
        ui.add_space(10.0);
        ui.label(RichText::new("Read from your own files on this PC; nothing of the game is part of HybridGt.").small().color(style::MUTED));
    }

    fn page_settings(&mut self, ui: &mut egui::Ui) {
        style::title(ui, "Settings");
        let mut changed = false;
        style::card(ui, |ui| {
            changed |= style::toggle_row(ui, "Stand-in light", "Lights interiors in place of the game's traced light (about 1 fps).", &mut self.config.ambient);
            changed |= style::toggle_row(ui, "In-place memory (Classic)", "Opens the game's memory to the card directly (measured +1 fps, one run).", &mut self.config.inplace);
            changed |= style::toggle_row(ui, "Wide hashing (Classic)", "Hashes copies on the card 64 threads a piece (faster median, bigger spikes).", &mut self.config.hash_wide);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Sound delay").strong().color(Color32::WHITE));
                    ui.label(RichText::new("Holds speech back to meet the mouths at low frame rates (0: off).").color(style::SOFT));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    changed |= ui.add(egui::Slider::new(&mut self.config.audio_delay_ms, 0..=600).suffix(" ms")).changed();
                });
            });
        });
        ui.add_space(12.0);
        style::card(ui, |ui| {
            ui.label(RichText::new("Folders").strong().color(Color32::WHITE));
            for (label, value) in [
                ("Shader cache", &mut self.config.cache_dir),
                ("Saves and settings", &mut self.config.data_dir),
                ("Logs", &mut self.config.logs_dir),
                ("AMD FidelityFX DLLs (for FSR 3.1)", &mut self.config.ffx_dir),
            ] {
                ui.label(RichText::new(label).color(style::SOFT));
                changed |= style::path_field(ui, value);
            }
        });
        if changed {
            self.config.save();
        }
    }

    fn page_builds(&mut self, ui: &mut egui::Ui) {
        style::title(ui, "Builds");
        let mut changed = false;
        style::card(ui, |ui| {
            ui.label(RichText::new("HybridGt runtime (victory.exe, VIctory-NVIDIA base)").strong().color(Color32::WHITE));
            changed |= style::path_field(ui, &mut self.config.runtime_exe);
            ui.add_space(6.0);
            ui.label(RichText::new("Classic runtime (optional, for the Classic preset)").strong().color(Color32::WHITE));
            changed |= style::path_field(ui, &mut self.config.classic_exe);
        });
        ui.add_space(12.0);
        style::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_max_width(ui.available_width() - 140.0);
                    ui.label(RichText::new("Build the runtime from source").strong().color(Color32::WHITE));
                    ui.label(RichText::new("Needs Rust and the Visual Studio C++ tools. Uses the folder with the runtime's Cargo.toml.").color(style::SOFT));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if style::pill(ui, "Build", false).clicked() {
                        self.build();
                    }
                });
            });
            changed |= style::path_field(ui, &mut self.config.source_dir);
        });
        if changed {
            self.config.save();
        }
    }

    fn page_logs(&mut self, ui: &mut egui::Ui) {
        style::title(ui, "Log");
        if self.last_log.is_none() {
            ui.label(RichText::new("Nothing started yet this session.").color(style::SOFT));
            return;
        }
        if let Some(log) = &self.last_log {
            ui.label(RichText::new(log.display().to_string()).small().color(style::MUTED));
        }
        style::card(ui, |ui| {
            egui::ScrollArea::vertical().max_height(470.0).stick_to_bottom(true).show(ui, |ui| {
                ui.add(egui::Label::new(RichText::new(&self.log_text).monospace().size(11.5).color(style::SOFT)).wrap());
            });
        });
    }
}

fn page_about(ui: &mut egui::Ui) {
    style::title(ui, "About HybridGt");
    style::card(ui, |ui| {
        ui.label(RichText::new("HybridGt runs the Xbox Series build of a game you own on a Windows PC, through the victory-gta6 runtime (a fork of VIctory).").color(Color32::WHITE));
        ui.add_space(8.0);
        ui.label(RichText::new("Built on the work of VIctory (Marvisak), fxceless's victory-gta6, VIctory-NVIDIA (pepezin-ai), foxhound (readmet3xt-lab), VIctory-7900XTX, Belobug, DexNom, ZTRX, VInAMD, cool_fox, Neptune VI and Kelvaroq, each credited in the project's history and THIRD_PARTY.md.").color(style::SOFT));
        ui.add_space(8.0);
        ui.label(RichText::new("No game files, keys or art are included or downloaded. GPL-3.0-or-later. Not affiliated with or endorsed by the game's publisher.").color(style::MUTED));
        ui.add_space(8.0);
        ui.hyperlink_to("github.com/LowBacon/HybridGt", "https://github.com/LowBacon/HybridGt");
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frame_line_is_read() {
        let line = "2026-11-27T12:28:16Z  INFO victory_gpu::host: 6.9 fps, 145.2 ms/frame (90.1 preparing: copies 1.0 ms 2.0 MB), 4552 draws/frame, dropped 3";
        let live = parse_live(line).expect("parsed");
        assert!((live.fps - 6.9).abs() < 1e-6 && (live.ms - 145.2).abs() < 1e-3 && live.draws == 4552);
    }

    #[test]
    fn game_config_attributes_are_read() {
        let xml = r#"<Identity Name="A.B" Publisher="CN=x" Version="1.0.2.1" /><ShellVisuals DefaultDisplayName="Some Game" PublisherDisplayName="Some Publisher" />"#;
        assert_eq!(attribute(xml, "Identity", "Version").as_deref(), Some("1.0.2.1"));
        assert_eq!(attribute(xml, "ShellVisuals", "DefaultDisplayName").as_deref(), Some("Some Game"));
        assert_eq!(attribute(xml, "ShellVisuals", "PublisherDisplayName").as_deref(), Some("Some Publisher"));
    }

    #[test]
    fn an_older_launchers_config_moves_to_classic() {
        let old = r#"{"game_dir":"G","runtime_exe":"old.exe","nvidia_exe":"new.exe","cache_dir":"C","data_dir":"D","preset":"Performance"}"#;
        let mut c: Config = serde_json::from_str(old).unwrap();
        assert_eq!(c.preset, Preset::Fast, "an unknown preset reads as Fast");
        // as `Config::load` does
        if c.version < 2 {
            let defaults = Config::default();
            c.classic_exe = std::mem::take(&mut c.runtime_exe);
            c.classic_cache_dir = std::mem::replace(&mut c.cache_dir, defaults.cache_dir);
            c.classic_data_dir = std::mem::replace(&mut c.data_dir, defaults.data_dir);
            c.runtime_exe = std::mem::take(&mut c.nvidia_exe);
        }
        assert_eq!((c.classic_exe.as_str(), c.classic_data_dir.as_str(), c.runtime_exe.as_str()), ("old.exe", "D", "new.exe"));
    }

    #[test]
    fn every_preset_names_its_folders() {
        for p in Preset::ALL {
            assert!(!p.switches().is_empty() && p.label().len() > 2);
        }
    }
}

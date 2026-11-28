//! HybridGt: a desktop launcher that finds your own copy of the game, prepares its shaders and
//! pipelines, and runs it on the bundled runtime (VIctory-NVIDIA with HybridGt's patch).
//!
//! Modules: `config` (settings), `library` (games, their art and playtime), `system` (this PC),
//! `deps` (what the runtime needs), `setup` (the one-click preparation), `jobs` (processes and
//! their progress), `presets` (switch sets), `diagnostics` (logs), `scan` (finding things),
//! `offline` (the network policy), `pages` and `style` (the interface).
#![windows_subsystem = "windows"]

mod config;
mod deps;
mod diagnostics;
mod jobs;
mod library;
mod mark;
mod offline;
mod pages;
mod presets;
mod scan;
mod setup;
mod style;
mod system;

use eframe::egui::{self, Align, Layout, RichText};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use config::Config;
use jobs::{Job, Kind, Live};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("HybridGt")
            .with_inner_size([1240.0, 780.0])
            .with_min_inner_size([1020.0, 660.0])
            .with_icon(icon()),
        centered: true,
        ..Default::default()
    };
    eframe::run_native("HybridGt", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}

/// HybridGt's logo (`tools/make_logo.py`), 256 pixels a side: the window icon and the brand.
pub const LOGO: &[u8] = include_bytes!("../assets/logo-256.png");

/// The window's icon: the logo (the executable's icon is the same, from `build.rs`), else the
/// drawn mark.
fn icon() -> egui::IconData {
    match library::decode_image(LOGO, 128) {
        Some(([w, h], rgba)) => egui::IconData { rgba, width: w as u32, height: h as u32 },
        None => {
            let (rgba, n) = mark_pixels(64);
            egui::IconData { rgba, width: n, height: n }
        }
    }
}

pub use mark::mark_pixels;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Page {
    Home,
    Library,
    Performance,
    Settings,
    Diagnostics,
}

/// Caches as measured on demand (walking a cache of thousands of files is not done every frame).
#[derive(Default, Clone)]
pub struct CacheStats {
    pub shaders: (u64, u64),
    pub pipelines: (u64, u64),
    pub measured: bool,
}

pub struct App {
    pub config: Config,
    pub sys: system::System,
    pub ram_free: u64,
    pub deps: Vec<deps::Check>,
    pub game: library::Game,
    pub splash: Option<egui::TextureHandle>,
    pub logo: Option<egui::TextureHandle>,
    /// HybridGt's own logo, for the sidebar.
    pub brand: Option<egui::TextureHandle>,
    pub playtime: library::Playtime,
    pub page: Page,
    pub job: Option<Job>,
    pub setup: Option<setup::Setup>,
    pub live: Option<Live>,
    pub message: String,
    pub scanning: Option<std::sync::mpsc::Receiver<scan::Found>>,
    pub scan_seen: Arc<AtomicUsize>,
    pub caches: Option<std::sync::mpsc::Receiver<CacheStats>>,
    pub cache_stats: CacheStats,
    pub log_text: String,
    pub log_summary: Option<diagnostics::LogSummary>,
    pub confirm_clear: bool,
    polled: Instant,
    played_tick: Instant,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        style::apply(&cc.egui_ctx);
        let mut config = Config::load();
        unpack_runtime(&mut config);
        let sys = system::read();
        let mut app = App {
            deps: deps::check(&config, &sys),
            game: library::read(&config.game_dir),
            ram_free: system::ram().1,
            config,
            sys,
            splash: None,
            logo: None,
            brand: None,
            playtime: library::Playtime::load(),
            page: Page::Home,
            job: None,
            setup: None,
            live: None,
            message: String::new(),
            scanning: None,
            scan_seen: Arc::default(),
            caches: None,
            cache_stats: CacheStats::default(),
            log_text: String::new(),
            log_summary: None,
            confirm_clear: false,
            polled: Instant::now(),
            played_tick: Instant::now(),
        };
        app.load_art(&cc.egui_ctx);
        if !scan::is_game_dir(Path::new(&app.config.game_dir)) || !Path::new(&app.config.runtime_exe).is_file() {
            app.start_scan();
        }
        app
    }

    /// The title's own images, from its folder.
    pub fn load_art(&mut self, ctx: &egui::Context) {
        let tex = |path: &Option<PathBuf>, name: &str, side: u32| {
            let (size, px) = library::load_image(path.as_ref()?, side)?;
            Some(ctx.load_texture(name, egui::ColorImage::from_rgba_unmultiplied(size, &px), egui::TextureOptions::LINEAR))
        };
        self.splash = tex(&self.game.splash, "splash", 1600);
        self.logo = tex(&self.game.logo, "logo", 320);
        if self.brand.is_none() {
            self.brand = library::decode_image(LOGO, 256)
                .map(|(size, px)| ctx.load_texture("hybridgt", egui::ColorImage::from_rgba_unmultiplied(size, &px), egui::TextureOptions::LINEAR));
        }
    }

    pub fn refresh(&mut self, ctx: &egui::Context) {
        self.game = library::read(&self.config.game_dir);
        self.deps = deps::check(&self.config, &self.sys);
        self.load_art(ctx);
    }

    pub fn start_scan(&mut self) {
        if self.scanning.is_some() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let seen = self.scan_seen.clone();
        seen.store(0, Ordering::Relaxed);
        std::thread::spawn(move || {
            let found = scan::scan(|n| seen.store(n, Ordering::Relaxed));
            let _ = tx.send(found);
        });
        self.scanning = Some(rx);
        self.message = "Looking for the game and the runtime on this PC...".into();
    }

    /// Takes what the scan found where nothing usable was chosen.
    fn take_scan(&mut self, found: scan::Found, ctx: &egui::Context) {
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
            (&mut c.streamline_dir, found.streamline_dir, defaults.streamline_dir),
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
        self.refresh(ctx);
    }

    /// Fetches NVIDIA's DLSS files (`get-dlss`), AMD's FSR 3 files (`get-fsr`) or the MFG
    /// Unlocker (`get-mfg`) by the runtime's own commands, which take them from their makers'
    /// GitHub and check them, into HybridGt's home. Only when pressed; the launcher itself has
    /// no network code.
    pub fn fetch(&mut self, command: &'static str) {
        if self.job.is_some() || self.setup.as_ref().is_some_and(setup::Setup::running) {
            self.message = "Something is already running.".into();
            return;
        }
        let exe = self.config.runtime_exe.clone();
        if !deps::is_ours(Path::new(&exe)) {
            self.message = "Set HybridGt's runtime first (Settings).".into();
            return;
        }
        let env = vec![("VICTORY_HOME".to_string(), config::app_dir().join("home").display().to_string())];
        match jobs::start(Kind::Download, &exe, &[command.to_string()], &env, None, Path::new(&self.config.logs_dir)) {
            Ok(job) => {
                self.message = "Downloading: the log is under Diagnostics.".into();
                self.job = Some(job);
            }
            Err(e) => self.message = e,
        }
    }

    pub fn play(&mut self) {
        if self.job.is_some() || self.setup.as_ref().is_some_and(setup::Setup::running) {
            self.message = "Something is already running.".into();
            return;
        }
        let problems = self.config.problems();
        if let Some(p) = problems.first() {
            self.message = p.clone();
            return;
        }
        let preset = self.config.preset;
        let (exe, _, _) = self.config.folders_for(preset);
        let exe = exe.to_string();
        let game_exe = Path::new(&self.config.game_dir).join(&self.game.executable).display().to_string();
        // Only HybridGt's own runtime is started (`deps::is_ours`).
        if !deps::is_ours(Path::new(&exe)) {
            self.message = "That victory.exe is not HybridGt's runtime: set HybridGt's under Settings, or build it.".into();
            return;
        }
        // The current runtime through `victory play`, so its F1 menu saves and applies its
        // settings; Classic as before.
        let (args, env) = if preset.classic() {
            let mut args = vec!["run".to_string(), game_exe, "--execute".to_string()];
            if let Some(console) = preset.console() {
                args.splice(2..2, ["--console".to_string(), console.to_string()]);
            }
            (args, presets::environment(&self.config, preset))
        } else {
            presets::play_command(&self.config, preset, &game_exe)
        };
        self.config.save();
        match jobs::start(Kind::Game, &exe, &args, &env, None, Path::new(&self.config.logs_dir)) {
            Ok(job) => {
                self.playtime.begin(&self.game_key());
                self.playtime.save();
                self.played_tick = Instant::now();
                self.live = None;
                self.job = Some(job);
                self.message = format!("{} started ({}).", self.game.name, preset.label());
            }
            Err(e) => self.message = format!("Could not start the game: {e}."),
        }
    }

    pub fn build(&mut self) {
        if self.job.is_some() {
            self.message = "Something is already running.".into();
            return;
        }
        let src = PathBuf::from(&self.config.source_dir);
        if !src.join("Cargo.toml").is_file() {
            self.message = "Set the runtime's source folder (with Cargo.toml) under Settings first.".into();
            return;
        }
        let Some(cargo) = deps::which("cargo").or_else(|| {
            let p = PathBuf::from(std::env::var_os("USERPROFILE")?).join(".cargo").join("bin").join("cargo.exe");
            p.is_file().then_some(p)
        }) else {
            self.message = "Rust (cargo) is not installed: building needs it.".into();
            return;
        };
        // `--offline`: a build never downloads; what is not cached is reported by cargo.
        let args: Vec<String> = ["build", "--release", "--offline", "-p", "victory"].iter().map(|s| s.to_string()).collect();
        match jobs::start(Kind::Build, &cargo.display().to_string(), &args, &[], Some(&src), Path::new(&self.config.logs_dir)) {
            Ok(job) => {
                self.job = Some(job);
                self.message = "Building the runtime (offline)...".into();
            }
            Err(e) => self.message = format!("Could not start the build: {e}."),
        }
    }

    pub fn start_setup(&mut self) {
        if self.job.is_some() {
            self.message = "Wait for what is running to end first.".into();
            return;
        }
        self.setup = Some(setup::Setup::new());
        self.page = Page::Home;
    }

    /// The key playtime is kept under: the title's identity.
    pub fn game_key(&self) -> String {
        if self.game.id.is_empty() { "game".into() } else { self.game.id.clone() }
    }

    pub fn measure_caches(&mut self) {
        if self.caches.is_some() {
            return;
        }
        let (_, cache, _) = self.config.folders_for(self.config.preset);
        let cache = PathBuf::from(cache);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let stats = CacheStats {
                shaders: system::folder_size(&cache.join("shaders")),
                pipelines: system::folder_size(&cache.join("pipelines")),
                measured: true,
            };
            let _ = tx.send(stats);
        });
        self.caches = Some(rx);
    }

    fn poll(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.scanning
            && let Ok(found) = rx.try_recv()
        {
            self.scanning = None;
            self.take_scan(found, ctx);
        }
        if let Some(rx) = &self.caches
            && let Ok(stats) = rx.try_recv()
        {
            self.cache_stats = stats;
            self.caches = None;
        }
        if self.polled.elapsed() < Duration::from_millis(500) {
            return;
        }
        self.polled = Instant::now();
        self.ram_free = system::ram().1;
        if let Some(s) = &mut self.setup
            && s.running()
        {
            let env = presets::environment(&self.config, self.config.preset);
            s.advance(&mut self.config, &self.sys, &env);
            if !s.running() {
                self.message = if s.ready() { "Environment ready.".into() } else { "Setup ended: see the steps for what is left.".into() };
                self.deps = deps::check(&self.config, &self.sys);
                self.cache_stats.measured = false;
            }
        }
        let mut ended = None;
        let key = self.game_key();
        if let Some(job) = &mut self.job {
            if job.kind == Kind::Game {
                let text = jobs::tail(&job.log, 96 * 1024);
                if let Some(l) = jobs::parse_live(&text) {
                    self.live = Some(l);
                }
                // Playtime counts while the game runs, saved every half minute.
                if self.played_tick.elapsed() >= Duration::from_secs(30) {
                    let secs = self.played_tick.elapsed().as_secs();
                    self.played_tick = Instant::now();
                    self.playtime.add(&key, secs);
                    self.playtime.save();
                }
            }
            if let Some(status) = job.ended() {
                ended = Some((job.kind, status.success()));
            }
        }
        if let Some((kind, ok)) = ended {
            if kind == Kind::Game {
                self.playtime.add(&key, self.played_tick.elapsed().as_secs());
                self.playtime.save();
            }
            if kind == Kind::Download {
                // What arrived: the folders taken where none is set.
                let (streamline, ffx) = scan::vendor_now(std::iter::once(Path::new(&self.config.runtime_exe)));
                for (slot, found) in [(&mut self.config.streamline_dir, streamline), (&mut self.config.ffx_dir, ffx)] {
                    if let Some(dir) = found
                        && slot.is_empty()
                    {
                        *slot = dir;
                    }
                }
                self.config.save();
            }
            if kind == Kind::Build && ok {
                let built = Path::new(&self.config.source_dir).join("target").join("release").join("victory.exe");
                if built.is_file() {
                    self.config.runtime_exe = built.display().to_string();
                    self.config.save();
                }
            }
            if let Some(job) = &self.job {
                self.log_summary = Some(diagnostics::summarize(&jobs::tail(&job.log, 4 << 20)));
            }
            self.message = format!("The {} ended{}.", kind.label(), if ok { "" } else { " with an error: see Diagnostics" });
            self.job = None;
            self.deps = deps::check(&self.config, &self.sys);
        }
        if self.page == Page::Diagnostics {
            let log = self.job.as_ref().map(|j| j.log.clone()).or_else(|| diagnostics::newest_log(Path::new(&self.config.logs_dir), "game"));
            if let Some(log) = log {
                let text = jobs::tail(&log, 64 * 1024);
                let lines: Vec<&str> = text.lines().collect();
                self.log_text = lines[lines.len().saturating_sub(250)..].join("\n");
            }
        }
    }

    /// One line on what is running, for the sidebar.
    fn activity(&self) -> Option<(String, Option<f32>)> {
        if let Some(s) = &self.setup
            && s.running()
        {
            return Some((format!("Preparing ({:.0} %)", s.fraction() * 100.0), Some(s.fraction())));
        }
        let job = self.job.as_ref()?;
        let secs = job.started.elapsed().as_secs();
        Some((format!("{} running, {}:{:02}", capitalized(job.kind.label()), secs / 60, secs % 60), None))
    }
}

fn capitalized(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().collect::<String>() + c.as_str())
}

/// With a runtime inside this executable (`HGT_RUNTIME_EXE` at build time), it is unpacked to
/// `%LOCALAPPDATA%\HybridGt\runtime\victory.exe` when missing or different, and used when no
/// other runtime is set.
fn unpack_runtime(config: &mut Config) {
    #[cfg(embedded_runtime)]
    {
        static RUNTIME: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/runtime.bin"));
        let dir = config::app_dir().join("runtime");
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

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll(ctx);
        let t = ctx.input(|i| i.time) as f32;
        let busy = self.job.is_some() || self.setup.as_ref().is_some_and(setup::Setup::running) || self.scanning.is_some();
        ctx.request_repaint_after(if self.config.animations || busy { Duration::from_millis(50) } else { Duration::from_secs(1) });

        egui::SidePanel::left("nav")
            .exact_width(230.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(style::PANEL).inner_margin(egui::Margin::symmetric(16.0, 18.0)))
            .show(ctx, |ui| {
                style::brand(ui, self.brand.as_ref());
                ui.add_space(14.0);
                ui.spacing_mut().item_spacing.y = 8.0;
                for (page, label, icon) in [
                    (Page::Home, "Home", "⌂"),
                    (Page::Library, "Library", "▦"),
                    (Page::Performance, "Performance", "⚡"),
                    (Page::Settings, "Settings", "⚙"),
                    (Page::Diagnostics, "Diagnostics", "⚕"),
                ] {
                    if style::nav_button(ui, &format!("{icon}   {label}"), self.page == page).clicked() {
                        self.page = page;
                        if page == Page::Performance {
                            self.measure_caches();
                        }
                    }
                }
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    ui.label(RichText::new(format!("HybridGt {}", env!("CARGO_PKG_VERSION"))).small().color(style::MUTED));
                    ui.horizontal(|ui| style::chip(ui, "● Offline", style::GOOD));
                    if let Some((what, f)) = self.activity() {
                        style::progress(ui, f, &what, t);
                    }
                    if !self.message.is_empty() {
                        ui.add(egui::Label::new(RichText::new(&self.message).small().color(style::SOFT)).wrap());
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(style::BACKGROUND).inner_margin(egui::Margin::same(26.0)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match self.page {
                    Page::Home => self.page_home(ui, t),
                    Page::Library => self.page_library(ui),
                    Page::Performance => self.page_performance(ui),
                    Page::Settings => self.page_settings(ui, ctx),
                    Page::Diagnostics => self.page_diagnostics(ui),
                });
            });
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_mark_is_round_with_clear_corners() {
        let (px, n) = super::mark_pixels(32);
        assert_eq!(px.len(), (n * n * 4) as usize);
        assert_eq!(px[3], 0, "the corner is clear");
        let centre = ((n / 4 * n + n / 2) * 4 + 3) as usize;
        assert_eq!(px[centre], 255, "the upper middle is the disc");
    }
}

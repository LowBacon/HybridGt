//! The five pages: Home, Library, Performance, Settings and Diagnostics. Every control does
//! what it says; statuses come from what is on disk or measured, never made up.

use eframe::egui::{self, Align, Color32, Layout, RichText, Vec2};
use std::path::Path;

use crate::deps::{Need, State};
use crate::library::{ago, duration};
use crate::presets::Preset;
use crate::setup::Status;
use crate::{App, Page, style, system};

fn label_value(ui: &mut egui::Ui, k: &str, v: &str) {
    ui.label(RichText::new(k).color(style::MUTED));
    ui.label(RichText::new(v).color(Color32::WHITE).strong());
    ui.end_row();
}

fn section(ui: &mut egui::Ui, text: &str) {
    ui.add_space(14.0);
    ui.label(RichText::new(text).size(17.0).strong().color(Color32::WHITE));
    ui.add_space(6.0);
}

fn status_look(s: &Status) -> (&'static str, Color32, String) {
    match s {
        Status::Pending => ("waiting", style::MUTED, String::new()),
        Status::Running(d, Some(t)) => ("running", style::VIOLET, format!("{d} of {t}")),
        Status::Running(d, None) => ("running", style::VIOLET, if *d > 0 { format!("{d} so far") } else { String::new() }),
        Status::Done(m) => ("done", style::GOOD, m.clone()),
        Status::NotApplicable(m) => ("not applicable", style::CYAN, m.clone()),
        Status::Blocked(m) => ("blocked", style::WARN, m.clone()),
        Status::Failed(m) => ("failed", style::STOP, m.clone()),
    }
}

impl App {
    fn shader_cache_state(&self) -> (String, Color32) {
        let (bytes, files) = self.cache_stats.shaders;
        if !self.cache_stats.measured {
            ("not measured yet".into(), style::MUTED)
        } else if files == 0 {
            ("not prepared".into(), style::WARN)
        } else {
            (format!("prepared: {} in {files} files", system::gb(bytes)), style::GOOD)
        }
    }

    fn pipeline_cache_state(&self) -> (String, Color32) {
        if self.config.preset.classic() {
            return ("built while playing (Classic)".into(), style::CYAN);
        }
        let (_, cache, _) = self.config.folders_for(self.config.preset);
        let dir = Path::new(cache).join("pipelines");
        let made = std::fs::read_to_string(dir.join("warm-done.txt")).map_or(0, |t| t.lines().count());
        if !dir.join("known-pipelines.bin").is_file() {
            ("needs one game session to record pipelines".into(), style::WARN)
        } else if made == 0 {
            ("recorded, not made ahead yet".into(), style::WARN)
        } else {
            (format!("{made} made ahead"), style::GOOD)
        }
    }

    fn setup_panel(&mut self, ui: &mut egui::Ui, t: f32) {
        let ready = self.setup.as_ref().is_some_and(|s| s.ready());
        let running = self.setup.as_ref().is_some_and(|s| s.running());
        style::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_max_width(ui.available_width() - 290.0);
                    let title = if ready { "Environment ready" } else { "Initialize the environment" };
                    ui.label(RichText::new(title).size(19.0).strong().color(Color32::WHITE));
                    ui.label(
                        RichText::new("Checks what the runtime needs, the game files and folders, prepares the shader cache and the pipeline cache, and writes a report. Nothing is downloaded.")
                            .color(style::SOFT),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if running {
                        if style::pill(ui, "Stop", false).clicked()
                            && let Some(s) = &mut self.setup
                        {
                            s.cancel();
                        }
                    } else if style::big_button(ui, if self.setup.is_some() { "RUN AGAIN" } else { "INITIALIZE" }, style::VIOLET).clicked() {
                        self.start_setup();
                    }
                });
            });
            if let Some(s) = &self.setup {
                ui.add_space(10.0);
                let f = s.fraction();
                let all_settled = !s.running();
                let text = if all_settled {
                    if s.ready() { "Every step done.".to_string() } else { "Every step settled; see those blocked or failed.".to_string() }
                } else {
                    format!("{:.0} % of the steps", f * 100.0)
                };
                style::progress(ui, Some(f), &text, t);
                ui.add_space(6.0);
                for (step, status) in &s.steps {
                    let (word, color, detail) = status_look(status);
                    style::status_row(ui, step.label(), &detail, word, color);
                }
                if let Some(r) = &s.report {
                    ui.label(RichText::new(format!("Report: {}", r.display())).small().color(style::MUTED));
                }
            }
        });
    }

    pub fn page_home(&mut self, ui: &mut egui::Ui, t: f32) {
        let rect = style::hero(ui, self.splash.as_ref(), t, self.config.animations, 300.0);
        let inner = rect.shrink2(Vec2::new(34.0, 30.0));
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_max_width(inner.width() - 230.0);
                    ui.add_space(10.0);
                    let name = if self.game.found { self.game.name.clone() } else { "No game folder chosen".into() };
                    ui.label(RichText::new(name).size(38.0).strong().color(Color32::WHITE));
                    if self.game.found {
                        ui.label(RichText::new(format!("Version {}  ·  {}", self.game.version, self.game.publisher)).size(15.0).color(style::SOFT));
                    } else {
                        ui.label(RichText::new("Settings → Game, or let the scan find it.").color(style::SOFT));
                    }
                    ui.add_space(12.0);
                    ui.horizontal_wrapped(|ui| {
                        if self.game.found {
                            style::chip(ui, "Installed", style::GOOD);
                        } else {
                            style::chip(ui, "Not found", style::WARN);
                        }
                        let blocked = self.deps.iter().filter(|c| c.blocks()).count();
                        if blocked == 0 {
                            style::chip(ui, "Requirements met", style::GOOD);
                        } else {
                            style::chip(ui, &format!("{blocked} requirement(s) missing"), style::STOP);
                        }
                        style::chip(ui, self.config.preset.label(), style::VIOLET);
                    });
                });
                ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                    let running = self.job.as_ref().is_some_and(|j| j.kind == crate::jobs::Kind::Game);
                    if running {
                        if style::big_button(ui, "STOP", style::STOP).clicked()
                            && let Some(job) = &mut self.job
                        {
                            job.stop();
                        }
                    } else if style::big_button(ui, "PLAY", style::ACCENT).clicked() {
                        self.play();
                    }
                });
            });
        });
        ui.add_space(16.0);
        let played = self.playtime.of(&self.game_key());
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(12.0, 12.0);
            style::stat(ui, "Last played", &played.last.map(ago).unwrap_or_else(|| "never".into()));
            style::stat(ui, "Time played", &duration(played.seconds));
            style::stat(ui, "Version", if self.game.version.is_empty() { "—" } else { &self.game.version });
            style::stat(ui, "Runtime", if Path::new(self.config.folders_for(self.config.preset).0).is_file() { "ready" } else { "missing" });
            if style::pill(ui, "Open library  ›", false).clicked() {
                self.page = Page::Library;
                self.measure_caches();
            }
        });
        ui.add_space(16.0);
        self.setup_panel(ui, t);
        section(ui, "How to play");
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(12.0, 12.0);
            for preset in Preset::ALL {
                if style::preset_card(ui, preset.label(), preset.blurb(), self.config.preset == preset).clicked() {
                    self.config.preset = preset;
                    self.config.save();
                    self.deps = crate::deps::check(&self.config, &self.sys);
                    self.cache_stats.measured = false;
                }
            }
        });
    }

    pub fn page_library(&mut self, ui: &mut egui::Ui) {
        style::title(ui, "Library");
        if !self.cache_stats.measured {
            self.measure_caches();
        }
        style::card(ui, |ui| {
            if !self.game.found {
                ui.label(RichText::new("No game yet. Choose its folder under Settings, or scan this PC.").color(style::SOFT));
                if style::pill(ui, "Scan this PC", false).clicked() {
                    self.start_scan();
                }
                return;
            }
            ui.horizontal(|ui| {
                let side = 190.0;
                match &self.logo {
                    Some(tex) => {
                        ui.add(egui::Image::new(tex).fit_to_exact_size(Vec2::splat(side)).rounding(egui::Rounding::same(18.0)));
                    }
                    None => {
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(side), egui::Sense::hover());
                        style::mark(ui.painter(), r.center(), side * 0.35);
                    }
                }
                ui.add_space(18.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(&self.game.name).size(26.0).strong().color(Color32::WHITE));
                    ui.label(RichText::new(&self.game.publisher).color(style::SOFT));
                    ui.add_space(8.0);
                    let played = self.playtime.of(&self.game_key());
                    egui::Grid::new("lib").num_columns(2).spacing([26.0, 7.0]).show(ui, |ui| {
                        label_value(ui, "Location", &self.game.dir.display().to_string());
                        label_value(ui, "Version", &self.game.version);
                        label_value(ui, "Size", &format!("{} in {} archives", system::gb(self.game.bytes), self.game.archives));
                        label_value(ui, "Time played", &format!("{} over {} sessions", duration(played.seconds), played.sessions));
                        label_value(ui, "Last played", &played.last.map(ago).unwrap_or_else(|| "never".into()));
                    });
                });
            });
            ui.add_space(12.0);
            let (shaders, sc) = self.shader_cache_state();
            let (pipes, pc) = self.pipeline_cache_state();
            let gpu = self.sys.gpus.iter().max_by_key(|g| g.vram.unwrap_or(0));
            let (compat, cc) = match gpu {
                Some(g) if g.vendor() == "NVIDIA" => (format!("{}: tested", g.name), style::GOOD),
                Some(g) if g.vendor() == "AMD" => (format!("{}: expected to run, not yet tested", g.name), style::WARN),
                Some(g) => (format!("{}: not tested", g.name), style::WARN),
                None => ("no graphics card found".into(), style::STOP),
            };
            style::status_row(ui, "Installation", &self.game.dir.display().to_string(), "installed", style::GOOD);
            style::status_row(ui, "Shader cache", &shaders, if sc == style::GOOD { "ready" } else { "to prepare" }, sc);
            style::status_row(ui, "Pipeline cache", &pipes, if pc == style::GOOD { "ready" } else if pc == style::CYAN { "n/a" } else { "pending" }, pc);
            style::status_row(ui, "Assets", "streamed from the game's archives as it plays (no separate cache)", "n/a", style::CYAN);
            style::status_row(ui, "Compatibility", &compat, if cc == style::GOOD { "tested" } else { "untested" }, cc);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if style::pill(ui, "▶  Play", true).clicked() {
                    self.play();
                }
                if style::pill(ui, "Prepare environment", false).clicked() {
                    self.start_setup();
                }
                if style::pill(ui, "Settings", false).clicked() {
                    self.page = Page::Settings;
                }
            });
        });
        ui.add_space(8.0);
        ui.label(RichText::new("What is shown about a game is read from its own folder on this PC; HybridGt carries nothing of it. More titles appear here as they are added.").small().color(style::MUTED));
    }

    pub fn page_performance(&mut self, ui: &mut egui::Ui) {
        style::title(ui, "Performance");
        let (total, free) = (self.sys.ram_total, self.ram_free);
        ui.columns(2, |cols| {
            style::card(&mut cols[0], |ui| {
                ui.label(RichText::new("This PC").size(17.0).strong().color(Color32::WHITE));
                ui.add_space(6.0);
                egui::Grid::new("pc").num_columns(2).spacing([20.0, 7.0]).show(ui, |ui| {
                    label_value(ui, "Processor", &format!("{} ({} threads)", self.sys.cpu, self.sys.threads));
                    for g in &self.sys.gpus {
                        label_value(ui, "Graphics", &format!("{}{}", g.name, g.vram.map_or(String::new(), |v| format!(" · {}", system::gb(v)))));
                    }
                    label_value(ui, "Memory", &format!("{} total, {} free now", system::gb(total), system::gb(free)));
                    label_value(ui, "Windows", &self.sys.windows);
                });
                ui.add_space(6.0);
                let used = if total > 0 { 1.0 - free as f32 / total as f32 } else { 0.0 };
                style::progress(ui, Some(used), &format!("{:.0} % of memory in use", used * 100.0), 0.0);
            });
            style::card(&mut cols[1], |ui| {
                ui.label(RichText::new("Graphics configuration").size(17.0).strong().color(Color32::WHITE));
                ui.add_space(6.0);
                let p = self.config.preset;
                let profile = match p.console() {
                    Some("series-s") => "Series S profile: 720p inside",
                    Some(_) => "Series X profile: native 1440p",
                    None => "Classic: the game's choice",
                };
                let upscaler = if p.fsr() {
                    "AMD FSR 3.1"
                } else if self.config.dlss && !p.classic() {
                    "NVIDIA DLSS (if its files are set)"
                } else {
                    "the game's own"
                };
                egui::Grid::new("gfx").num_columns(2).spacing([20.0, 7.0]).show(ui, |ui| {
                    label_value(ui, "Preset", p.label());
                    label_value(ui, "Rendering", profile);
                    label_value(ui, "Upscaling", upscaler);
                    label_value(ui, "Ray tracing", if matches!(p, Preset::RayTracing | Preset::AllInOne) { "the game's own (experimental)" } else { "off; light estimated" });
                    label_value(ui, "Memory", ["Auto", "Balanced", "Low"][usize::from(self.config.memory.min(2))]);
                    label_value(ui, "Frame cap", &if self.config.fps_cap > 0 { format!("{} a second", self.config.fps_cap) } else { "the game's own pacing".into() });
                });
            });
        });
        ui.add_space(12.0);
        style::card(ui, |ui| {
            ui.label(RichText::new("Rendering compatibility and caches").size(17.0).strong().color(Color32::WHITE));
            ui.add_space(6.0);
            let d3d = self.deps.iter().find(|c| c.name.starts_with("Direct3D")).is_some_and(|c| c.state == State::Ok);
            style::status_row(ui, "Direct3D 12", "the runtime draws through Direct3D 12", if d3d { "available" } else { "missing" }, if d3d { style::GOOD } else { style::STOP });
            let (s, sc) = self.shader_cache_state();
            style::status_row(ui, "Shader cache", &s, if sc == style::GOOD { "ready" } else { "to prepare" }, sc);
            let (pp, pc) = self.pipeline_cache_state();
            style::status_row(ui, "Pipeline cache", &pp, if pc == style::GOOD { "ready" } else { "pending" }, pc);
            if self.cache_stats.measured {
                let (b, f) = self.cache_stats.pipelines;
                ui.label(RichText::new(format!("Pipeline files: {} in {f} files", system::gb(b))).small().color(style::MUTED));
            }
            if let Some(total) = self.config.shader_total {
                ui.label(RichText::new(format!("The last preparation counted {total} shaders.")).small().color(style::MUTED));
            }
            if style::pill(ui, "Measure caches again", false).clicked() {
                self.cache_stats.measured = false;
                self.measure_caches();
            }
        });
        ui.add_space(8.0);
        ui.label(RichText::new("Frame rates and frame times of a session are under Diagnostics.").small().color(style::MUTED));
    }

    pub fn page_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        style::title(ui, "Settings");
        let mut changed = false;
        let mut refresh = false;
        style::card(ui, |ui| {
            ui.label(RichText::new("Game").size(17.0).strong().color(Color32::WHITE));
            ui.label(RichText::new("The folder of your own copy (with game_scarlett_master.exe)").color(style::SOFT));
            if style::path_field(ui, &mut self.config.game_dir) {
                changed = true;
                refresh = true;
            }
            ui.horizontal(|ui| {
                if self.scanning.is_some() {
                    ui.add(egui::Spinner::new().color(style::ACCENT));
                    ui.label(RichText::new(format!("Scanning... {} folders", self.scan_seen.load(std::sync::atomic::Ordering::Relaxed))).color(style::SOFT));
                } else if style::pill(ui, "Scan this PC", false).clicked() {
                    self.start_scan();
                }
            });
        });
        ui.add_space(12.0);
        style::card(ui, |ui| {
            ui.label(RichText::new("Graphics").size(17.0).strong().color(Color32::WHITE));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_max_width(ui.available_width() - 330.0);
                    ui.label(RichText::new("Memory").strong().color(Color32::WHITE));
                    ui.label(RichText::new("Auto fills the card with textures up to nine tenths of its memory. Balanced and Low cap that: less memory, more textures arriving late.").color(style::SOFT));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    for (level, name) in [(2u8, "Low"), (1, "Balanced"), (0, "Auto")] {
                        if style::pill(ui, name, self.config.memory == level).clicked() {
                            self.config.memory = level;
                            changed = true;
                        }
                    }
                });
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_max_width(ui.available_width() - 330.0);
                    ui.label(RichText::new("Frame cap").strong().color(Color32::WHITE));
                    ui.label(RichText::new("A cap lets the processor and card rest between frames.").color(style::SOFT));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    for (fps, name) in [(60u32, "60"), (30, "30"), (0, "Game's own")] {
                        if style::pill(ui, name, self.config.fps_cap == fps).clicked() {
                            self.config.fps_cap = fps;
                            changed = true;
                        }
                    }
                });
            });
            ui.add_space(6.0);
            changed |= style::toggle_row(ui, "NVIDIA DLSS", "Upscaling through NVIDIA Streamline, from the folder set below (Series X presets).", &mut self.config.dlss);
            changed |= style::toggle_row(ui, "Estimated light", "Lights rooms and shaded sides where the game's ray tracing is off.", &mut self.config.ambient);
            changed |= style::toggle_row(ui, "In-place memory (Classic)", "Opens the game's memory to the card directly.", &mut self.config.inplace);
            changed |= style::toggle_row(ui, "Wide hashing (Classic)", "Hashes copies on the card 64 threads a piece.", &mut self.config.hash_wide);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_max_width(ui.available_width() - 330.0);
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
            ui.label(RichText::new("Runtimes and folders").size(17.0).strong().color(Color32::WHITE));
            for (label, value) in [
                ("Runtime (victory.exe)", &mut self.config.runtime_exe),
                ("Classic runtime (optional)", &mut self.config.classic_exe),
                ("Shader and pipeline cache", &mut self.config.cache_dir),
                ("Saves and settings", &mut self.config.data_dir),
                ("Classic cache", &mut self.config.classic_cache_dir),
                ("Classic saves", &mut self.config.classic_data_dir),
                ("Logs", &mut self.config.logs_dir),
                ("AMD FidelityFX DLLs (FSR 3.1)", &mut self.config.ffx_dir),
                ("NVIDIA Streamline (DLSS)", &mut self.config.streamline_dir),
                ("Runtime source (to build it)", &mut self.config.source_dir),
            ] {
                ui.label(RichText::new(label).color(style::SOFT));
                if style::path_field(ui, value) {
                    changed = true;
                    refresh = true;
                }
            }
            ui.add_space(4.0);
            if style::pill(ui, "Build the runtime from source (offline)", false).clicked() {
                self.build();
            }
        });
        ui.add_space(12.0);
        style::card(ui, |ui| {
            ui.label(RichText::new("DLSS, FSR 3 and frame generation files").size(17.0).strong().color(Color32::WHITE));
            ui.label(
                RichText::new("Not part of HybridGt. Each button has the runtime fetch them from their makers' own GitHub, check them, and keep them in HybridGt's folder. Nothing is fetched until you press one.")
                    .color(style::SOFT),
            );
            let have = |dir: &str, file: &str| !dir.is_empty() && Path::new(dir).join(file).is_file();
            let dlss = have(&self.config.streamline_dir, "sl.interposer.dll");
            let fsr = have(&self.config.ffx_dir, "amd_fidelityfx_loader_dx12.dll");
            style::status_row(ui, "NVIDIA DLSS and frame generation", "NVIDIA Streamline, about 70 MB", if dlss { "present" } else { "not yet" }, if dlss { style::GOOD } else { style::MUTED });
            style::status_row(ui, "AMD FSR 3 frame generation", "AMD FidelityFX SDK, about 30 MB", if fsr { "present" } else { "not yet" }, if fsr { style::GOOD } else { style::MUTED });
            ui.horizontal_wrapped(|ui| {
                if style::pill(ui, "Get NVIDIA DLSS", false).clicked() {
                    self.fetch("get-dlss");
                }
                if style::pill(ui, "Get AMD FSR 3", false).clicked() {
                    self.fetch("get-fsr");
                }
                if style::pill(ui, "Get MFG Unlocker (mod)", false).clicked() {
                    self.fetch("get-mfg");
                }
            });
            ui.label(
                RichText::new("MFG Unlocker is someone else's mod (MIT) that lets DLSS frame generation run on cards NVIDIA does not support it on, RTX 20 and 30. RTX 40 cards do 2x without it. Turn it on in the game's F1 menu. Frame generation multiplies the frames shown, not the game's speed.")
                    .small()
                    .color(style::MUTED),
            );
        });
        ui.add_space(12.0);
        style::card(ui, |ui| {
            ui.label(RichText::new("Logging and appearance").size(17.0).strong().color(Color32::WHITE));
            ui.horizontal(|ui| {
                ui.label(RichText::new("Runtime log").color(style::SOFT));
                for level in ["warn", "info", "debug"] {
                    if style::pill(ui, level, self.config.log_level == level).clicked() {
                        self.config.log_level = level.into();
                        changed = true;
                    }
                }
            });
            changed |= style::toggle_row(ui, "Animations", "The banner's drift and the hover motion.", &mut self.config.animations);
        });
        ui.add_space(12.0);
        style::card(ui, |ui| {
            ui.label(RichText::new("Offline").size(17.0).strong().color(Color32::WHITE));
            for line in crate::offline::POLICY {
                ui.label(RichText::new(format!("•  {line}")).color(style::SOFT));
            }
        });
        ui.add_space(12.0);
        style::card(ui, |ui| {
            ui.label(RichText::new("Reset and repair").size(17.0).strong().color(Color32::WHITE));
            ui.horizontal_wrapped(|ui| {
                if style::pill(ui, "Run the setup again", false).clicked() {
                    self.start_setup();
                }
                if style::pill(ui, "Reset preferences (keeps folders)", false).clicked() {
                    let keep = self.config.clone();
                    self.config = crate::config::Config {
                        game_dir: keep.game_dir,
                        runtime_exe: keep.runtime_exe,
                        classic_exe: keep.classic_exe,
                        classic_cache_dir: keep.classic_cache_dir,
                        classic_data_dir: keep.classic_data_dir,
                        source_dir: keep.source_dir,
                        cache_dir: keep.cache_dir,
                        data_dir: keep.data_dir,
                        logs_dir: keep.logs_dir,
                        ffx_dir: keep.ffx_dir,
                        streamline_dir: keep.streamline_dir,
                        shader_total: keep.shader_total,
                        ..crate::config::Config::default()
                    };
                    changed = true;
                }
                let clear = if self.confirm_clear { "Click again: delete the shader cache" } else { "Clear the shader cache" };
                if style::pill(ui, clear, self.confirm_clear).clicked() {
                    if self.confirm_clear {
                        let (_, cache, _) = self.config.folders_for(self.config.preset);
                        let shaders = Path::new(cache).join("shaders");
                        self.message = match std::fs::remove_dir_all(&shaders) {
                            Ok(()) => "Shader cache cleared: prepare it again before playing.".into(),
                            Err(e) => format!("Could not clear the shader cache: {e}"),
                        };
                        self.cache_stats.measured = false;
                        self.confirm_clear = false;
                    } else {
                        self.confirm_clear = true;
                    }
                }
            });
        });
        if changed {
            self.config.save();
            self.deps = crate::deps::check(&self.config, &self.sys);
        }
        if refresh {
            self.refresh(ctx);
        }
    }

    pub fn page_diagnostics(&mut self, ui: &mut egui::Ui) {
        style::title(ui, "Diagnostics");
        style::card(ui, |ui| {
            ui.label(RichText::new("Requirements").size(17.0).strong().color(Color32::WHITE));
            ui.add_space(4.0);
            for c in &self.deps {
                let (word, color) = match (c.state, c.need) {
                    (State::Ok, _) => ("ok", style::GOOD),
                    (_, Need::Required) => ("missing", style::STOP),
                    (State::Low, _) => ("low", style::WARN),
                    (_, Need::Recommended) => ("recommended", style::WARN),
                    (_, Need::Optional) => ("optional", style::MUTED),
                };
                let detail = if c.state == State::Ok { c.detail.clone() } else { format!("{} — {}", c.detail, c.fix) };
                style::status_row(ui, c.name, &detail, word, color);
            }
            for p in self.config.problems() {
                ui.label(RichText::new(format!("•  {p}")).color(style::WARN));
            }
        });
        if let Some(s) = &self.setup {
            ui.add_space(12.0);
            style::card(ui, |ui| {
                ui.label(RichText::new("Last setup").size(17.0).strong().color(Color32::WHITE));
                for (step, status) in &s.steps {
                    let (word, color, detail) = status_look(status);
                    style::status_row(ui, step.label(), &detail, word, color);
                }
            });
        }
        if let (Some(live), true) = (self.live, self.job.as_ref().is_some_and(|j| j.kind == crate::jobs::Kind::Game)) {
            ui.add_space(12.0);
            style::card(ui, |ui| {
                ui.label(RichText::new("This session").size(17.0).strong().color(Color32::WHITE));
                ui.horizontal(|ui| {
                    style::stat(ui, "Frames a second", &format!("{:.1}", live.fps));
                    style::stat(ui, "Frame time", &format!("{:.0} ms", live.ms));
                    style::stat(ui, "Draws a frame", &format!("{}", live.draws));
                });
            });
        }
        if let Some(sum) = &self.log_summary {
            ui.add_space(12.0);
            style::card(ui, |ui| {
                ui.label(RichText::new("Last run's log").size(17.0).strong().color(Color32::WHITE));
                ui.label(RichText::new(format!("{} errors, {} warnings", sum.errors, sum.warnings)).color(style::SOFT));
                if let Some(c) = &sum.crashed {
                    ui.label(RichText::new(format!("Ended by: {c}")).color(style::STOP));
                }
                for (n, level, msg) in &sum.top {
                    ui.label(RichText::new(format!("{n:>6} × {level}: {msg}")).monospace().size(11.5).color(if *level == "error" { style::STOP } else { style::SOFT }));
                }
            });
        }
        ui.add_space(12.0);
        style::card(ui, |ui| {
            ui.label(RichText::new("Log").size(17.0).strong().color(Color32::WHITE));
            if self.log_text.is_empty() {
                ui.label(RichText::new("No log yet.").color(style::SOFT));
            } else {
                egui::ScrollArea::vertical().max_height(360.0).stick_to_bottom(true).show(ui, |ui| {
                    ui.add(egui::Label::new(RichText::new(&self.log_text).monospace().size(11.0).color(style::SOFT)).wrap());
                });
            }
        });
    }
}

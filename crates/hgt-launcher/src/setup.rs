//! One click from nothing to ready: the stages run in order, each says what it really did, and
//! the whole is complete only when every stage finished, or was classified as not applicable or
//! blocked with its reason. Nothing is downloaded: what is missing is reported.

use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::jobs::{self, Job, Kind};
use crate::system::System;

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Pending,
    /// `(done, total)`: a total only when one is really known.
    Running(u64, Option<u64>),
    Done(String),
    /// Not needed or not possible on this runtime, with why: does not hold the rest up.
    NotApplicable(String),
    /// Could not be done now, with why and what would unblock it.
    Blocked(String),
    Failed(String),
}

impl Status {
    /// Finished one way or another.
    pub fn settled(&self) -> bool {
        !matches!(self, Status::Pending | Status::Running(..))
    }

    pub fn complete(&self) -> bool {
        matches!(self, Status::Done(_) | Status::NotApplicable(_))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Dependencies,
    GameFiles,
    Configuration,
    Shaders,
    Pipelines,
    Assets,
    Compatibility,
    Integrity,
}

impl Step {
    pub const ALL: [Step; 8] = [
        Step::Dependencies,
        Step::GameFiles,
        Step::Configuration,
        Step::Shaders,
        Step::Pipelines,
        Step::Assets,
        Step::Compatibility,
        Step::Integrity,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Step::Dependencies => "Dependencies",
            Step::GameFiles => "Game files",
            Step::Configuration => "Configuration",
            Step::Shaders => "Shader cache",
            Step::Pipelines => "Pipeline cache",
            Step::Assets => "Asset cache",
            Step::Compatibility => "Graphics compatibility",
            Step::Integrity => "Integrity check",
        }
    }
}

pub struct Setup {
    pub steps: Vec<(Step, Status)>,
    pub job: Option<Job>,
    pub report: Option<PathBuf>,
}

impl Setup {
    pub fn new() -> Self {
        Setup { steps: Step::ALL.iter().map(|&s| (s, Status::Pending)).collect(), job: None, report: None }
    }

    pub fn running(&self) -> bool {
        self.steps.iter().any(|(_, s)| !s.settled())
    }

    /// The share of the work done, from real counts: settled stages, plus the running one's
    /// known fraction. 1.0 only when every stage is settled.
    pub fn fraction(&self) -> f32 {
        let n = self.steps.len() as f32;
        let mut done = 0.0;
        for (_, s) in &self.steps {
            done += match s {
                Status::Running(d, Some(t)) if *t > 0 => (*d as f32 / *t as f32).min(0.99),
                Status::Running(..) | Status::Pending => 0.0,
                _ => 1.0,
            };
        }
        done / n
    }

    /// Every stage done or not applicable: the environment is ready.
    pub fn ready(&self) -> bool {
        self.steps.iter().all(|(_, s)| s.complete())
    }

    fn set(&mut self, step: Step, status: Status) {
        if let Some(s) = self.steps.iter_mut().find(|(k, _)| *k == step) {
            s.1 = status;
        }
    }

    fn current(&self) -> Option<Step> {
        self.steps.iter().find(|(_, s)| !s.settled()).map(|(k, _)| *k)
    }

    /// Moves the work on: called every frame or so by the window. `env` is the run's switches
    /// (the cache folder among them).
    pub fn advance(&mut self, config: &mut Config, sys: &System, env: &[(String, String)]) {
        let Some(step) = self.current() else { return };
        // A running stage: follow its process.
        if let Some(job) = &mut self.job {
            let text = jobs::tail(&job.log, 256 * 1024);
            let ended = job.ended();
            match step {
                Step::Shaders => {
                    let p = jobs::parse_shaders(&text);
                    match ended {
                        None => self.set(step, Status::Running(p.done, config.shader_total)),
                        Some(st) if st.success() && p.finished => {
                            config.shader_total = Some(p.done + p.failed);
                            config.save();
                            let mut msg = format!("{} shaders translated and cached", p.done);
                            if p.failed > 0 {
                                msg.push_str(&format!("; {} the translator does not support yet (listed in the report)", p.failed));
                            }
                            self.set(step, Status::Done(msg));
                            self.job = None;
                        }
                        Some(st) => {
                            self.set(step, Status::Failed(format!("the runtime stopped ({st}); see {}", short(&self.job_log()))));
                            self.job = None;
                        }
                    }
                }
                Step::Pipelines => {
                    let p = jobs::parse_pipelines(&text);
                    match ended {
                        None => self.set(step, Status::Running(p.done, p.to_make)),
                        Some(st) if st.success() => {
                            let msg = if p.to_make == Some(0) {
                                format!("all {} known pipelines were made before", p.known)
                            } else {
                                format!("{} pipelines made for the driver's cache ({} refused by the card)", p.done, p.refused)
                            };
                            self.set(step, Status::Done(msg));
                            self.job = None;
                        }
                        Some(st) => {
                            self.set(step, Status::Failed(format!("the runtime stopped ({st}); see {}", short(&self.job_log()))));
                            self.job = None;
                        }
                    }
                }
                _ => self.job = None,
            }
            return;
        }
        // Start the next stage.
        let (exe, cache, data) = config.folders_for(config.preset);
        let (exe, cache, data) = (exe.to_string(), PathBuf::from(cache), PathBuf::from(data));
        let logs = PathBuf::from(&config.logs_dir);
        match step {
            Step::Dependencies => {
                let checks = crate::deps::check(config, sys);
                let blocking: Vec<&str> = checks.iter().filter(|c| c.blocks()).map(|c| c.name).collect();
                if blocking.is_empty() {
                    let optional = checks.iter().filter(|c| c.state != crate::deps::State::Ok).count();
                    self.set(step, Status::Done(format!("all required present; {optional} optional or recommended missing (see Diagnostics)")));
                } else {
                    self.set(step, Status::Blocked(format!("missing: {} (see Diagnostics for what to do)", blocking.join(", "))));
                    self.stop_rest("waits for the dependencies");
                }
            }
            Step::GameFiles => {
                let g = crate::library::read(&config.game_dir);
                let shaders = Path::new(&config.game_dir).join("shaders.rpf").is_file();
                if !g.found {
                    self.set(step, Status::Blocked("no game in the chosen folder".into()));
                    self.stop_rest("waits for the game folder");
                } else if !shaders || g.archives == 0 {
                    self.set(step, Status::Blocked("the game folder lacks its archives (shaders.rpf): is the copy complete?".into()));
                    self.stop_rest("waits for complete game files");
                } else {
                    self.set(step, Status::Done(format!("{} {} : {} archives, {}", g.name, g.version, g.archives, crate::system::gb(g.bytes))));
                }
            }
            Step::Configuration => {
                let mut made = Vec::new();
                for dir in [&cache, &data, &logs] {
                    if std::fs::create_dir_all(dir).is_err() {
                        made.push(dir.display().to_string());
                    }
                }
                config.save();
                if made.is_empty() {
                    self.set(step, Status::Done(format!("folders ready; settings saved to {}", short(&crate::config::config_path()))));
                } else {
                    self.set(step, Status::Blocked(format!("cannot make {}", made.join(", "))));
                    self.stop_rest("waits for writable folders");
                }
            }
            Step::Shaders => {
                let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).saturating_sub(2).max(1);
                let args = vec!["shaders".into(), config.game_dir.clone(), "--jobs".into(), threads.to_string()];
                match jobs::start(Kind::Shaders, &exe, &args, env, None, &logs) {
                    Ok(job) => {
                        self.job = Some(job);
                        self.set(step, Status::Running(0, config.shader_total));
                    }
                    Err(e) => {
                        self.set(step, Status::Failed(e));
                        self.stop_rest("waits for the shader cache");
                    }
                }
            }
            Step::Pipelines => {
                if config.preset.classic() {
                    self.set(step, Status::NotApplicable("the Classic runtime has no pipeline warm-up; it builds pipelines while playing".into()));
                } else if !cache.join("pipelines").join("known-pipelines.bin").is_file() {
                    self.set(
                        step,
                        Status::Blocked(
                            "needs one game session first: the game writes down the pipelines it makes, then this step makes them ahead (run setup again after playing once)".into(),
                        ),
                    );
                } else {
                    let args = vec!["warm".to_string()];
                    match jobs::start(Kind::Pipelines, &exe, &args, env, None, &logs) {
                        Ok(job) => {
                            self.job = Some(job);
                            self.set(step, Status::Running(0, None));
                        }
                        Err(e) => self.set(step, Status::Failed(e)),
                    }
                }
            }
            Step::Assets => {
                self.set(
                    step,
                    Status::NotApplicable(
                        "the runtime streams the game's assets from its archives as the game asks (DirectStorage); there is no separate asset cache to build".into(),
                    ),
                );
            }
            Step::Compatibility => {
                let gpu = sys.gpus.iter().max_by_key(|g| g.vram.unwrap_or(0));
                let msg = match gpu {
                    Some(g) if g.vendor() == "NVIDIA" => format!("{}: the runtime's tested vendor", g.name),
                    Some(g) if g.vendor() == "AMD" => format!("{}: supported by its base (foxhound was made on an RX 7900 XTX); not yet tested with this build", g.name),
                    Some(g) => format!("{}: not tested", g.name),
                    None => "no graphics card found".into(),
                };
                if gpu.is_some() {
                    self.set(step, Status::Done(msg));
                } else {
                    self.set(step, Status::Blocked(msg));
                }
            }
            Step::Integrity => {
                let (bytes, files) = crate::system::folder_size(&cache.join("shaders"));
                let game_ok = crate::scan::is_game_dir(Path::new(&config.game_dir));
                if files > 0 && game_ok {
                    self.set(step, Status::Done(format!("shader cache {} in {files} files; game files in place; settings saved", crate::system::gb(bytes))));
                } else if !game_ok {
                    self.set(step, Status::Failed("the game folder changed during setup".into()));
                } else {
                    self.set(step, Status::Failed("the shader cache is empty".into()));
                }
                self.report = write_report(self, config).ok();
            }
        }
    }

    fn job_log(&self) -> PathBuf {
        self.job.as_ref().map(|j| j.log.clone()).unwrap_or_default()
    }

    /// Marks every stage after the current one as blocked by it.
    /// (Stages run in order, so the unsettled ones are those after it.)
    fn stop_rest(&mut self, why: &str) {
        for (_, s) in &mut self.steps {
            if !s.settled() {
                *s = Status::Blocked(why.into());
            }
        }
    }

    pub fn cancel(&mut self) {
        if let Some(job) = &mut self.job {
            job.stop();
        }
        self.job = None;
        for (_, s) in &mut self.steps {
            if !s.settled() {
                *s = Status::Blocked("stopped".into());
            }
        }
    }
}

fn short(p: &Path) -> String {
    p.file_name().map_or_else(|| p.display().to_string(), |n| n.to_string_lossy().into_owned())
}

/// The setup report, beside the logs.
pub fn write_report(setup: &Setup, config: &Config) -> std::io::Result<PathBuf> {
    let dir = PathBuf::from(&config.logs_dir);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("setup-report-{}.txt", crate::library::now()));
    let mut text = String::from("HybridGt setup report\n\n");
    for (step, status) in &setup.steps {
        let (word, detail) = match status {
            Status::Done(d) => ("done", d.as_str()),
            Status::NotApplicable(d) => ("not applicable", d.as_str()),
            Status::Blocked(d) => ("blocked", d.as_str()),
            Status::Failed(d) => ("failed", d.as_str()),
            Status::Pending | Status::Running(..) => ("not finished", ""),
        };
        text.push_str(&format!("{:<24} {:<15} {}\n", step.label(), word, detail));
    }
    text.push_str(&format!("\nready: {}\n", if setup.ready() { "yes" } else { "no" }));
    std::fs::write(&path, text)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_is_real_and_reaches_one_only_when_all_settle() {
        let mut s = Setup::new();
        assert_eq!(s.fraction(), 0.0);
        s.steps[0].1 = Status::Done("x".into());
        s.steps[1].1 = Status::Running(500, Some(1000));
        let f = s.fraction();
        assert!((f - 1.5 / 8.0).abs() < 1e-6, "{f}");
        // An unknown total counts as nothing yet, never a guess.
        s.steps[1].1 = Status::Running(500, None);
        assert!((s.fraction() - 1.0 / 8.0).abs() < 1e-6);
        for (_, st) in &mut s.steps {
            *st = Status::Done("x".into());
        }
        s.steps[4].1 = Status::Blocked("needs a session".into());
        assert_eq!(s.fraction(), 1.0, "every stage settled");
        assert!(!s.ready(), "but a blocked stage is not ready");
        s.steps[4].1 = Status::NotApplicable("n/a".into());
        assert!(s.ready());
    }

    #[test]
    fn a_blocked_stage_blocks_those_after_it() {
        let mut s = Setup::new();
        s.steps[0].1 = Status::Done("x".into());
        s.steps[1].1 = Status::Blocked("no game".into());
        s.stop_rest("waits for the game folder");
        assert!(s.steps[2..].iter().all(|(_, st)| matches!(st, Status::Blocked(w) if w.contains("waits"))));
        assert!(!s.running());
    }

    #[test]
    fn a_blank_pc_setup_stops_at_the_dependencies_with_a_reason() {
        let mut s = Setup::new();
        let mut c = Config { logs_dir: std::env::temp_dir().join("hgt-setup-test").display().to_string(), ..Config::default() };
        let sys = crate::system::read();
        s.advance(&mut c, &sys, &[]);
        assert!(matches!(&s.steps[0].1, Status::Blocked(w) if w.contains("Game folder")), "{:?}", s.steps[0].1);
        assert!(!s.running(), "nothing left waiting");
        assert!(s.job.is_none(), "nothing was started");
    }
}

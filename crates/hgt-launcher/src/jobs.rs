//! Processes the launcher starts (the game, shader and pipeline preparation, a build), their
//! logs, and what their output says about progress. The parsers are plain functions of the
//! text, so they are tested without starting anything.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Game,
    Shaders,
    Pipelines,
    Build,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::Game => "game",
            Kind::Shaders => "shader preparation",
            Kind::Pipelines => "pipeline preparation",
            Kind::Build => "build",
        }
    }
}

pub struct Job {
    pub child: Child,
    pub kind: Kind,
    pub log: PathBuf,
    pub started: Instant,
}

/// Starts `exe` with `args` and `env` (every inherited `VICTORY_` switch cleared first), its
/// output to a new log in `logs`, no console window.
/// The launcher's own environment word asking for a High-priority process.
pub const PRIORITY: &str = "HGT_PRIORITY";

pub fn start(kind: Kind, exe: &str, args: &[String], env: &[(String, String)], cwd: Option<&Path>, logs: &Path) -> Result<Job, String> {
    if !Path::new(exe).is_file() {
        return Err(format!("no program at {exe}"));
    }
    std::fs::create_dir_all(logs).map_err(|e| format!("cannot make the log folder: {e}"))?;
    let log = logs.join(format!("{}-{}.log", kind.label().replace(' ', "-"), crate::library::now()));
    let file = std::fs::File::create(&log).map_err(|e| format!("cannot write the log: {e}"))?;
    let err = file.try_clone().map_err(|e| e.to_string())?;
    let mut cmd = Command::new(exe);
    cmd.args(args).stdout(Stdio::from(file)).stderr(Stdio::from(err)).stdin(Stdio::null());
    for (k, _) in std::env::vars().filter(|(k, _)| k.starts_with("VICTORY_")) {
        cmd.env_remove(k);
    }
    // `HGT_PRIORITY=high` is the launcher's own word (the Performance preset): the process is
    // made at High priority, and the word goes no further.
    let high = env.iter().any(|(k, v)| k == PRIORITY && v == "high");
    cmd.envs(env.iter().filter(|(k, _)| k != PRIORITY).map(|(k, v)| (k.as_str(), v.as_str())));
    if let Some(dir) = cwd.map(Path::to_path_buf).or_else(|| Path::new(exe).parent().map(Path::to_path_buf)) {
        cmd.current_dir(dir);
    }
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        const HIGH_PRIORITY_CLASS: u32 = 0x0000_0080;
        cmd.creation_flags(CREATE_NO_WINDOW | if high { HIGH_PRIORITY_CLASS } else { 0 });
    }
    let child = cmd.spawn().map_err(|e| format!("could not start: {e}"))?;
    Ok(Job { child, kind, log, started: Instant::now() })
}

impl Job {
    /// The exit status once the process has ended.
    pub fn ended(&mut self) -> Option<ExitStatus> {
        self.child.try_wait().ok().flatten()
    }

    pub fn stop(&mut self) {
        let _ = self.child.kill();
    }
}

/// The last `bytes` of a log, without the runtime's colour codes.
pub fn tail(path: &Path, bytes: u64) -> String {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut f) = std::fs::File::open(path) else { return String::new() };
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    let _ = f.seek(SeekFrom::Start(len.saturating_sub(bytes)));
    let mut buf = Vec::new();
    let _ = f.read_to_end(&mut buf);
    strip_colours(&String::from_utf8_lossy(&buf))
}

pub fn strip_colours(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
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

/// The frame line the runtime logs every second.
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Live {
    pub fps: f32,
    pub ms: f32,
    pub draws: u32,
}

fn number_before(s: &str, key: &str) -> Option<f64> {
    let end = s.find(key)?;
    let head = s[..end].trim_end();
    let start = head.rfind(|c: char| !(c.is_ascii_digit() || c == '.')).map_or(0, |i| i + 1);
    head[start..].parse().ok()
}

pub fn parse_live(text: &str) -> Option<Live> {
    let line = text.lines().rev().find(|l| l.contains(" fps, ") && l.contains("ms/frame") && l.contains("draws/frame"))?;
    let after = &line[line.find(": ").map_or(0, |i| i + 2)..];
    Some(Live {
        fps: number_before(after, " fps")? as f32,
        ms: number_before(after, " ms/frame")? as f32,
        draws: number_before(after, " draws/frame")? as u32,
    })
}

/// Progress of `victory shaders`: shaders in the cache so far, and at the end the total and
/// how many do not translate.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct ShaderProgress {
    pub done: u64,
    pub finished: bool,
    pub failed: u64,
    /// `(count, reason)` of what does not translate, from the final report.
    pub reasons: Vec<(u64, String)>,
}

pub fn parse_shaders(text: &str) -> ShaderProgress {
    let mut p = ShaderProgress::default();
    for line in text.lines() {
        let l = line.trim();
        if let Some(rest) = l.split_once(" shaders in the cache") {
            if let Ok(n) = rest.0.trim().parse::<u64>() {
                p.done = p.done.max(n);
            }
            if let Some(f) = number_before(l, " do not translate") {
                p.finished = true;
                p.failed = f as u64;
            }
        } else if p.finished
            && let Some((count, why)) = l.split_once("  ")
            && let Ok(n) = count.trim().parse::<u64>()
        {
            let why = why.split("  (e.g.").next().unwrap_or(why).trim().to_string();
            p.reasons.push((n, why));
        }
    }
    p
}

/// Progress of `victory warm`: pipelines to make, made so far, and whether it ended.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct PipelineProgress {
    pub known: u64,
    pub to_make: Option<u64>,
    pub done: u64,
    pub finished: bool,
    pub refused: u64,
}

pub fn parse_pipelines(text: &str) -> PipelineProgress {
    let mut p = PipelineProgress::default();
    for line in text.lines() {
        let l = line.trim();
        if l.contains(" pipelines known, ") {
            p.known = number_before(l, " pipelines known").unwrap_or(0.0) as u64;
            p.to_make = number_before(l, " to make").map(|n| n as u64);
        } else if let Some((a, b)) = l.split_once(" of ")
            && let (Ok(n), Some(_)) = (a.trim().parse::<u64>(), b.split(',').next())
        {
            p.done = p.done.max(n);
        } else if l.contains(" made in ") && l.contains(" have no host pipeline") {
            p.finished = true;
            p.done = number_before(l, " made in").map_or(p.done, |n| n as u64);
            p.refused = l
                .split_once('(')
                .and_then(|(_, r)| r.split_whitespace().next())
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
        }
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frame_line_is_read() {
        let line = "\u{1b}[2m2026\u{1b}[0m INFO victory_gpu::host: 14.2 fps, 70.4 ms/frame (9.1 preparing: copies 1.0 ms 2.0 MB), 4552 draws/frame, dropped 3";
        let live = parse_live(&strip_colours(line)).expect("parsed");
        assert_eq!(live, Live { fps: 14.2, ms: 70.4, draws: 4552 });
    }

    #[test]
    fn shader_preparation_is_followed() {
        let mid = "1000 shaders in the cache, 12 s\n2000 shaders in the cache, 25 s\n";
        let p = parse_shaders(mid);
        assert_eq!((p.done, p.finished), (2000, false));
        let end = format!("{mid}25866 shaders in the cache, 218 do not translate, 300 s\n    195  resources: an address in an SGPR that is no root parameter  (e.g. abcd)\n     22  emit: no wave32 lowering for this wave64 program  (e.g. ef01)\n");
        let p = parse_shaders(&end);
        assert_eq!((p.done, p.finished, p.failed), (25866, true, 218));
        assert_eq!(p.reasons.len(), 2);
        assert_eq!(p.reasons[0], (195, "resources: an address in an SGPR that is no root parameter".to_string()));
    }

    #[test]
    fn pipeline_preparation_is_followed() {
        let text = "41000 pipelines known, 39160 made before (abc), 120 bad; 1720 to make, 10 processes\n1000 of 1720, 40 s\n";
        let p = parse_pipelines(text);
        assert_eq!((p.known, p.to_make, p.done, p.finished), (41000, Some(1720), 1000, false));
        let text = format!("{text}1720 made in 80 s (3 have no host pipeline, 0 ended their process); ms each: median 30.0\n");
        let p = parse_pipelines(&text);
        assert_eq!((p.done, p.finished, p.refused), (1720, true, 3));
    }
}

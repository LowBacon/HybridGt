//! What went wrong, from what is on disk: the newest log of each kind, its errors and warnings
//! grouped by message, and the caches' state. Read only; nothing is started.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The newest log in `dir` whose name starts with `prefix`.
pub fn newest_log(dir: &Path, prefix: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with(prefix))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .max_by_key(|(t, _)| *t)
        .map(|(_, p)| p)
}

#[derive(Default, Debug, Clone)]
pub struct LogSummary {
    pub errors: u64,
    pub warnings: u64,
    /// `(count, level, message)`, most frequent first.
    pub top: Vec<(u64, &'static str, String)>,
    pub crashed: Option<String>,
}

/// Groups a log's `ERROR` and `WARN` lines by message (time stamps and numbers taken out).
pub fn summarize(text: &str) -> LogSummary {
    let mut s = LogSummary::default();
    let mut groups: BTreeMap<(&'static str, String), u64> = BTreeMap::new();
    for line in text.lines() {
        let level = if line.contains(" ERROR ") {
            "error"
        } else if line.contains(" WARN ") {
            "warning"
        } else {
            if line.contains("unhandled exception") && s.crashed.is_none() {
                s.crashed = Some(line.trim().chars().take(200).collect());
            }
            continue;
        };
        if level == "error" {
            s.errors += 1;
        } else {
            s.warnings += 1;
        }
        let msg = line.split_once(": ").map_or(line, |(_, m)| m);
        let key: String = msg.chars().map(|c| if c.is_ascii_digit() { '#' } else { c }).take(140).collect();
        *groups.entry((level, key)).or_default() += 1;
    }
    let mut top: Vec<_> = groups.into_iter().map(|((l, m), n)| (n, l, m)).collect();
    top.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(b.1)));
    top.truncate(12);
    s.top = top;
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warnings_are_grouped_and_counted() {
        let log = "2026 INFO x: fine\n2026 WARN victory_gpu::draw: a dispatch without a host pipeline dropped pipeline=0x12\n2026 WARN victory_gpu::draw: a dispatch without a host pipeline dropped pipeline=0x34\n2026 ERROR victory_gpu::host: bad thing 7\nVIctory: unhandled exception 0xc0000005 at x\n";
        let s = summarize(log);
        assert_eq!((s.errors, s.warnings), (1, 2));
        assert_eq!(s.top[0].0, 2, "{:?}", s.top);
        assert!(s.crashed.is_some());
    }

    #[test]
    fn the_newest_log_is_found() {
        let dir = std::env::temp_dir().join(format!("hgt-logs-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("game-1.log"), "a").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(dir.join("game-2.log"), "b").unwrap();
        std::fs::write(dir.join("build-3.log"), "c").unwrap();
        assert_eq!(newest_log(&dir, "game").unwrap().file_name().unwrap(), "game-2.log");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

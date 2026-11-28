//! Offline only, enforced, not just shown.
//!
//! - The launcher has no network code: no HTTP or socket crate is among its dependencies and
//!   no source file opens a connection or a web page (tests below fail the build otherwise).
//!   It checks for no updates, sends no telemetry, signs in nowhere and downloads nothing:
//!   what is missing is reported for the player to supply.
//! - The game reaches loopback only: the runtime answers every way out of Winsock and WinHTTP
//!   the title can use (connects and sends to anything but 127.0.0.0/8 and ::1 fail, name
//!   lookups fail, WinHTTP connects to other hosts fail, proxies and WPAD are off). That is
//!   the runtime's own `victory-hle` `modules::net`, always on: no switch turns it off.

/// What the Settings and Diagnostics pages say.
pub const POLICY: &[&str] = &[
    "No update checks, telemetry, sign-in or cloud saves: the launcher has no network code.",
    "Nothing is downloaded: missing pieces are reported, for you to supply from files you have.",
    "The game reaches only this PC (loopback): the runtime refuses every other connection and name lookup.",
];

#[cfg(test)]
mod tests {
    /// The crate's dependencies: none that speaks to a network.
    #[test]
    fn no_network_crate_is_a_dependency() {
        let toml = include_str!("../Cargo.toml");
        for banned in ["reqwest", "ureq", "hyper", "curl", "tokio", "attohttpc", "isahc", "surf", "minreq", "webbrowser", "open ="] {
            assert!(!toml.contains(banned), "the launcher depends on {banned}");
        }
    }

    /// No source file opens a socket, a URL or a browser.
    #[test]
    fn no_source_file_reaches_out() {
        let sources = [
            include_str!("main.rs"),
            include_str!("config.rs"),
            include_str!("deps.rs"),
            include_str!("diagnostics.rs"),
            include_str!("jobs.rs"),
            include_str!("library.rs"),
            include_str!("pages.rs"),
            include_str!("presets.rs"),
            include_str!("scan.rs"),
            include_str!("setup.rs"),
            include_str!("style.rs"),
            include_str!("system.rs"),
        ];
        let banned = [
            ["std", "::net"].concat(),
            ["Tcp", "Stream"].concat(),
            ["Udp", "Socket"].concat(),
            ["hyper", "link"].concat(),
            ["open", "_url"].concat(),
            ["Open", "Url"].concat(),
            ["Shell", "Execute"].concat(),
            ["http", "s://"].concat(),
        ];
        for (i, s) in sources.iter().enumerate() {
            for b in &banned {
                assert!(!s.contains(b.as_str()), "source {i} contains {b}");
            }
        }
    }
}

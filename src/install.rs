//! `mpm install` — provision the mpm agent skill so a coding agent learns the
//! three-moment follow-ups protocol (check the log before a PR, file deferrals
//! during, mark done after). The skill text is embedded at compile time, so this
//! works on any platform with no checkout and no network.
//!
//! - `mpm install claude|codex|gemini` → write into that host's skill directory
//!   (Claude `~/.claude/skills/mpm`, Codex `$CODEX_HOME|~/.codex/skills/mpm`,
//!   Gemini: write a stable copy then `gemini skills link`).
//! - `mpm install` (no host) → write the skill to a temp file and print the path
//!   plus a short instruction, so any other agent can read it and decide whether
//!   to copy it verbatim or adapt it to its own skill format.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const SKILL_MD: &str = include_str!("../skills/mpm/SKILL.md");
const SKILL_NAME: &str = "mpm";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Host {
    Claude,
    Codex,
    Gemini,
}

impl Host {
    pub fn label(self) -> &'static str {
        match self {
            Host::Claude => "Claude Code",
            Host::Codex => "Codex CLI",
            Host::Gemini => "Gemini CLI",
        }
    }

    pub fn parse(s: &str) -> Option<Host> {
        match s.to_ascii_lowercase().as_str() {
            "claude" | "claude-code" | "claudecode" => Some(Host::Claude),
            "codex" => Some(Host::Codex),
            "gemini" => Some(Host::Gemini),
            _ => None,
        }
    }
}

/// Entry point. `Some(host)` installs into that host; `None` writes the skill to
/// a temp file for an arbitrary agent to pick up.
pub fn run(host: Option<&str>) -> Result<()> {
    match host {
        None => write_for_arbitrary_agent(),
        Some(h) => match Host::parse(h) {
            Some(host) => install_to(host, &home_dir()?),
            None => anyhow::bail!(
                "unknown host '{h}' (expected: claude, codex, gemini).\n\
                 Run `mpm install` with no host to get the skill as a file any agent can read."
            ),
        },
    }
}

/// Cross-platform home directory: `$HOME` on Unix, `%USERPROFILE%` on Windows.
pub fn home_dir() -> Result<PathBuf> {
    #[cfg(windows)]
    let var = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"));
    #[cfg(not(windows))]
    let var = std::env::var_os("HOME");
    var.map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .context("could not determine home directory (set HOME / USERPROFILE)")
}

/// The host directory that holds skill packs, honoring `CODEX_HOME`.
fn host_root(host: Host, home: &Path) -> PathBuf {
    match host {
        Host::Claude => home.join(".claude"),
        Host::Codex => std::env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".codex")),
        Host::Gemini => home.join(".config").join("mpm"),
    }
}

/// Absolute path of the SKILL.md a host install writes.
pub fn skill_dest(host: Host, home: &Path) -> PathBuf {
    host_root(host, home).join("skills").join(SKILL_NAME).join("SKILL.md")
}

/// Write the embedded skill to `dest`, creating parents. Returns the directory.
fn write_skill_file(dest: &Path) -> Result<PathBuf> {
    let dir = dest.parent().expect("skill dest always has a parent");
    std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    std::fs::write(dest, SKILL_MD).with_context(|| format!("write {}", dest.display()))?;
    Ok(dir.to_path_buf())
}

/// Install the skill into one host.
pub fn install_to(host: Host, home: &Path) -> Result<()> {
    let dest = skill_dest(host, home);
    let dir = write_skill_file(&dest)?;
    match host {
        Host::Claude | Host::Codex => {
            println!("✓ {} — skill installed → {}", host.label(), dest.display());
        }
        Host::Gemini => link_gemini(&dir)?,
    }
    Ok(())
}

fn link_gemini(dir: &Path) -> Result<()> {
    let dir_str = dir.to_string_lossy();
    match std::process::Command::new("gemini")
        .args(["skills", "link", "--consent", &dir_str])
        .output()
    {
        Ok(out) if out.status.success() => {
            println!("✓ Gemini CLI — skill linked from {}", dir.display());
        }
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr);
            println!(
                "✓ Gemini CLI — skill written → {}\n  `gemini skills link` failed ({}); link manually:\n    gemini skills link --consent {}",
                dir.display(),
                err.trim(),
                dir.display()
            );
        }
        Err(_) => {
            println!(
                "✓ Gemini CLI — skill written → {}\n  `gemini` CLI not found; once installed, run:\n    gemini skills link --consent {}",
                dir.display(),
                dir.display()
            );
        }
    }
    Ok(())
}

/// Path the no-host install writes to: a stable temp location (overwritten each
/// run, no timestamp) so the printed path is reproducible.
pub fn arbitrary_agent_skill_path() -> PathBuf {
    std::env::temp_dir().join("mpm-skill").join("SKILL.md")
}

/// Write the skill to a temp file and tell the calling agent where to read it.
fn write_for_arbitrary_agent() -> Result<()> {
    let dest = arbitrary_agent_skill_path();
    write_skill_file(&dest)?;
    println!(
        "Wrote the mpm skill to:\n  {path}\n\n\
         To the agent reading this: open that file and adopt the skill — either copy\n\
         SKILL.md verbatim into your own skills directory, or rewrite it into your\n\
         own skill format. It teaches when and how to call `mpm` for follow-ups.\n\n\
         For a known host you can skip this: `mpm install claude|codex|gemini`.",
        path = dest.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_dest_claude_is_dot_claude_skills_mpm() {
        assert_eq!(
            skill_dest(Host::Claude, Path::new("/h")),
            Path::new("/h/.claude/skills/mpm/SKILL.md")
        );
    }

    #[test]
    fn skill_dest_codex_uses_host_root() {
        assert_eq!(
            skill_dest(Host::Codex, Path::new("/h")),
            host_root(Host::Codex, Path::new("/h")).join("skills/mpm/SKILL.md")
        );
    }

    #[test]
    fn host_parse_accepts_aliases_and_rejects_unknown() {
        assert_eq!(Host::parse("Claude"), Some(Host::Claude));
        assert_eq!(Host::parse("claude-code"), Some(Host::Claude));
        assert_eq!(Host::parse("CODEX"), Some(Host::Codex));
        assert_eq!(Host::parse("gemini"), Some(Host::Gemini));
        assert_eq!(Host::parse("vscode"), None);
    }

    #[test]
    fn install_to_writes_embedded_skill() {
        let home = std::env::temp_dir().join("mpm_install_test_claude");
        let _ = std::fs::remove_dir_all(&home);
        install_to(Host::Claude, &home).unwrap();
        let written = std::fs::read_to_string(skill_dest(Host::Claude, &home)).unwrap();
        assert_eq!(written, SKILL_MD);
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn arbitrary_agent_path_is_under_temp_dir() {
        let p = arbitrary_agent_skill_path();
        assert!(p.starts_with(std::env::temp_dir()));
        assert!(p.ends_with("SKILL.md"));
    }

    #[test]
    fn embedded_skill_is_well_formed() {
        assert!(SKILL_MD.starts_with("---"));
        assert!(SKILL_MD.contains("description:"));
    }
}

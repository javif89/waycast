use std::process::{Command, Stdio};
use nucleo_matcher::{
    Matcher, Utf32Str,
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
};

/// Spawn a detached process that preserves the display environment
pub fn spawn_detached(program: &str, args: &[&str]) -> Result<(), std::io::Error> {
    use std::os::unix::process::CommandExt;

    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // Explicitly preserve important environment variables
    if let Ok(wayland_display) = std::env::var("WAYLAND_DISPLAY") {
        cmd.env("WAYLAND_DISPLAY", wayland_display);
    }
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    if let Ok(xdg_runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        cmd.env("XDG_RUNTIME_DIR", xdg_runtime_dir);
    }
    if let Ok(xdg_session_type) = std::env::var("XDG_SESSION_TYPE") {
        cmd.env("XDG_SESSION_TYPE", xdg_session_type);
    }
    if let Ok(xdg_current_desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        cmd.env("XDG_CURRENT_DESKTOP", xdg_current_desktop);
    }

    unsafe {
        cmd.pre_exec(|| {
            // Start new process group but don't create new session
            // This allows detachment while preserving session environment
            libc::setpgid(0, 0);
            Ok(())
        });
    }

    cmd.spawn()?;

    Ok(())
}

/// Trait for types that can be fuzzy searched
pub trait FuzzySearchable {
    /// Return the string to match against
    fn search_key(&self) -> String;
}

/// A simple wrapper around nucleo-matcher for fuzzy string matching
pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl FuzzyMatcher {
    /// Create a new fuzzy matcher with default configuration
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(nucleo_matcher::Config::DEFAULT),
        }
    }

    /// Match a query against a list of strings, returning the best matches with their scores
    /// 
    /// Returns a Vec of (score, original_string) tuples, sorted by score (best first)
    pub fn match_strings(&mut self, query: &str, candidates: &[String], max_results: usize) -> Vec<(u16, String)> {
        if query.is_empty() {
            return Vec::new();
        }

        let atom = Atom::new(
            query,
            CaseMatching::Ignore,
            Normalization::Smart,
            AtomKind::Fuzzy,
            false,
        );

        let mut scored_matches: Vec<(u16, String)> = Vec::new();

        for candidate in candidates {
            if let Some(score) = atom.score(Utf32Str::Ascii(candidate.as_bytes()), &mut self.matcher) {
                scored_matches.push((score, candidate.clone()));
            }
        }

        // Sort by score (higher scores first)
        scored_matches.sort_by(|a, b| b.0.cmp(&a.0));

        // Return top results
        scored_matches.into_iter().take(max_results).collect()
    }

    /// Match a query against a list of FuzzySearchable items, returning the best matches
    /// 
    /// Returns a Vec of matched items, sorted by relevance (best first)
    pub fn match_items<'a, T: FuzzySearchable>(&mut self, query: &str, candidates: &'a [T], max_results: usize) -> Vec<&'a T> {
        if query.is_empty() {
            return Vec::new();
        }

        let atom = Atom::new(
            query,
            CaseMatching::Ignore,
            Normalization::Smart,
            AtomKind::Fuzzy,
            false,
        );

        let mut scored_matches: Vec<(u16, &'a T)> = Vec::new();

        for candidate in candidates {
            let search_key = candidate.search_key();
            if let Some(score) = atom.score(Utf32Str::Ascii(search_key.as_bytes()), &mut self.matcher) {
                scored_matches.push((score, candidate));
            }
        }

        // Sort by score (higher scores first)
        scored_matches.sort_by(|a, b| b.0.cmp(&a.0));

        // Return top results (without scores)
        scored_matches
            .into_iter()
            .take(max_results)
            .map(|(_, item)| item)
            .collect()
    }
}

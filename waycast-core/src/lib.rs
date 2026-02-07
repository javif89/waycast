use nucleo_matcher::{
    Matcher, Utf32Str,
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum LaunchError {
    CouldNotLaunch(String),
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum ItemKind {
    DesktopEntry,
    File,
    Project,
    Unknown,
}

pub enum Icon {
    Themed(String),
    Path(String),
}

#[derive(Debug, Clone)]
pub struct LauncherItem {
    pub id: String,
    pub kind: ItemKind,
    pub title: String,
    pub description: Option<String>,
    pub icon: String,
}

impl FuzzySearchable for LauncherItem {
    fn primary_key(&self) -> String {
        self.title.clone()
    }

    fn secondary_keys(&self) -> Vec<String> {
        vec![self.description.clone().unwrap_or_default()]
    }
}

pub trait WaycastScanner {
    fn scan(&self) -> Vec<LauncherItem>;
}

/// Trait for types that can be fuzzy searched
pub trait FuzzySearchable {
    /// Return the primary string to match against (highest priority)
    fn primary_key(&self) -> String;

    /// Return secondary search keys (lower priority than primary)
    /// Default implementation returns empty Vec for types with no secondary keys
    fn secondary_keys(&self) -> Vec<String> {
        Vec::new()
    }
}

/// A simple wrapper around nucleo-matcher for fuzzy string matching
pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyMatcher {
    /// Create a new fuzzy matcher with default configuration
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(nucleo_matcher::Config::DEFAULT),
        }
    }

    /// Match a query against a list of FuzzySearchable items, returning the best matches
    ///
    /// Returns a Vec of matched items, sorted by relevance (best first)
    pub fn match_items<'a, T: FuzzySearchable>(
        &mut self,
        query: &str,
        candidates: &'a [T],
        max_results: usize,
    ) -> Vec<&'a T> {
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
            let mut best_score = None;

            // Try primary key first (full score)
            let primary_key = candidate.primary_key();
            if let Some(score) =
                atom.score(Utf32Str::Ascii(primary_key.as_bytes()), &mut self.matcher)
            {
                best_score = Some(score);
            }

            // Try secondary keys (with slight penalty to prioritize primary)
            let secondary_keys = candidate.secondary_keys();
            for secondary_key in &secondary_keys {
                if let Some(score) =
                    atom.score(Utf32Str::Ascii(secondary_key.as_bytes()), &mut self.matcher)
                {
                    // Apply small penalty to secondary matches (90% of original score)
                    let adjusted_score = (score as f32 * 0.9) as u16;
                    best_score = Some(
                        best_score.map_or(adjusted_score, |existing| existing.max(adjusted_score)),
                    );
                }
            }

            if let Some(score) = best_score {
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

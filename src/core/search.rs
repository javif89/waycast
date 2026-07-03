use nucleo_matcher::{
    Matcher, Utf32Str,
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
};
use std::cmp::Reverse;

use super::LauncherItem;

impl FuzzySearchable for LauncherItem {
    fn primary_key(&self) -> String {
        self.title.clone()
    }

    fn secondary_keys(&self) -> Vec<String> {
        vec![self.description.clone().unwrap_or_default()]
    }
}

pub trait FuzzySearchable {
    fn primary_key(&self) -> String;

    fn secondary_keys(&self) -> Vec<String> {
        Vec::new()
    }
}

pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(nucleo_matcher::Config::DEFAULT),
        }
    }

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
            let primary_key = candidate.primary_key();
            if let Some(score) =
                atom.score(Utf32Str::Ascii(primary_key.as_bytes()), &mut self.matcher)
            {
                best_score = Some(score);
            }

            for secondary_key in &candidate.secondary_keys() {
                if let Some(score) =
                    atom.score(Utf32Str::Ascii(secondary_key.as_bytes()), &mut self.matcher)
                {
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

        scored_matches.sort_by_key(|item| Reverse(item.0));
        scored_matches
            .into_iter()
            .take(max_results)
            .map(|(_, item)| item)
            .collect()
    }
}

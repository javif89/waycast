use std::path::Path;

use tokei::{Config, Language, LanguageType, Languages};

pub struct ProjectLanguage {
    pub name: String,
    pub percentage: f64,
}

pub struct TypeScanner {
    tokei_config: Config,
    ignore_langs: [LanguageType; 5],
}

impl TypeScanner {
    pub fn new() -> TypeScanner {
        TypeScanner {
            tokei_config: Config::default(),
            ignore_langs: [
                LanguageType::Css,
                LanguageType::Json,
                LanguageType::Markdown,
                LanguageType::CppHeader,
                LanguageType::CHeader,
            ],
        }
    }

    // Scan a project for languages used. Limit gives the
    // top [limit] entries
    pub fn scan<P: AsRef<Path>>(&self, path: P, limit: Option<usize>) -> Vec<ProjectLanguage> {
        let mut langs = Languages::new();
        langs.get_statistics(&[path], &[], &self.tokei_config);

        let total_code: usize = langs.iter().map(|(_, l)| l.code).sum();
        let mut rows: Vec<ProjectLanguage> = langs
            .iter()
            .map(|(lt, l)| {
                (
                    *lt,
                    l.code,
                    if total_code > 0 {
                        (l.code as f64) * 100.0 / (total_code as f64)
                    } else {
                        0.0
                    },
                )
            })
            .filter(|(l, _, _)| !self.ignore_langs.contains(l))
            .map(|(lang, _, percent)| ProjectLanguage {
                name: String::from(lang.name()),
                percentage: percent,
            })
            .collect();
        rows.sort_by(|a, b| b.percentage.partial_cmp(&a.percentage).unwrap());

        if let Some(l) = limit {
            rows.truncate(l);
        }

        rows
    }
}

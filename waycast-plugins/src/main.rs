use std::{collections::BTreeMap, path::PathBuf};
use tokei::{Config, LanguageType, Languages};

fn lang_breakdown(paths: &[&str], excluded: &[&str]) -> Vec<(LanguageType, usize, f64)> {
    let mut langs = Languages::new();
    let cfg = Config::default();
    langs.get_statistics(paths, excluded, &cfg);

    let total_code: usize = langs.iter().map(|(_, l)| l.code).sum();
    let mut rows: Vec<_> = langs
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
        .collect();
    rows.sort_by_key(|(_, lines, _)| std::cmp::Reverse(*lines));
    rows
}

pub fn main() {
    if let Ok(entries) = std::fs::read_dir(PathBuf::from("/home/javi/projects")) {
        for e in entries
            .into_iter()
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap())
        {
            let langs = lang_breakdown(&[e.path().to_str().unwrap()], &[]);

            let top = langs
                .iter()
                .map(|(l, _, _)| l.to_owned())
                .take(3)
                .collect::<Vec<LanguageType>>();

            println!("{}: {:?}", e.path().display(), top);
        }
    }
}

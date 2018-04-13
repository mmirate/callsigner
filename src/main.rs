extern crate itertools;
extern crate regex;
extern crate serde_json;
extern crate structopt;

use itertools::Itertools;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
struct Opt {
    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str), default_value = "-")]
    files: Vec<PathBuf>,
}

fn read_names() -> Vec<(Option<usize>, Vec<String>)> {
    let dash = PathBuf::from("-".to_owned());
    let opt = Opt::from_args();
    println!("{:?}", opt);
    let stdin_ = io::stdin();
    let mut stdin: Option<Box<BufRead>> = Some(Box::new(stdin_.lock()));
    let mut inputs: Vec<Box<BufRead>> = vec![];
    for p in &opt.files {
        (if p == &dash {
            std::mem::replace(&mut stdin, None)
        } else {
            let b: Box<BufRead> = Box::new(BufReader::new(File::open(p).unwrap()));
            stdin = None;
            Some(b)
        })
        .map(|x| inputs.push(x));
    }
    if let Some(s) = stdin {
        inputs.push(s)
    }

    let line_regex = regex::Regex::new(r#"([(](?P<p>\d+)[)] +)?(?P<n>.+)"#).unwrap();

    let mut names = vec![];
    for input in inputs {
        for line_ in input.lines() {
            if let Ok(line) = line_ {
                if let Some(caps) = line_regex.captures(&line) {
                    let pri: Option<usize> =
                        caps.name("p").map(|m| str::parse(m.as_str()).unwrap());
                    let mut words = caps
                        .name("n")
                        .map(|m| m.as_str())
                        .unwrap_or_default()
                        .split_whitespace()
                        .collect::<Vec<_>>();
                    for i in words
                        .iter()
                        .enumerate()
                        .filter(|&(ref _i, ref s)| s.starts_with('"'))
                        .map(|(i, _)| i)
                        .collect::<Vec<_>>()
                    {
                        let x = words.remove(i);
                        words.insert(0, x);
                    }
                    let name = words
                        .into_iter()
                        .map(|s| s.trim_matches('"').to_lowercase())
                        .collect();
                    names.push((pri, name));
                }
            }
        }
    }

    names
}

fn score_assignment(a: &[(Vec<String>, (Option<usize>, usize, usize))]) -> Option<usize> {
    let (chars, score_terms): (Vec<_>, Vec<_>) = a
        .iter()
        .map(|&(ref words, (ref pri, ref w, ref c))| {
            Some((
                words.get(*w)?.chars().nth(*c)?,
                4 * c * pri.unwrap_or(1) + w * pri.unwrap_or(1),
            ))
        })
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .unzip();
    let has_dups = {
        let mut chars = chars;
        chars.sort();
        let old_l = chars.len();
        chars.dedup();
        chars.len() != old_l
    };
    if has_dups {
        Some(usize::max_value())
    } else {
        Some(score_terms.into_iter().sum())
    }
}

fn output_assignment<D: std::fmt::Display>(
    a: &[(Vec<String>, (Option<usize>, usize, usize))],
    score: D,
) {
    println!("--- {}", score);
    for &(ref words, (ref _pri, ref w, ref c)) in a {
        println!(
            "{} - {}",
            words[*w].chars().nth(*c).unwrap().to_uppercase(),
            words.join(" ")
        );
    }
}

fn main() {
    let mut names = read_names()
        .into_iter()
        .map(|(pri, k)| (k, (pri, 0, 0)))
        .collect::<Vec<_>>();

    println!("{:?}", names);

    let mut best_names = None;
    let mut best_score = usize::max_value();

    let prod = (0..names.len())
        .map(|n| {
            let mut v = (0..names[n].0.len())
                .flat_map(|w| (0..std::cmp::min(names[n].0[w].len(), 3)).zip(std::iter::repeat(w)))
                .collect::<Vec<_>>();
            v.sort();
            v.into_iter()
        })
        .multi_cartesian_product();

    for cws in prod {
        for (n, (c, w)) in cws.into_iter().enumerate() {
            (names[n].1).1 = w;
            (names[n].1).2 = c;
        }
        if let Some(score) = score_assignment(&names) {
            if score < best_score {
                best_names = Some(names.clone());
                best_score = score;
                output_assignment(best_names.as_ref().unwrap(), score);
            }
        }
    }

    output_assignment(
        best_names.as_ref().unwrap(),
        format!("BEST: {}", best_score),
    );
}

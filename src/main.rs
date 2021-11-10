extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::{collections::HashMap, path::PathBuf, time::Instant};
use clap::Parser;
use git2::{Commit, ObjectType, Repository, Sort};
use lazy_static::lazy_static;
use regex::Regex;

pub type DownloadCounts = HashMap<String, DownloadCountsValue>;

#[derive(Serialize, Deserialize)]
pub struct DownloadCountsValue {
    count: i64,
}

#[derive(Parser)]
#[clap(version = "1.0", author = "goaaats <goatsdev@protonmail.com>")]
struct Opts {
    /// Sets a custom template file.
    #[clap(short, long, default_value = "template.html")]
    template: String,
    /// Path to the input repository.
    input_repo: String,
    /// Amount of commits to analyze.
    #[clap(short, long)]
    num_commits: Option<i32>,
    /// Log verbose
    #[clap(short, long)]
    verbose: bool,
}

fn is_commit_applicable(commit: &Commit) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new("Update download counts").unwrap();
    }

    let message = commit.message();
    if message.is_none(){
        return false;
    }

    RE.is_match(message.unwrap())
}

fn main() {
    let opts: Opts = Opts::parse();

    let now = Instant::now();

    let repo = match Repository::open(opts.input_repo) {
        Ok(repo) => repo,
        Err(e) => panic!("Failed to open repository: {}", e),
    };

    let mut revwalk = match repo.revwalk() {
        Ok(revwalk) => revwalk,
        Err(e) => panic!("Could not start revwalk: {}", e),
    };

    revwalk.set_sorting(Sort::TIME).unwrap();
    revwalk.push_head().unwrap();
    for rev in revwalk {
        let oid = rev.unwrap();
        let commit = repo.find_commit(oid).unwrap();
     
        if !is_commit_applicable(&commit){
            continue;
        }

        let tree = commit.tree().unwrap();
        let entry = tree.get_path(&PathBuf::from("downloadcounts.json")).unwrap();

        assert!(entry.kind() == Some(ObjectType::Blob));

        let object = entry.to_object(&repo).unwrap();
        let bytes = object.as_blob().unwrap().content();
        let text = std::str::from_utf8(bytes).unwrap();

        let model: DownloadCounts = serde_json::from_str(text).unwrap();
    }

    println!("Done in {}s", now.elapsed().as_secs_f32());
}

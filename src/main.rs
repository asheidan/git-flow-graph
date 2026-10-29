use std::{collections::HashMap, iter::repeat_n};

use chrono::{DateTime, FixedOffset};
use clap::Parser;
use git2::{Branch, Oid, Repository, Sort};
use git_flow_graph::colorize::Colorize;

#[derive(Parser, Debug)]
struct Args {
    /// Set the path to the repository (".git" directory). This can also be controlled by setting
    /// the GIT_DIR environment variable. It can be an abslute path or relative path to the current
    /// working directory.
    #[clap(long, env="GIT_DIR", default_value_t=String::from("."), value_name="PATH")]
    git_dir: String,

    #[clap(long, default_value_t=String::from("feature/"))]
    feature_prefix: String,

    #[clap(long, default_value_t=String::from("hotfix/"))]
    hotfix_prefix: String,

    #[clap(long, default_value_t = 50)]
    summary_length: usize,
}

fn list_branches(repo: &Repository) -> HashMap<Oid, Vec<String>> {
    let branches = match repo.branches(None) {
        Err(e) => panic!("Failed to list branches: {}", e),
        Ok(branches) => branches,
    };

    // Mutable to be able to sort it after collection
    let mut branch_map: Vec<(Oid, Branch)> = branches
        .map(|info| match info {
            Err(e) => panic!("Failed to get branch: {}", e),
            Ok((branch, _)) => branch,
        })
        .map(|branch| {
            let sha = match branch.get().peel_to_commit() {
                Err(e) => panic!("Failed to peel reference to commit: {}", e),
                Ok(commit) => commit.id(),
            };

            // branch_name is a borrow based on branch. Need value to outlive scope.
            (sha, branch)
        })
        .collect();
    branch_map.sort_by_key(|(sha, _)| *sha);

    branch_map
        .chunk_by(|a, b| a.0 == b.0)
        .map(|group| match *group {
            [(sha, _), ..] => (
                sha,
                group
                    .iter()
                    .map(|(_, branch)| branch.name().unwrap().unwrap().to_string())
                    .collect::<Vec<String>>(),
            ),
            [] => panic!("We got an empty slice, we don't want an empty slice"),
        })
        .collect()
}

fn print_padding(open_branch_count: usize) {
    repeat_n("│ ", open_branch_count).for_each(|s| print!("{}", s));
}

fn main() {
    // Ways of handling storing the commit-graph:
    // - Thread on Stack Overflow
    //   https://stackoverflow.com/questions/34747464/implement-graph-like-data-structure-in-rust
    // - Article on Graphs and arena allocation
    //   https://github.com/nrc/r4cppp/blob/master/graphs/README.md
    // - Modeling graphs in Rust using vector indices
    //   https://smallcultfollowing.com/babysteps/blog/2015/04/06/modeling-graphs-in-rust-using-vector-indices/
    let args = Args::parse();
    let summary_length = args.summary_length;

    let repo = Repository::open(args.git_dir).unwrap();

    let branch_heads = list_branches(&repo);
    //branch_heads.append((repo.head().unwrap().peel_to_commit().unwrap(), String::from("HEAD")));

    let mut revwalk = repo.revwalk().unwrap();
    revwalk.set_sorting(Sort::TOPOLOGICAL).unwrap();

    // Revwalk seems to treat the pushed references as a stack. The last pushed reference is walked
    // first.
    repo.reflog("HEAD")
        .unwrap()
        .iter()
        .take(10)
        .rev()
        .for_each(|reflog_entry| revwalk.push(reflog_entry.id_new()).unwrap());
    //revwalk.push_ref("refs/heads/feature/a-feature").unwrap();
    revwalk.push_head().unwrap();

    let mut open_branches: Vec<Oid> = Vec::new();
    let mut next_commit: Option<Oid> = None;

    revwalk
        .take(10)
        .map(|maybe_commit| maybe_commit.unwrap())
        .for_each(|oid| {
            // This is a terrible variable name :(
            let is_open = open_branches.iter().position(|sha| sha == &oid);
            if let Some(position) = is_open {
                // TODO: Render the "start of the branch here"
                println!("├─╯");
                open_branches.remove(position);
            }

            let expected_sha = next_commit.unwrap_or(oid);
            if expected_sha != oid {
                open_branches.push(expected_sha);
                print_padding(open_branches.len());
                println!();
            }

            let commit = repo.find_commit(oid).unwrap();

            match commit.parent_count() {
                0 => next_commit = None,
                1 => next_commit = Some(commit.parent(0).unwrap().id()),
                _ => println!("I don't handle merges yet"),
            }

            let commit_time = commit.time();
            let commit_timezone = FixedOffset::east_opt(commit_time.offset_minutes() * 60).unwrap();
            let commit_datetime = DateTime::from_timestamp_secs(commit_time.seconds())
                .unwrap()
                .with_timezone(&commit_timezone);

            let author_name = String::from(commit.author().name().unwrap());

            print_padding(open_branches.len());
            print!(
                // Oid can be shortened by using precision {:.length}
                "○  {:.8} {} {}",
                Colorize::Purple(oid),
                author_name,
                Colorize::Cyan(commit_datetime.naive_local()) // TODO: Control date output with options
            );

            if let Some(branches) = branch_heads.get(&oid) {
                let branch_decoration = branches.iter().fold(String::new(), |a, b| a + b + " ");

                if branch_decoration.len() != 0 {
                    print!(" {}", Colorize::Green(branch_decoration));
                }
            }

            // ---- End of first line ---------------------------------
            println!();

            print_padding(open_branches.len());

            let commit_stem = match next_commit {
                None => " ",
                Some(_) => "│",
            };
            println!(
                "{:1}  {:.summary_length$}",
                commit_stem,
                commit.summary().unwrap()
            );
        });
}

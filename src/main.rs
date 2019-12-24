extern crate clap;
extern crate git2;
extern crate graphql_client;
extern crate reqwest;
extern crate serde;
extern crate serde_derive;

mod git_extras;
mod github;

use clap::{App, Arg};
use git2::{Config, Repository, Status};
use git_extras::Repo;
use std::process::{Command, ExitStatus};
use std::{env, io, process};

use github::branches_by_milestone;

fn main() {
    let opts = App::new("git-integrate")
        .arg(
            Arg::with_name("milestone")
                .value_name("MILESTONE")
                .help("GitHub milestone number")
                .index(1),
        )
        .arg(
            Arg::with_name("branch")
                .value_name("BRANCH")
                .help("Branch to build")
                .index(2),
        )
        .get_matches();

    let milestone = opts
        .value_of("milestone")
        .and_then(|x| x.trim().parse().ok())
        .expect("No github label provided");
    let dest_branch = opts.value_of("branch").expect("No branch provided");

    let current_dir = match env::current_dir() {
        Ok(current_dir) => current_dir,
        Err(e) => panic!("{}", e),
    };

    let repository = match Repository::discover(current_dir.as_path()) {
        Ok(repository) => repository,
        Err(e) => panic!("{}", e),
    };

    let remote = match repository.find_remote("origin") {
        Ok(remote) => remote,
        Err(e) => panic!("{}", e),
    };

    let repo = match Repo::new(&remote) {
        Some(repo) => repo,
        None => panic!("Could not build remote info"),
    };

    let config = Config::open_default().expect("Could not find a git configuration file!");
    let github_token = config
        .get_string("integrate.github-token")
        .expect("Could not find integrate.github-token in any git configuration file!");

    if !git_fetch().expect("Error fetching from remote").success() {
        process::exit(1)
    }

    if !git_checkout(dest_branch)
        .expect(&format!("Could not checkout branch {}", dest_branch))
        .success()
    {
        process::exit(1)
    }

    let branches = match branches_by_milestone(github_token, repo, milestone) {
        Ok(branches) => branches,
        Err(e) => panic!("{}", e),
    };

    for branch in branches {
        println!("\nMerging {}", branch);
        merge_branch(branch, &repository);
    }
}

fn merge_branch(branch: String, repository: &Repository) {
    if !git_merge(&branch)
        .expect(&format!("Failure merging branch {}", branch))
        .success()
    {
        let dirty = repository
            .statuses(None)
            .expect("Error checking dirty repository")
            .iter()
            .any(|s| s.status() == Status::CONFLICTED);

        if dirty {
            println!(
                "\nMerge conflict detected, either fix the conflict and \
                 \nuse `git commit --no-edit` commit this merge or use \
                 \n`git merge --abort` to quit this merge"
            );
            process::exit(1);
        }

        if !git_commit()
            .expect(&format!("Failure merging branch {}", branch))
            .success()
        {
            println!("Failure mergeing branch {}", branch);
            process::exit(1);
        }
    }
}

fn git_fetch() -> io::Result<ExitStatus> {
    Command::new("git").arg("fetch").arg("--all").status()
}

fn git_checkout(branch: &str) -> io::Result<ExitStatus> {
    Command::new("git")
        .arg("checkout")
        .arg("--no-track")
        .arg("-B")
        .arg(branch)
        .arg("origin/master")
        .status()
}

fn git_merge(branch: &String) -> io::Result<ExitStatus> {
    Command::new("git")
        .arg("merge")
        .arg("--no-ff")
        .arg("--no-edit")
        .arg("--rerere-autoupdate")
        .arg("--log")
        .arg(&format!("origin/{}", branch))
        .status()
}

fn git_commit() -> io::Result<ExitStatus> {
    Command::new("git").arg("commit").arg("--no-edit").status()
}

use std::env;
use std::fs;
use std::process::{Command, Stdio};

const WORK_DIR: &str = "SEXXI_DIR";

const UPSTREAM_REMOTE_URL: &str = "git@github.com:rust-lang/rust.git";
const UPSTREAM_REMOTE: &str = "origin";

const SG_REMOTE_URL: &str = "git@github.com:sexxi-goose/rust.git";
const SG_REMOTE: &str = "sg";

const SG_REMOTE_ERR: &str = "sg remote not detected";

fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

fn git_clone(work_tree: &str) -> Result<(), &'static str> {
    let git_clone = Command::new("git")
        .stdout(Stdio::piped())
        .args(&["clone", UPSTREAM_REMOTE_URL, work_tree])
        .spawn()
        .expect("ok");

    let output = git_clone.wait_with_output().expect("ok");

    if !output.status.success() {
        return Err("Failed to clone the rust repo");
    }

    Ok(())
}

fn add_remote(git_dir: &str, work_tree: &str) -> Result<(), &'static str> {
    let git_add_remote = Command::new("git")
        .args(&["--git-dir", &git_dir])
        .args(&["--work-tree", &work_tree])
        .stdout(Stdio::piped())
        .args(&["remote", "add", SG_REMOTE, SG_REMOTE_URL])
        .spawn()
        .expect("ok");

    let output = git_add_remote.wait_with_output().expect("ok");

    if !output.status.success() {
        return Err("Failed to add SEXXI remote");
    }

    Ok(())
}

fn git_fetch(git_dir: &str, work_tree: &str) -> Result<(), &'static str> {
    let git_fetch = Command::new("git")
        .args(&["--git-dir", &git_dir])
        .args(&["--work-tree", &work_tree])
        .stdout(Stdio::piped())
        .args(&["fetch", UPSTREAM_REMOTE])
        .spawn()
        .expect("ok");

    let output = git_fetch.wait_with_output().expect("ok");

    if !output.status.success() {
        return Err("Failed to fetch from origin");
    }

    Ok(())
}

fn git_force_push(git_dir: &str, work_tree: &str) -> Result<(), &'static str> {
    let git_force_push = Command::new("git")
        .args(&["--git-dir", &git_dir])
        .args(&["--work-tree", &work_tree])
        .stdout(Stdio::piped())
        .args(&["push", SG_REMOTE, "origin/master:master", "-f"])
        .spawn()
        .expect("ok");

    let output = git_force_push.wait_with_output().expect("ok");

    if !output.status.success() {
        return Err("Failed to force push to sg");
    }

    Ok(())
}

fn git_check_remote(git_dir: &str, work_tree: &str) -> Result<(), &'static str> {
    let git_check_remote = Command::new("git")
        .args(&["--git-dir", &git_dir])
        .args(&["--work-tree", &work_tree])
        .args(&["remote", "show", SG_REMOTE])
        .output()
        .expect("ok");

    if !git_check_remote.status.success() {
        return Err(SG_REMOTE_ERR);
    }

    Ok(())
}

fn main() {
    let mut workdir = env::var("PWD").unwrap();
    match env::var(WORK_DIR) {
        Ok(val) => {
            workdir = val;
            println!("workdir: {}", workdir);
        }
        Err(_) => println!("using default directory: {}", workdir),
    }

    // Creating the workdir if not exist.
    if !path_exists(&workdir) {
        let result = fs::create_dir_all(&workdir);
        match result {
            Ok(_) => {}
            Err(e) => panic!("unable to create workdir: {}", e),
        }
    }

    let repo_dir = format!("{}/rust", &workdir);
    let repo_git_dir = format!("{}/.git", &repo_dir);

    // Check if we already have the rust repo cloned.
    let repo_exists = Command::new("git")
        .args(&["--git-dir", &repo_git_dir])
        .args(&["--work-tree", &repo_dir])
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .expect("rust");

    if repo_exists.status.success() {
        // We fetch new update if the repo is already cloned.
        println!("Existing rust repo detected. Fetching for new updates.");
        if let Err(e) = git_fetch(&repo_git_dir, &repo_dir) {
            panic!(format!("git fetch failed: {}", e));
        }
    } else {
        // We clone the repo if does not exist.
        println!("Unable to detect Rust repo. Cloning.");
        if let Err(e) = git_clone(&repo_dir) {
            panic!(format!("git clone failed: {}", e));
        }
    }

    // Check if we have added sg remote to our repo.
    match git_check_remote(&repo_git_dir, &repo_dir) {
        Ok(_) => {}
        // if not, we add the sg remote.
        Err(e) if e == SG_REMOTE_ERR => {
            if let Err(e) = add_remote(&repo_git_dir, &repo_dir) {
                panic!(format!("git remote add sg failed: {}", e));
            }
        }
        Err(e) => panic!(format!("unexpected error: {}", e)),
    }

    // We force push the change to our sg repo.
    println!("Pushing new changes to {}", SG_REMOTE);
    if let Err(e) = git_force_push(&repo_git_dir, &repo_dir) {
        panic!(format!("git push failed: {}", e));
    }
}

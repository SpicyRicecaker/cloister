use std::{
    fs::{read_dir, FileType},
    path::{Path, PathBuf},
};

use anyhow::{bail, Error};
use clap::{Parser, Subcommand};
use git2::{Repository, RepositoryOpenFlags, StatusOptions};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds files to myapp
    Zip { path: String },
}

enum Subcommands {
    Add,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    // user must supply a path to a repository, as well as the relative path into that repository.
    // but that's really annoying. for usability, we should just let the user
    // specify a path, and recurse up into that repository.
    let cli = Cli::parse();

    match &cli.command {
        Commands::Zip { path } => {
            // verify that path exists
            let path = Path::new(&path);

            if !path.exists() {
                bail!("path {path:?} does not exist...");
            }

            let git_path = get_ancestor_git(&path)?;

            git(git_path, path)?;
        }
    }

    Ok(())
}

// recursively traverse until gitignore file is found
fn get_ancestor_git(path: &Path) -> Result<PathBuf, Error> {
    let mut path = PathBuf::from(path);

    if !path.is_dir() {
        bail!("path {path:?} is not a directory...");
    }

    loop {
        if read_dir(&path)?
            .flat_map(|p| p.ok())
            .any(|e| e.file_name().eq_ignore_ascii_case(".git") && e.file_type().unwrap().is_dir())
        {
            return Ok(path);
        }

        if !path.pop() {
            bail!("ancestor directories did not contain a git repository lol");
        }
    }
}

fn git(git_path: PathBuf, relative_path: &Path) -> Result<(), Error> {
    let repo = Repository::open_ext(".", RepositoryOpenFlags::empty(), Path::new("."))?;
    let workdir = repo.workdir().unwrap();

    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    opts.include_unmodified(true);
    opts.pathspec(Path::new("*"));

    let statuses = repo.statuses(Some(&mut opts))?;

    for status in statuses.iter() {
        let path = status.path().unwrap();
        println!("{}", workdir.join(path).display());
    }

    let repo = Repository::open_ext(".", RepositoryOpenFlags::empty(), Path::new("."))?;
    let workdir = repo.workdir().unwrap();

    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    opts.include_unmodified(true);
    opts.pathspec(Path::new("*"));

    let statuses = repo.statuses(Some(&mut opts))?;

    for status in statuses.iter() {
        let path = status.path().unwrap();
        println!("{}", workdir.join(path).display());
    }

    todo!()
}

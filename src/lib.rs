pub mod run;

use anyhow::{bail, Error};
use clap::{Parser, Subcommand};
use log::LevelFilter;
use run::zip;
use std::path::PathBuf;

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
    Zip { zip_dir: String },
}

#[allow(dead_code)]
enum Subcommands {
    Add,
}

pub struct App {
    zip_dir: PathBuf,
}

impl App {
    pub fn new() -> Result<Self, Error> {
        // user supplies a path to a repository, as well as the relative subfolder into that repository.
        let cli = Cli::parse();

        let Commands::Zip { zip_dir } = &cli.command;
        // verify that path exists
        let zip_dir = PathBuf::from(zip_dir);

        if !zip_dir.exists() {
            bail!("path {zip_dir:?} does not exist...");
        }

        Ok(Self { zip_dir })
    }
    pub fn run(&self) -> Result<(), Error> {
        zip(&self.zip_dir)?;

        Ok(())
    }
}

pub fn init_logger() {
    env_logger::builder().filter_level(LevelFilter::Info).init();
}

#[cfg(test)] // this directive just makes it so that imports in tests are not marked errors
mod test {
    use std::{fs, path::Path, io::Write, env::set_current_dir, process::Command};
    use log::LevelFilter;
    use super::*;

    #[test]
    fn test() {
        init_logger();
        // create a directory "test"
        // create a subdirectory "cs261"
        // create a subdirectory "assignment1"
        fs::create_dir_all("./test/cs261/assignment1").unwrap();
        // add a file to that subdirectory "hello.c"
        // make "hello.c" contain "hello world"

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(Path::new("./test/cs261/assignment1/hello.c"))
            .unwrap();
        file.write_all("hello world".as_bytes()).unwrap();

        // change working directory to "cs261"
        set_current_dir(Path::new("./test/cs261")).unwrap();

        // run git("assignment1")
        zip(Path::new("./assignment1")).unwrap();
        // run unzip "assignment1.zip" (ensure macos only)
        let _ = Command::new("unzip")
            .args(["assignment1.zip"])
            .output()
            .expect("failed to unzip the assignment");
        // [assert] check if "bob.c" exists and contains the string "hello world"
        let p = Path::new("./hello.c");

        let result = p.exists() && fs::read_to_string(p).unwrap() == "hello world";

        // remove directory "test"
        set_current_dir(Path::new("../..")).unwrap();
        fs::remove_dir_all(Path::new("test")).unwrap();

        assert!(result);
    }

    #[test]
    #[ignore] // this directive prevents this test from being run by default
    /// This test determines the effect of set_current_dir and "."
    fn test_current_dir() {
        env_logger::builder().filter_level(LevelFilter::Info).init();

        // create path .
        let p = Path::new(".");
        // check value of .
        dbg!(p.canonicalize().unwrap());

        // change current directory to src directory
        set_current_dir(Path::new("./src")).unwrap();
        // check value of previous .
        dbg!(p.canonicalize().unwrap());
        // create new . and check its value
        let np = Path::new(".");
        dbg!(np.canonicalize().unwrap());

        // conclusion: changing the current directory modifies not only the current
        // path but also previous paths, thus you cannot store "./" paths, modify
        // the current directory, then use the stored paths ...
    }
}

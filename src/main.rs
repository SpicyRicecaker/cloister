use std::{
    env::set_current_dir,
    fs::{self, File},
    io::Seek,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Context, Error};
use clap::{Parser, Subcommand};
use git2::{Repository, RepositoryInitOptions, RepositoryOpenFlags, StatusOptions, StatusShow};
use log::{info, LevelFilter};
use zip::{write::FileOptions, ZipWriter};

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

#[allow(dead_code)]
enum Subcommands {
    Add,
}

fn main() -> Result<(), Error> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    // user supplies a path to a repository, as well as the relative subfolder into that repository.
    let cli = Cli::parse();

    match &cli.command {
        Commands::Zip { path } => {
            // verify that path exists
            let path = Path::new(path);

            if !path.exists() {
                bail!("path {path:?} does not exist...");
            }

            git(path)?;
        }
    }

    Ok(())
}

/// This function takes in a subdirectory path into a git repository, and
/// creates a zip file in the current directory including the files in the
/// subdirectory, with the basename of the subdirectory as the name of the zip
/// file.
fn git(src_dir: &Path) -> Result<(), Error> {
    // will search up to parent paths for git repo if needed?
    let repo = Repository::open_ext(src_dir, RepositoryOpenFlags::empty(), Path::new("."))?;

    let mut opts = StatusOptions::new();
    // opts.include_untracked(true);
    // opts.recurse_untracked_dirs(true);
    // opts.include_unmodified(true);
    opts.show(StatusShow::IndexAndWorkdir);

    let dir_name = src_dir
        .file_name()
        .context("unable to convert")?
        .to_str()
        .context("unable to turn dir name!")?;

    let glob_path: PathBuf = [src_dir].iter().collect();
    opts.pathspec(glob_path);

    let statuses = repo.statuses(Some(&mut opts))?;

    // create zip file, with the name of the current repo. place in current working directory
    let dst_path_str = format!("{dir_name}.zip");
    let dst_path = Path::new(&dst_path_str);
    let zip_file = File::create(dst_path).unwrap();

    let c: Vec<PathBuf> = statuses
        .into_iter()
        .map(|s| PathBuf::from(s.path().unwrap()))
        .collect();

    info!("the directory {src_dir:?} contains the files {c:?}");

    info!("begining to compress {src_dir:?} into {:?}", &zip_file);

    zip_dir(
        c,
        src_dir.to_str().context("unable to convert")?,
        zip_file,
        zip::CompressionMethod::Deflated,
    )?;

    info!("done!!");

    Ok(())
}

fn zip_dir<T>(
    it: Vec<PathBuf>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    info!("found git repo at: {prefix:?}");

    let mut zip = ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        // read, write
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for path in it {
        let name = path
            .strip_prefix(Path::new(prefix))
            .unwrap()
            .to_str()
            .unwrap();

        if !path.is_file() {
            panic!();
        }

        // add contents of file into zip
        info!("adding file {path:?} as {name:?} to {prefix:?}.zip ...");
        zip.start_file(name, options)?;
        let mut f = File::open(path)?;
        f.read_to_end(&mut buffer)?;
        zip.write_all(&buffer)?;
        buffer.clear();
    }
    zip.finish()?;
    Result::Ok(())
}

#[test]
fn test() {
    env_logger::builder().filter_level(LevelFilter::Info).init();
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

    // initialize a repository in that directory
    let _ = Repository::init_opts("test", &RepositoryInitOptions::new()).unwrap();

    // change working directory to "cs261"
    set_current_dir(Path::new("./test/cs261")).unwrap();

    // run git("assignment1")
    git(Path::new("./assignment1")).unwrap();
    // run unzip "assignment1.zip" (ensure macos only)
    let _ = Command::new("unzip")
        .args(["assignment1.zip"])
        .output()
        .expect("failed to unzip the assignment");
    // [assert] check if "bob.c" exists and contains the string "hello world"
    let p = Path::new("./bob.c");

    let result = p.exists() && fs::read_to_string(p).unwrap() == "hello world";

    // remove directory "test"
    set_current_dir(Path::new("../..")).unwrap();
    fs::remove_dir_all(Path::new("test")).unwrap();

    assert!(result);
}

#[test]
#[ignore] // this directory prevents this test from being run by default
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
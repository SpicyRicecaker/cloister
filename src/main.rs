use std::{
    fs::{read_dir, DirEntry, File, FileType, self},
    io::Seek,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Error};
use clap::{Parser, Subcommand};
use git2::{Repository, RepositoryOpenFlags, StatusOptions};
use log::info;
use zip::{result::ZipError, write::FileOptions, ZipWriter};

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
            let path = Path::new(path);

            if !path.exists() {
                bail!("path {path:?} does not exist...");
            }

            // dbg!("f", &path);

            git(path)?;
        }
    }

    Ok(())
}

fn git(src_dir: &Path) -> Result<(), Error> {
    // will search up to parent paths for git repo if needed?
    let repo = Repository::open_ext(src_dir, RepositoryOpenFlags::empty(), Path::new("."))?;
    let workdir = repo.workdir().unwrap();

    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(true);
    opts.include_unmodified(true);

    let dir_name = src_dir
        .file_name()
        .context("unablet o convert")?
        .to_str()
        .context("unable to turn dir name!")?;

    let glob_path: PathBuf = [src_dir, Path::new("**")].iter().collect();
    opts.pathspec(glob_path);

    let statuses = repo.statuses(Some(&mut opts))?;

    // create zip file, with the name of the current repo. place in current working directory
    let dst_path_str = format!("{dir_name}.zip");
    let dst_path = Path::new(&dst_path_str);
    let zip_file = File::create(dst_path).unwrap();

    let c: Vec<PathBuf> = statuses
            .into_iter()
            .map(|s| s.path().unwrap().to_string())
            .map(|p| workdir.join(p))
            .collect();

        // dbg!(&c);

    zip_dir(
        c,
        src_dir.canonicalize()?.to_str().context("unable to convert")?,
        zip_file,
        zip::CompressionMethod::Bzip2,
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
    dbg!(prefix);

    let mut zip = ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for path in it {
        // for entry in fs::read_dir(path_current)? {
            let name = path.strip_prefix(Path::new(prefix)).unwrap();

            // Write file or directory explicitly
            // Some unzip tools unzip files with directory paths correctly, some do not!
            if path.is_file() {
                println!("adding file {path:?} as {name:?} ...");
                #[allow(deprecated)]
                zip.start_file_from_path(name, options)?;
                let mut f = File::open(path)?;

                f.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
                buffer.clear();
            } else if !name.as_os_str().is_empty() {
                // Only if not root! Avoids path spec / warning
                // and mapname conversion failed error on unzip
                println!("adding dir {path:?} as {name:?} ...");
                #[allow(deprecated)]
                zip.add_directory_from_path(name, options)?;
            }
        // }
    }
    zip.finish()?;
    Result::Ok(())
}

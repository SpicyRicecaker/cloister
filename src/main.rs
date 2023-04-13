use std::{
    fs::File,
    io::Seek,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Error};
use clap::{Parser, Subcommand};
use git2::{Repository, RepositoryOpenFlags, StatusOptions};
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
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .init();
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

fn git(src_dir: &Path) -> Result<(), Error> {
    // will search up to parent paths for git repo if needed?
    let repo = Repository::open_ext(src_dir, RepositoryOpenFlags::empty(), Path::new("."))?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(true);
    opts.include_unmodified(true);

    let dir_name = src_dir
        .file_name()
        .context("unable to convert")?
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
        .map(|s| PathBuf::from(s.path().unwrap()))
        .collect();

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

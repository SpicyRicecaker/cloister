use std::{
    fs::{self, read_dir, DirEntry, File, FileType},
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
        // .map(|p| workdir.join(p))
        .collect();

    // dbg!(&c);
    dbg!(src_dir, &zip_file);

    zip_dir(
        c,
        src_dir
            .to_str()
            .context("unable to convert")?,
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
        dbg!(&path);
        // for entry in fs::read_dir(path_current)? {
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {path:?} as {name:?} ...");
            // dbg!(&path, &name);
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            unreachable!();
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

// // BEGIN EXAMPLE LOL XD

// use std::io::prelude::*;
// use std::io::{Seek, Write};
// use std::iter::Iterator;
// use zip::result::ZipError;
// use zip::write::FileOptions;

// use std::fs::File;
// use std::path::Path;
// use walkdir::{DirEntry, WalkDir};

// fn main() {
//     std::process::exit(real_main());
// }

// const METHOD_STORED: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Stored);

// #[cfg(any(
//     feature = "deflate",
//     feature = "deflate-miniz",
//     feature = "deflate-zlib"
// ))]
// const METHOD_DEFLATED: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Deflated);
// #[cfg(not(any(
//     feature = "deflate",
//     feature = "deflate-miniz",
//     feature = "deflate-zlib"
// )))]
// const METHOD_DEFLATED: Option<zip::CompressionMethod> = None;

// #[cfg(feature = "bzip2")]
// const METHOD_BZIP2: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Bzip2);
// #[cfg(not(feature = "bzip2"))]
// const METHOD_BZIP2: Option<zip::CompressionMethod> = None;

// #[cfg(feature = "zstd")]
// const METHOD_ZSTD: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Zstd);
// #[cfg(not(feature = "zstd"))]
// const METHOD_ZSTD: Option<zip::CompressionMethod> = None;

// fn real_main() -> i32 {
//     let args: Vec<_> = std::env::args().collect();
//     if args.len() < 3 {
//         println!(
//             "Usage: {} <source_directory> <destination_zipfile>",
//             args[0]
//         );
//         return 1;
//     }

//     let src_dir = &*args[1];
//     let dst_file = &*args[2];
//     for &method in [METHOD_STORED, METHOD_DEFLATED, METHOD_BZIP2, METHOD_ZSTD].iter() {
//         if method.is_none() {
//             continue;
//         }
//         match doit(src_dir, dst_file, method.unwrap()) {
//             Ok(_) => println!("done: {src_dir} written to {dst_file}"),
//             Err(e) => println!("Error: {e:?}"),
//         }
//     }

//     0
// }

// fn zip_dir<T>(
//     it: &mut dyn Iterator<Item = DirEntry>,
//     prefix: &str,
//     writer: T,
//     method: zip::CompressionMethod,
// ) -> zip::result::ZipResult<()>
// where
//     T: Write + Seek,
// {
//     let mut zip = zip::ZipWriter::new(writer);
//     let options = FileOptions::default()
//         .compression_method(method)
//         .unix_permissions(0o755);

//     let mut buffer = Vec::new();
//     for entry in it {
//         let path = entry.path();
//         let name = path.strip_prefix(Path::new(prefix)).unwrap();

//         // Write file or directory explicitly
//         // Some unzip tools unzip files with directory paths correctly, some do not!
//         if path.is_file() {
//             println!("adding file {path:?} as {name:?} ...");
//             #[allow(deprecated)]
//             zip.start_file_from_path(name, options)?;
//             let mut f = File::open(path)?;

//             f.read_to_end(&mut buffer)?;
//             zip.write_all(&buffer)?;
//             buffer.clear();
//         } else if !name.as_os_str().is_empty() {
//             // Only if not root! Avoids path spec / warning
//             // and mapname conversion failed error on unzip
//             println!("adding dir {path:?} as {name:?} ...");
//             #[allow(deprecated)]
//             zip.add_directory_from_path(name, options)?;
//         }
//     }
//     zip.finish()?;
//     Result::Ok(())
// }

// fn doit(
//     src_dir: &str,
//     dst_file: &str,
//     method: zip::CompressionMethod,
// ) -> zip::result::ZipResult<()> {
//     if !Path::new(src_dir).is_dir() {
//         return Err(ZipError::FileNotFound);
//     }

//     let path = Path::new(dst_file);
//     let file = File::create(path).unwrap();

//     let walkdir = WalkDir::new(src_dir);
//     let it = walkdir.into_iter();

//     dbg!(src_dir, &file);
//     zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;

//     Ok(())
// }
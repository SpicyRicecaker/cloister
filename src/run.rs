use std::{
    fs::File,
    io::Seek,
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Error};
use ignore::Walk;
use log::info;
use zip::{write::FileOptions, ZipWriter};

/// This function takes in a subdirectory path into a git repository, and
/// creates a zip file in the current directory including the files in the
/// subdirectory, with the basename of the subdirectory as the name of the zip
/// file.
pub fn zip(premise_dir: &Path) -> Result<(), Error> {
    let premise_dir_name = premise_dir
        .file_name()
        .context("unable to convert")?
        .to_str()
        .context("unable to turn dir name!")?;

    // create zip file, with the name of the current repo. place in current working directory
    let conclusion_dir = format!("{premise_dir_name}.zip");
    let conclusion_path = Path::new(&conclusion_dir);
    let conclusion_zip_file = File::create(conclusion_path).unwrap();

    let premise_subfiles: Vec<_> = Walk::new(premise_dir).filter_map(|e| e.ok()).collect();

    info!("the directory {premise_dir_name:?} contains the files {premise_subfiles:?}");

    info!(
        "begining to compress {premise_dir_name:?} into {:?}.zip",
        premise_dir_name
    );

    zip_dir(
        premise_subfiles,
        premise_dir,
        conclusion_zip_file,
        zip::CompressionMethod::Deflated,
    )?;

    info!("done!!");

    Ok(())
}

fn zip_dir<T>(
    it: Vec<ignore::DirEntry>,
    prefix: &Path,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    info!("found git repo at: {prefix:?}");

    let mut zip_writer = ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        // read, write
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it.into_iter().filter(|e| e.path().is_file()) {
        let name = entry
            .path()
            .strip_prefix(Path::new(prefix))
            .unwrap()
            .to_str()
            .unwrap();

        // add contents of file into zip
        info!(
            "adding file {:?} as {name:?} to {prefix:?}.zip ...",
            entry.path()
        );
        zip_writer.start_file(name, options)?;
        let mut f = File::open(entry.path())?;
        f.read_to_end(&mut buffer)?;
        zip_writer.write_all(&buffer)?;
        buffer.clear();
    }
    zip_writer.finish()?;
    Result::Ok(())
}

use rand::{self, Rng, distributions::Alphanumeric};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

fn main() -> io::Result<()> {
    let path = Path::new(".");
    let mut oldfiles = vec![];

    for entry in fs::read_dir(path)? {
        let file_path = entry?.path();
        if file_path.is_file() || file_path.is_dir() {
            oldfiles.push(file_path.to_string_lossy().to_string());
        }
    }

    println!("{:?}", oldfiles);

    let tmpfile_path = write_filenames_to_tmpfile(&oldfiles)?;
    println!("tmpfile_path = {}", tmpfile_path.display());
    open_file_in_vim(&tmpfile_path)?;
    fs::remove_file(&tmpfile_path)?;

    Ok(())
}

fn write_filenames_to_tmpfile(lines: &[String]) -> io::Result<PathBuf> {
    // Generate a random filename using the `rand` crate
    let mut rng = rand::thread_rng();
    let filename: String = std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric) as char)
        .take(8)
        .collect();
    let file_path = PathBuf::from(format!("/tmp/rbrn2_{}", filename));

    // Open a new file for writing
    let mut file = File::create(&file_path)?;

    // Iterate over the lines and write each line to the file
    for line in lines {
        writeln!(file, "{}", line)?;
    }

    // Flush the file to ensure all the data is written
    file.flush()?;

    // Return the path of the written file
    Ok(file_path)
}

fn open_file_in_vim<T: AsRef<Path>>(filename: T) -> io::Result<()> {
    let filename_str = filename.as_ref().to_str().unwrap();

    let status = Command::new("vim")
        .arg(filename_str)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("vim exited with non-zero status: {}", status),
        ));
    }

    Ok(())
}

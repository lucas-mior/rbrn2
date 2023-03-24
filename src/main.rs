use rand::Rng;
use rand::distributions::Alphanumeric;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::{Result, Write};
use std::fs::{File, OpenOptions};
use tempfile::NamedTempFile;
use std::process::{Command, Stdio};

fn main() {
    let path = Path::new(".");
    let mut files_and_dirs = vec![];

    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let file_path = entry.path();
        if file_path.is_file() || file_path.is_dir() {
            files_and_dirs.push(file_path.to_string_lossy().into_owned());
        }
    }

    println!("{:?}", files_and_dirs);

    let a = write_filenames_to_tmpfile(&files_and_dirs).unwrap();
    println!("a = {}", a.display());
    // open_file_in_vim(&a);
}

fn write_filenames_to_tmpfile(lines: &Vec<String>) -> Result<PathBuf> {
    // Generate a random filename using the `rand` crate
    let mut rng = rand::thread_rng();
    let filename: String = std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric) as char)
        .take(8)
        .collect();
    let file_path = PathBuf::from(format!("/tmp/{}", filename));

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

fn open_file_in_vim(filename: &str) {
    let mut vim_command = Command::new("vim");
    vim_command.arg(filename);
    vim_command.stdin(Stdio::inherit());
    vim_command.stdout(Stdio::inherit());
    vim_command.stderr(Stdio::inherit());

    let mut vim_process = vim_command.spawn().unwrap();
    vim_process.wait().unwrap();
}

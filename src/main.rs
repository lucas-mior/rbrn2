use rand::{self, Rng, distributions::Alphanumeric};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use std::io::{BufRead, BufReader, Result};
use std::os::unix::ffi::OsStrExt;
use std::ffi::CString;
use std::os::raw::{c_int, c_char};

fn main() -> io::Result<()> {
    let path = Path::new(".");
    let mut oldfiles = vec![];

    for entry in fs::read_dir(path)? {
        let file_path = entry?.path();
        if file_path.is_file() || file_path.is_dir() {
            oldfiles.push(file_path.to_string_lossy().to_string());
        }
    }

    println!("oldfiles {:?}", oldfiles);

    let tmpfile_path = write_filenames_to_tmpfile(&oldfiles)?;
    println!("tmpfile_path = {}", tmpfile_path.display());
    open_file_in_vim(&tmpfile_path)?;
    // fs::remove_file(&tmpfile_path)?;
    let newfiles = read_lines_from_file(&tmpfile_path)?;

    println!("newfiles: {:?}", newfiles);

    rename_files(&oldfiles, &newfiles)?;

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

fn read_lines_from_file<T: AsRef<Path>>(file_path: T) -> Result<Vec<String>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    for line in reader.lines() {
        lines.push(line?.trim().to_string());
    }
    Ok(lines)
}

fn rename_files(oldfiles: &[String], newfiles: &[String]) -> io::Result<()> {
    // Iterate over the oldfiles and newfiles vectors
    for (oldfile, newfile) in oldfiles.iter().zip(newfiles) {
        // Check if the filenames are different
        if oldfile != newfile {
            // Rename the file
            rename(&oldfile, &newfile)?;
            println!("Renamed file from {} to {}", oldfile, newfile);
        }
    }
    Ok(())
}

fn rename(old_path: &str, new_path: &str) -> std::io::Result<()> {
    let old_cstr = CString::new(old_path.as_bytes())?;
    let new_cstr = CString::new(new_path.as_bytes())?;
    let result = unsafe {
        libc::renameat2(
            libc::AT_FDCWD, old_cstr.as_ptr(),
            libc::AT_FDCWD, new_cstr.as_ptr(),
            libc::RENAME_EXCHANGE
        )
    };

    if result == -1 {
        // fallback to using standard rename function
        std::fs::rename(old_path, new_path)?;
    }

    Ok(())
}

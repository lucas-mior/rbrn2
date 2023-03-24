use rand::{Rng, distributions::Alphanumeric};
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, Result, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    ffi::CString,
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

    let tmpfile_path = write_filenames_to_tmpfile(&oldfiles)?;
    open_file_in_vim(&tmpfile_path)?;
    let newfiles = read_lines_from_file(&tmpfile_path)?;

    rename_files(&oldfiles, &newfiles)?;

    fs::remove_file(&tmpfile_path)?;
    Ok(())
}

fn write_filenames_to_tmpfile(lines: &[String]) -> io::Result<PathBuf> {
    let mut rng = rand::thread_rng();
    let filename: String = std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric) as char)
        .take(8)
        .collect();
    let file_path = PathBuf::from(format!("/tmp/rbrn2_{}", filename));

    let mut file = File::create(&file_path)?;

    for line in lines {
        writeln!(file, "{}", line)?;
    }

    file.flush()?;

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
    for (oldfile, newfile) in oldfiles.iter().zip(newfiles) {
        if oldfile != newfile {
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

    if result < 0 {
        std::fs::rename(old_path, new_path)?;
    }

    Ok(())
}

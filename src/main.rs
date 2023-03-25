use rand::{Rng, distributions::Alphanumeric};
use std::{
    fs::{read_dir, remove_file, File, rename},
    io::{self, BufRead, BufReader, Result, Write},
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
};
use std::collections::HashMap;

fn main() -> io::Result<()> {
    let old_files = get_files_in_directory(".")?;

    let tmp_file = write_filenames_to_tmpfile(&old_files)?;
    open_file_in_vim(&tmp_file)?;
    let new_files = read_lines_from_file(&tmp_file)?;

    if old_files.len() != new_files.len() {
        println!("Lenghts differ");
        process::exit(1);
    }
    if has_duplicates(&new_files) {
        println!("has duplicates!!!");
        process::exit(1);
    }
    rename_files(&old_files, &new_files)?;

    remove_file(&tmp_file)?;
    Ok(())
}

fn get_files_in_directory<T: AsRef<Path>>(directory: T) -> io::Result<Vec<String>> {
    let mut files = vec![];
    for entry in read_dir(directory)? {
        let file_path = entry?.path();
        if file_path.is_file() || file_path.is_dir() {
            files.push(file_path.to_string_lossy().to_string());
        }
    }
    Ok(files)
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

fn rename_files(old_files: &[String], new_files: &[String]) -> io::Result<()> {
    old_files.iter()
        .zip(new_files)
        .filter(|(old_file, new_file)| old_file != new_file)
        .try_for_each(|(old_file, new_file)| {
            rename(old_file, new_file)?;
            println!("Renamed file from {} to {}", old_file, new_file);
            Ok(())
        })
}

fn has_duplicates<T: AsRef<str>>(v: &[T]) -> bool {
    let mut map = HashMap::new();

    for s in v {
        let count = map.entry(s.as_ref()).or_insert(0);
        *count += 1;
        if *count > 1 {
            return true;
        }
    }

    false
}

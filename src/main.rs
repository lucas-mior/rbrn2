use rand::{Rng, distributions::Alphanumeric};
use std::{
    fs::{read_dir, remove_file, File, rename},
    io::{self, BufRead, BufReader, Result, Write, stdout, stderr},
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
    collections::HashMap,
    ffi::CString,
};

fn usage(stream: &mut dyn Write) {
    writeln!(stream, "usage: brn2 [--help | <filename>]").unwrap();
    writeln!(stream, "Without arguments, rename files in current dir.").unwrap();
    writeln!(stream, "<filename>, rename files listed in <filename>.").unwrap();
    writeln!(stream, "--help : display this help message.").unwrap();
    writeln!(stream, "Be sure to have EDITOR or VISUAL environment variables properly set.").unwrap();
    process::exit(if stream as *const _ == &std::io::stdout() as *const _ { 0 } else { 1 });
}

const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut oldfiles;
    if args.len() == 0 {
        oldfiles = get_files_in_directory(".")?;
    } else if args.len() == 1 {
        match args[0].as_str() {
            "-h" | "--help" => { 
                usage(&mut stdout());
                process::exit(0);
            },
            _ => oldfiles = read_lines_from_file(&args[0])?,
        };
    } else {
        usage(&mut stderr());
        process::exit(1);
    }

    let tmp_file = write_filenames_to_tmpfile(&oldfiles)?;
    open_file_in_vim(&tmp_file)?;
    let newfiles = read_lines_from_file(&tmp_file)?;

    if oldfiles.len() != newfiles.len() {
        eprintln!(
            "You are renaming {} file{} but buffer contains {} file name{}",
            oldfiles.len(), if oldfiles.len() == 1 { "" } else { "s" },
            newfiles.len(), if newfiles.len() == 1 { "" } else { "s" }
        );
        process::exit(1);
    }
    if has_duplicates(&newfiles) {
        eprintln!("Since there are duplicated names in the buffer, no files will be renamed.");
        process::exit(1);
    }
    rename_files(&mut oldfiles, &newfiles)?;

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

fn rename_files(oldfiles: &mut[String], newfiles: &[String]) -> Result<()> {
    for i in 0..oldfiles.len() {
        let oldname = &oldfiles[i];
        let newname = &newfiles[i];
        if oldfiles[i] == newfiles[i] {
            continue;
        }

        // Try using renameat2 with RENAME_EXCHANGE flag
        let oldpath_c = CString::new(oldfiles[i].as_bytes()).unwrap();
        let newpath_c = CString::new(newfiles[i].as_bytes()).unwrap();
        let result = unsafe { 
            libc::renameat2(libc::AT_FDCWD, oldpath_c.as_ptr(), 
                            libc::AT_FDCWD, newpath_c.as_ptr(), 
                            libc::RENAME_EXCHANGE)
        };
        if result >= 0 {
            println!("{oldname} -> {GREEN}{newname}{RESET}");

            for j in i+1..oldfiles.len() {
                if oldfiles[j] == newfiles[i] {
                    oldfiles.swap(j, i);
                    println!("{} -> {GREEN}{}{RESET}", newfiles[i], oldfiles[i]);
                }
            }
        } else {
            // Fall back to rename if renameat2 fails
            if let Err(e) = rename(&oldfiles[i], &newfiles[i]) {
                eprintln!("Error renaming {} to {}: {}", &oldfiles[i], &newfiles[i], e);
                continue;
            }
            println!("{oldname} -> {GREEN}{newname}{RESET}");
        }
    }
    Ok(())
}

fn has_duplicates<T: AsRef<str>>(v: &[T]) -> bool {
    let mut map = HashMap::new();
    let mut dup = false;

    for s in v {
        let count = map.entry(s.as_ref()).or_insert(0);
        *count += 1;
        if *count > 1 {
            eprintln!("{} appers more than once in the buffer", s.as_ref());
            dup = true;
        }
    }

    return dup;
}

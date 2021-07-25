use clap::{App, Arg};
use git2::{Repository, Status};
use ring::digest::{Context, Digest, SHA256};
use std::fs;
use std::fs::metadata;
use std::fs::File;
use std::io::ErrorKind;
use std::io::{BufReader, Read, Write};
use std::process::Command;

const CPP_EXT: &'static [&str] = &["cpp", "hpp", "tpp", "h", "c"];

fn process_file(filename: &str, verbose: bool) {
    let mut processed = true;
    for ext in CPP_EXT.iter() {
        if filename.ends_with(ext) {
            processed = true;
            match format_file(filename) {
                Ok(FormatResult::NoChanges) => {
                    if verbose {
                        println!("No changes: {}", filename);
                    }
                }
                Ok(FormatResult::Formated) => {
                    println!("Formated: {}", filename);
                }
                Err(error) => {
                    println!("Error {} on file {}", error, filename)
                }
            }
            break;
        }
    }
    if !processed && verbose {
        println!("Not a cpp file: {}", filename);
    }
}

fn process_path(input: &str, verbose: bool) {
    let md = metadata(input);
    if md.is_err() {
        println!("Unknown directory or file: {}", input);
        return;
    }
    let md = md.unwrap();
    if md.is_dir() {
        let paths = fs::read_dir(input).unwrap();
        for path in paths {
            if path.is_ok() {
                process_path(path.unwrap().path().to_str().unwrap(), verbose);
            }
        }
    } else if md.is_file() {
        process_file(input, verbose);
    } else {
        if verbose {
            println!("Given directory or file is invalid {}", input);
        }
    }
}

fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest, std::io::Error> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

fn get_file_hash(filename: &str) -> Result<Digest, std::io::Error> {
    let input = File::open(filename)?;
    let reader = BufReader::new(input);
    sha256_digest(reader)
}
enum FormatResult {
    NoChanges,
    Formated,
}

fn format_file(filename: &str) -> Result<FormatResult, std::io::Error> {
    let output = Command::new("clang-format.exe").arg(&filename).output()?;
    match output.status.code() {
        Some(code) => {
            if code == 0 {
                let old_hash = get_file_hash(filename)?;
                //println!("Old hash {:?}", old_hash);
                let new_hash = sha256_digest(&output.stdout[..])?;
                //println!("New hash {:?}", new_hash);
                if old_hash.as_ref() == new_hash.as_ref() {
                    return Ok(FormatResult::NoChanges);
                } else {
                    let mut file = File::create(&filename)?;
                    file.write_all(&output.stdout[..])?;
                    drop(file);
                    return Ok(FormatResult::Formated);
                }
            } else {
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    format!("Clang-format process returns error {}", code),
                ));
            }
        }
        None => {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                format!("Clang-format process terminated by signal"),
            ))
        }
    }
}

fn main() {
    let matches = App::new("clang-format-git")
        .arg(
            Arg::with_name("input")
                .help("Input repository")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("format all")
                .long("all")
                .short("a")
                .help("Verbose output")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .help("Verbose output")
                .takes_value(false),
        )
        .get_matches();

    let input = matches.value_of("input").unwrap_or(".");
    let verbose = matches.is_present("verbose");
    let all = matches.is_present("format all");

    if all {
        process_path(input, verbose);
    } else {
        if verbose {
            println!("Checking index from {:?}", input);
        }

        let repo = Repository::open(input).expect("Failed to open repository");
        //let repo = Repository::discover(&mut input).expect("Failed to open repository");

        let index = repo
            .statuses(Option::None)
            .expect("Failed to retrieve repository status");

        if verbose {
            println!("Formating...");
        }

        for d in index.iter() {
            let status = d.status();
            //println!("val {:?} file {}", status, d.path().unwrap());
            if !(status & (Status::INDEX_MODIFIED | Status::INDEX_NEW)).is_empty() {
                let filename = d.path().unwrap();
                match metadata(&filename) {
                    Ok(m) => {
                        if m.is_file() {
                            process_file(filename, verbose);
                        } else if verbose {
                            println!("Not a file: {}", filename);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    if verbose {
        println!("Finished");
    }
}

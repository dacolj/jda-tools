use clap::{App, Arg};
use git2::{Repository, Status};
use ring::digest::{Context, Digest, SHA256};
use std::fs;
use std::fs::metadata;
use std::fs::File;
use std::io::ErrorKind;
use std::io::{BufReader, Read, Write};
use std::process::Command;

const CPP_EXT: &'static str =".cpp,.hpp,.tpp,.h,.c,ipp";

struct Config {
    verbose: bool,
    dry_run: bool,
    extensions: Vec<String>
}

fn process_file(filename: &str, config: &Config) {
    for ext in config.extensions.iter() {
        if filename.ends_with(ext) {
            match format_file(filename, &config) {
                Ok(_) => {}
                Err(error) => {
                    println!("Error {} on file {}", error, filename)
                }
            }
            break;
        }
    }
}

fn process_path(input: &str, config: &Config) {
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
                process_path(path.unwrap().path().to_str().unwrap(), config);
            }
        }
    } else if md.is_file() {
        process_file(input, &config);
    } else {
        if config.verbose {
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

fn format_file(filename: &str, config: &Config) -> Result<(), std::io::Error> {
    let output = Command::new("clang-format.exe").arg(&filename).output()?;
    match output.status.code() {
        Some(code) => {
            if code == 0 {
                let old_hash = get_file_hash(filename)?;
                //println!("Old hash {:?}", old_hash);
                let new_hash = sha256_digest(&output.stdout[..])?;
                //println!("New hash {:?}", new_hash);
                if old_hash.as_ref() == new_hash.as_ref() {
                    if config.verbose {
                        println!("No changes: {}", filename);
                    }
                } else if config.dry_run {
                    println!("Will format: {}", filename);
                } else {
                    println!("Formated: {}", filename);
                    let mut file = File::create(&filename)?;
                    file.write_all(&output.stdout[..])?;
                    drop(file);
                }
                return Ok(());
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
                .help("Format all")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .help("Verbose output")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("dry run")
                .long("dry-run")
                .short("d")
                .help("Don't write files, just show what would be done")
                .takes_value(false),
        ).arg(
            Arg::with_name("extensions")
                .long("extensions")
                .short("e")
                .help(format!("Extensions to manges, default \"{}\"", CPP_EXT).as_str())
                .takes_value(false),
        )
        .get_matches();

    let input = matches.value_of("input").unwrap_or(".");

    let extensions: Vec<String> = match matches.value_of("extensions") {
        Some(value) =>{
            value
        }
        None => CPP_EXT,
    }.split(',').map(String::from).collect();

    let config = Config {
        verbose: matches.is_present("verbose"),
        dry_run: matches.is_present("dry run"),
        extensions: extensions,
    };

    let all = matches.is_present("format all");

    if all {
        process_path(input, &config);
    } else {
        if config.verbose {
            println!("Checking index from {:?}", input);
        }

        let repo = Repository::open(input).expect("Failed to open repository");
        //let repo = Repository::discover(&mut input).expect("Failed to open repository");

        let index = repo
            .statuses(Option::None)
            .expect("Failed to retrieve repository status");

        if config.verbose {
            println!("Formating...");
        }

        println!("Hello");
        for d in index.iter() {
            let status = d.status();
            if !(status
                & (Status::WT_RENAMED
                    | Status::WT_NEW
                    | Status::WT_MODIFIED
                    | Status::INDEX_MODIFIED
                    | Status::INDEX_NEW
                    | Status::INDEX_RENAMED))
                .is_empty()
            {
                let filename = d.path().unwrap();
                match metadata(&filename) {
                    Ok(m) => {
                        if m.is_file() {
                            process_file(filename, &config);
                        } else if config.verbose {
                            println!("Not a file: {}", filename);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    if config.verbose {
        println!("Finished");
    }
}

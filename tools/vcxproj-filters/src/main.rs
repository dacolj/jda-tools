use clap::{App, Arg};
use std::fs;
use std::str::FromStr;
mod filters;

fn find_vcxproj() -> Vec<String> {
    let mut result: Vec<String> = vec![];

    let paths = fs::read_dir(".").unwrap();
    for path in paths {
        if path.is_ok() {
            let value = path.unwrap().path();
            if value.is_file() && value.ends_with( ".vcxproj") {
                result.push(String::from_str(value.to_str().unwrap()).unwrap())
            }
        }
    }

    result
}

fn main() -> std::io::Result<()> {
    let matches = App::new("vcxproj filters generator")
        //.version(VERSION)
        //.about(NAME)
        //.bin_name(BINARY)
        .arg(
            Arg::with_name("input")
                .help("Input vcxproj")
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();

    let vals: Vec<String> = match matches.values_of("input") {
        Some(val) => val.map(|x| x.to_string()).collect(),
        None => find_vcxproj(),
    };

    if vals.is_empty(){
        println!("Failed to find vcxproj file in current folder");
        return Ok(());
    }

    for vcxproj_file in vals {
        println!("Processing {}...", vcxproj_file);
        filters::manage(vcxproj_file.as_str(), None, None)?;
    }

    println!("Done");

    Ok(())
}

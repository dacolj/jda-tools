mod attribute;
mod check;
mod class;
mod common;
mod doxygen;
mod enumerated;
mod function;
mod project;
use clap::{App, Arg};
use std::str::FromStr;

use crate::check::*;
use crate::project::*;

fn main() {
    let matches = App::new("CPP Documentation Analyzer")
        .version("0.1")
        .about("CPP Documentation Analyzer")
        .bin_name("filters.exe")
        .arg(
            Arg::with_name("output")
                .long("output")
                .short("o")
                .help("Output format")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("input")
                .help("Input folders")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .help("Verbose output")
                .takes_value(false),
        )
        .get_matches();

    let vec: Vec<String> = match matches.values_of("input") {
        Some(val) => val.map(|x| String::from_str(x).unwrap()).collect(),
        None => vec![String::from(".")],
    };

    let output_format: String = match matches.value_of("output") {
        Some(val) => val.to_lowercase(),
        None => String::from("output.csv"),
    };

    let mut output: Box<dyn ErrorWriter> = match output_format.as_str() {
        "vs" => Box::new(VisualStudioErrorWriter {}),
        _ => Box::new(CsvErrorWriter::new(output_format.as_str()).unwrap()),
    };

    let mut project = Project::new();
    match project.analyse(vec, output.as_mut()) {
        Err(e) => println!("Error: {:?}", e),
        _ => (),
    }

    println!("Done");
}

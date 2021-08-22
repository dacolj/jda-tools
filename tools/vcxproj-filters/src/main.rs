use clap::{App, Arg};
use std::str::FromStr;
mod filters;

fn main() -> std::io::Result<()> {
    let matches = App::new("VS filters manager")
        .version("0.1")
        .about("Generate or update vcxproj filter file")
        .bin_name("filters.exe")
        .arg(
            Arg::with_name("output")
                .long("output")
                .short("o")
                .help("Output filters file if you don't want to override input (requires a single vcxproj as input)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("no-sorting")
                .long("keep-order")
                .short("k")
                .help("Don't sort file and groups")
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
            Arg::with_name("ignore-existing")
                .long("ignore-existing")
                .short("i")
                .help("Ignore existing filter file")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("input")
                .help("Input vcxproj file(s)")
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();

    let verbose = matches.is_present("verbose");

    let vals: Vec<String> = match matches.values_of("input") {
        Some(val) => val.map(|x| x.to_string()).collect(),
        None => filters::find_vcxproj(verbose),
    };

    if vals.is_empty() {
        println!("Failed to find vcxproj file in current folder");
        return Ok(());
    }

    if matches.is_present("output") && vals.len() != 1 {
        println!("Output option cannot be used if with mutliple inputs");
        return Ok(());
    }

    let sort = !matches.is_present("no-sorting");
    for vcxproj_file in vals {
        println!("Processing {}...", vcxproj_file);

        let input_filters_filer = match matches.is_present("ignore-existing") {
            true => None,
            false => Some(format!("{}.filters", vcxproj_file)),
        };

        let output_file = match matches.value_of("output") {
            Some(val) => Some(String::from_str(val).unwrap()),
            None => None,
        };

        filters::apply(
            vcxproj_file.as_str(),
            input_filters_filer,
            output_file,
            sort,
            verbose,
        )?;

        println!("Done");
    }

    Ok(())
}

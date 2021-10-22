mod attribute;
mod check;
mod class;
mod common;
mod doxygen;
mod function;
use class::*;
use std::str::FromStr;

use crate::check::*;

fn main() {
    println!("Hello, world!");
    let mut vec = Vec::new();
    vec.push(String::from_str("path").unwrap());
    let (doxyfile, doxygen_output) = doxygen::generate_doxyfile(&vec, None).unwrap();
    doxygen::launch_doxygen(
        doxyfile.as_str(),
        "C:/Program Files/doxygen/bin/doxygen.exe",
    )
    .unwrap();
    //let mut output = CsvErrorWriter::new(r"C:\Workspace\Jean\Other\DoxygenSampleProject\output.csv").unwrap();
    let mut output = VisualErrorWriter {};
    for filename in
        doxygen::find_class_xml_files(format!("{}/xml/", &doxygen_output).as_str())
    {
        // println!("=================================================");
        // println!("Processing file {}", &filename);
        match Class::read(filename.as_str()) {
            Ok(v) => {
                //println!("Result: {:?}", &v);
                check::check_class(&v, &mut output);
            }
            Err(e) => println!("Error: {:?}", e),
        };
    }
    println!("Done");
}

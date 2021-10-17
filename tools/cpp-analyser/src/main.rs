mod attribute;
mod check;
mod class;
mod common;
mod doxygen;
mod function;
use class::*;

use crate::check::ErrorWriter;

fn main() {
    println!("Hello, world!");


    let mut output = ErrorWriter::new(r"C:\Workspace\Jean\Other\DoxygenSampleProject\output.csv").unwrap();
    for filename in
        doxygen::find_class_xml_files(r"C:\Workspace\Jean\Other\DoxygenSampleProject\xml\")
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

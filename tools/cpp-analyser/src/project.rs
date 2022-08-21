use crate::check;
use crate::check::*;
use crate::class::*;
use crate::doxygen;

use std::collections::LinkedList;
use std::fs;
use std::str::FromStr;

#[derive(Debug)]
pub struct Project {
    pub name: String,
    pub folder: String,
    pub doxygen_output: String,
    pub classes: LinkedList<Class>,
    pub objects: LinkedList<Class>,
    pub enums: LinkedList<String>,
}

impl Project {
    pub fn new() -> Project {
        Project {
            name: String::new(),
            folder: String::new(),
            doxygen_output: String::new(),
            classes: LinkedList::new(),
            objects: LinkedList::new(),
            enums: LinkedList::new(),
        }
    }

    pub fn analyse(
        &mut self,
        folder: Vec<String>,
        error_writer: &mut dyn ErrorWriter,
    ) -> Result<(), std::io::Error> {
        let (doxyfile, doxygen_output) = doxygen::generate_doxyfile(&folder, None).unwrap();
        println!("Temporary Doxyfile: {}", &doxyfile);

        doxygen::launch_doxygen(
            doxyfile.as_str(),
            "C:/Program Files/doxygen/bin/doxygen.exe",
        )?;

        println!("Output folder: {}", &doxygen_output);

        for path in fs::read_dir(format!("{}xml/", &doxygen_output).as_str()).unwrap() {
            let path = path?;
            let filename = String::from_str(path.path().to_str().unwrap()).unwrap();
            if !filename.ends_with(".xml") {
                continue;
            }
            if path.file_name().to_str().unwrap().starts_with("class") {
                match Class::read(filename.as_str(), false) {
                    Ok(v) => match check::check_class(&v, error_writer) {
                        Err(e) => println!("Error: {:?}", e),
                        _ => (),
                    },
                    Err(e) => println!("Error: {:?}", e),
                };
            }
            if path.file_name().to_str().unwrap().starts_with("struct") {
                match Class::read(filename.as_str(), true) {
                    Ok(v) => match check::check_class(&v, error_writer) {
                        Err(e) => println!("Error: {:?}", e),
                        _ => (),
                    },
                    Err(e) => println!("Error: {:?}", e),
                };
            }
        }
        Ok(())
    }
}

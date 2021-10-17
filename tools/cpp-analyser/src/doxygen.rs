use std::fs::File;
use std::fs::{self};
use std::str::FromStr;

pub fn find_class_xml_files(path: &str) -> Vec<String>{
    let mut vec = Vec::<String>::new();
    let paths = fs::read_dir(path).unwrap();
    for path in paths {
        if path.is_ok() {
            let value = String::from_str(path.unwrap().path().to_str().unwrap()).unwrap();
            if value.ends_with(".xml")  && value.contains("class_"){
                vec.push(value);
            }
        }
    }
    vec
}
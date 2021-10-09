use crate::common::*;

use std::fs::File;
use std::io::BufReader;
use xml::reader::{EventReader, XmlEvent};

pub struct Function {
    access: Access,
    name: String,
    ret_type: Option<String>,
    ret_description: Option<String>,
    brief: Option<String>,
    description: Option<String>,
    parameters: Vec<Parameter>,
    location: Option<Location>,
}

impl Function {
    pub fn new(access: &Access) -> Function {
        Function {
            access: access.clone(),
            name: String::new(),
            ret_type: Option::None,
            ret_description: Option::None,
            brief: Option::None,
            description: Option::None,
            parameters: Vec::new(),
            location: Option::None,
        }
    }

    pub fn read(
        access: &Access,
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<Function, std::io::Error> {
        let mut func = Function::new(access);
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                }) => match name.local_name.as_str() {
                    "type" => func.ret_type = Some(read_characters_only(parser)?),
                    "name" => func.name = read_characters_only(parser)?,
                    "briefdescription" => func.brief = Some(read_characters_only(parser)?),
                    "detaileddescription" => {
                        println!("[INFO] Attribute detailed description not managed")
                    }
                    "location" => func.location = Some(Location::read(attributes)),
                    _ => {}
                },
                Ok(XmlEvent::EndElement { ref name }) => {
                    if name.local_name == "memberdef" {
                        break;
                    }
                }
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        e.clone(),
                    ))
                }
                _ => {}
            }
        }

        Ok(func)
    }
}

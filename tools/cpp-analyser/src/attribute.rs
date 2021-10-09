use crate::common::*;

use std::fs::File;
use std::io::BufReader;
use xml::reader::{EventReader, XmlEvent};


pub struct Attribute {
    access: Access,
    ctype: String,
    name: String,
    brief: Option<String>,
    detailed: Option<String>,
    location: Option<Location>,
}

impl Attribute{
    pub fn new(access: &Access) -> Attribute{
        Attribute {
            access : access.clone(),
            ctype: String::new(),
            name: String::new(),
            brief: Option::None,
            detailed: Option::None,
            location: Option::None,
        }
    }

    pub  fn read(
        access: &Access,
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<Attribute, std::io::Error> {

        let mut attr = Attribute::new(access);
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                }) => {
                    match name.local_name.as_str() {
                        "type" => attr.ctype = read_characters_only(parser)?,
                        "name" => attr.name = read_characters_only(parser)?,
                        "briefdescription" => attr.brief = Some(read_characters_only(parser)?),
                        "detaileddescription" => println!("[INFO] Attribute detailed description not managed"),
                        "location" =>attr.location = Some(Location::read(attributes)),
                            _ => {},
                    }
                }
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

        Ok(attr)
    }
}
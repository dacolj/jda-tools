use crate::common::*;

use std::fs::File;
use std::io::BufReader;
use xml::{
    attribute::OwnedAttribute,
    reader::{EventReader, XmlEvent},
};

#[derive(Debug)]
pub struct Attribute {
    pub access: Access,
    pub is_static: bool,
    pub ctype: String,
    pub name: String,
    pub brief: Option<String>,
    pub detailed: Option<String>,
    pub location: Option<Location>,
}

impl Attribute {
    pub fn new() -> Attribute {
        Attribute {
            access: Access::Private,
            is_static: false,
            ctype: String::new(),
            name: String::new(),
            brief: Option::None,
            detailed: Option::None,
            location: Option::None,
        }
    }

    pub fn read(
        parser: &mut EventReader<BufReader<File>>,
        xml_attributes: &Vec<OwnedAttribute>
    ) -> Result<Attribute, std::io::Error> {
        let mut attr = Attribute::new();

        attr.is_static = read_xml_attribute(xml_attributes, "static").unwrap_or_default() == "yes";
        attr.access = match read_xml_attribute(xml_attributes, "prot"){
            Some(val)=> access_from_str(val.as_str()).unwrap_or(Access::Private),
            None => Access::Private,
        };

        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                }) => match name.local_name.as_str() {
                    "type" => attr.ctype = read_characters_only(parser)?,
                    "name" => attr.name = read_characters_only(parser)?,
                    "briefdescription" => attr.brief = Some(read_description(parser)?),
                    "detaileddescription" => attr.detailed = Some(read_description(parser)?),
                    "location" => attr.location = Some(Location::read(attributes)),
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

        Ok(attr)
    }
}

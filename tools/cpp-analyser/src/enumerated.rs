use crate::common::*;

use std::fs::File;
use std::io::BufReader;
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug)]
pub struct Enumerated {
    pub name: String,
    pub full_name: String,
    pub brief: Option<String>,
    pub detailed: Option<String>,
    pub values: Vec<EnumValue>,
    pub location: Option<Location>,
}

#[derive(Debug)]
pub struct EnumValue {
    pub name: String,
    pub brief: Option<String>,
}

impl Enumerated {
    fn new() -> Enumerated {
        Enumerated {
            name: String::new(),
            full_name: String::new(),
            brief: Option::None,
            detailed: Option::None,
            values: Vec::new(),
            location: Option::None,
        }
    }

    pub fn read(parser: &mut EventReader<BufReader<File>>) -> Result<Enumerated, std::io::Error> {
        let mut enum_obj = Enumerated::new();
        let mut depth = 0;
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                }) => {
                    if depth != 0 {
                        depth += 1
                    } else {
                        match name.local_name.as_str() {
                            "qualifiedname" => {
                                enum_obj.full_name =
                                    read_characters_only(parser)?.unwrap_or_default()
                            }
                            "name" => {
                                enum_obj.name = read_characters_only(parser)?.unwrap_or_default()
                            }
                            "briefdescription" => enum_obj.brief = read_description(parser)?,
                            "location" => {
                                enum_obj.location = Some(Location::read(attributes));
                                depth += 1;
                            }
                            "enumvalue" => {
                                enum_obj.values.push(Enumerated::read_enum_value(parser)?);
                            }
                            _ => depth += 1,
                        }
                    }
                }
                Ok(XmlEvent::EndElement { ref name }) => {
                    if depth == 0 {
                        if name.local_name != "memberdef" {
                            println!(
                                "Inconsistency {}, expected memberdef",
                                name.local_name.as_str()
                            );
                        }
                        break;
                    }
                    depth -= 1;
                    //
                    //     break;
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

        Ok(enum_obj)
    }

    pub fn read_enum_value(
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<EnumValue, std::io::Error> {
        let mut value = EnumValue {
            name: String::new(),
            brief: Option::None,
        };

        let mut depth = 0;
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement { ref name, .. }) => {
                    if depth != 0 {
                        depth += 1
                    } else {
                        match name.local_name.as_str() {
                            "name" => {
                                value.name = read_characters_only(parser)?.unwrap_or_default()
                            }
                            "briefdescription" => value.brief = read_description(parser)?,
                            _ => depth += 1,
                        }
                    }
                }
                Ok(XmlEvent::EndElement { ref name }) => {
                    if depth == 0 {
                        if name.local_name != "enumvalue" {
                            println!(
                                "Inconsistency {}, expected enumvalue",
                                name.local_name.as_str()
                            );
                        }
                        break;
                    }
                    depth -= 1;
                    //
                    //     break;
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
        Ok(value)
    }
}

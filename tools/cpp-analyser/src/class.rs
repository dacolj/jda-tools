use crate::attribute::*;
use crate::common::*;
use crate::enumerated::*;
use crate::function::*;

use std::fs::File;
use std::io::BufReader;
use std::str;
use xml::attribute::OwnedAttribute;
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub brief: Option<String>,
    pub detailed: Option<String>,
    pub attributes: Vec<Attribute>,
    pub functions: Vec<Function>,
    pub enums: Vec<Enumerated>,
    pub location: Option<Location>,
    pub is_struct: bool,
}

impl Class {
    fn new() -> Class {
        return Class {
            name: String::new(),
            brief: Option::None,
            detailed: Option::None,
            attributes: Vec::new(),
            functions: Vec::new(),
            enums: Vec::new(),
            location: Option::None,
            is_struct: false,
        };
    }

    pub fn object_type(&self) -> &'static str {
        match self.is_struct {
            true => "struct",
            false => "class",
        }
    }

    fn read_compound_name(
        &mut self,
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<(), std::io::Error> {
        self.name = read_characters_only(parser)?.unwrap_or_default();
        Ok(())
    }

    fn get_kind_attr(attributes: &Vec<OwnedAttribute>) -> Result<String, std::io::Error> {
        match attributes.iter().find(|&r| r.name.local_name == "kind") {
            Some(val) => Ok(val.value.to_string().clone()),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No kind attribute found",
            )),
        }
    }

    pub fn read_attributes(
        &mut self,
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<(), std::io::Error> {
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                }) => {
                    if name.local_name == "memberdef" {
                        self.attributes.push(Attribute::read(parser, attributes)?);
                    }
                }
                Ok(XmlEvent::EndElement { ref name }) => {
                    if name.local_name == "sectiondef" {
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
        Ok(())
    }

    pub fn read_functions(
        &mut self,
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<(), std::io::Error> {
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                }) => {
                    if name.local_name == "memberdef" {
                        self.functions.push(Function::read(parser, attributes)?);
                    }
                }
                Ok(XmlEvent::EndElement { ref name }) => {
                    if name.local_name == "sectiondef" {
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
        Ok(())
    }

    pub fn read_types(
        &mut self,
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<(), std::io::Error> {
        let mut depth: i32 = 0;
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                }) => {
                    if depth != 0 {
                        depth += 1;
                    } else {
                        if name.local_name == "memberdef" {
                            let kind = Class::get_kind_attr(attributes)?;
                            if kind == "enum" {
                                let enumerated = Enumerated::read(parser)?;
                                self.enums.push(enumerated);
                            } else if kind == "typedef" {
                                depth += 1;
                            } else {
                                depth += 1;
                                println!("Unknown class type: {}", kind);
                            }
                        }
                    }
                }
                Ok(XmlEvent::EndElement { ref name }) => {
                    if depth == 0 {
                        if name.local_name != "sectiondef" {
                            println!("Invalid end section def");
                        }
                        break;
                    }
                    depth -= 1
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
        Ok(())
    }

    fn read_content(
        &mut self,
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<(), std::io::Error> {
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                }) => match name.local_name.as_str() {
                    "compoundname" => self.read_compound_name(parser)?,
                    "sectiondef" => {
                        let kind = Class::get_kind_attr(attributes)?;
                        if kind.contains("-attrib") {
                            self.read_attributes(parser)?;
                        } else if kind.contains("-func") {
                            self.read_functions(parser)?;
                        } else if kind.ends_with("-type") {
                            self.read_types(parser)?;
                        } else if kind == "friend" {
                        } else if kind == "signal" {
                            self.read_functions(parser)?;
                        } else if kind == "related" {
                        } else {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Unknown kind: {}", kind),
                            ));
                        }
                    }
                    "briefdescription" => self.brief = read_description(parser)?,
                    "detaileddescription" => self.detailed = read_description(parser)?,
                    "location" => self.location = Some(Location::read(attributes)),
                    _ => {}
                },
                Ok(XmlEvent::EndElement { ref name }) => {
                    if name.local_name == "compounddef" {
                        break;
                    }
                }
                Ok(XmlEvent::EndDocument) => break,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        e.clone(),
                    ))
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn read(filename: &str, is_struct: bool) -> Result<Class, std::io::Error> {
        let file = std::io::BufReader::new(File::open(filename)?);

        let mut parser = EventReader::new(file);

        let mut class = Class::new();
        class.is_struct = is_struct;

        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement { ref name, .. }) => {
                    if name.local_name == "compounddef" {
                        class.read_content(&mut parser)?;
                    }
                }
                Ok(XmlEvent::EndElement { .. }) => {}
                Ok(XmlEvent::EndDocument) => break,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        e.clone(),
                    ))
                }
                _ => {}
            }
        }

        Ok(class)
    }
}

use crate::attribute::*;
use crate::common::*;
use crate::function::*;

use std::fs::File;
use std::io::BufReader;
use std::str;
use xml::attribute::OwnedAttribute;
use xml::reader::{EventReader, XmlEvent};

pub struct Class {
    name: String,
    brief: Option<String>,
    detailed: Option<String>,
    attributes: Vec<Attribute>,
    functions: Vec<Function>,
}

impl Class {
    fn new() -> Class {
        return Class {
            name: String::new(),
            brief: Option::None,
            detailed: Option::None,
            attributes: Vec::new(),
            functions: Vec::new(),
        };
    }

    fn read_compound_name(
        &mut self,
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<(), std::io::Error> {
        self.name = read_characters_only(parser)?;
        Ok(())
    }

    fn read_function(
        &mut self,
        access: Access,
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<(), std::io::Error> {
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
        access: &Access,
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<(), std::io::Error> {
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement { ref name, .. }) => {
                    if name.local_name == "memberdef" {
                        self.attributes.push(Attribute::read(access, parser)?);
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
        access: &Access,
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<(), std::io::Error> {
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement { ref name, .. }) => {
                    if name.local_name == "memberdef" {
                        self.functions.push(Function::read(access, parser)?);
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
                }) => {
                    if name.local_name == "compoundname" {
                        self.read_compound_name(parser)?;
                    } else if name.local_name == "sectiondef" {
                        let kind = Class::get_kind_attr(attributes)?;
                        if kind == "public-attrib" {
                            self.read_attributes(&Access::Public, parser)?;
                        } else if kind == "protected-attrib" {
                            self.read_attributes(&Access::Protected, parser)?;
                        } else if kind == "private-attrib" {
                            self.read_attributes(&Access::Private, parser)?;
                        } else if kind == "public-func" {
                            self.read_functions(&Access::Public, parser)?;
                        } else if kind == "protected-func" {
                            self.read_functions(&Access::Protected, parser)?;
                        } else if kind == "private-func" {
                            self.read_functions(&Access::Private, parser)?;
                        } else {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Unknown kind: {}", kind),
                            ));
                        }
                    }
                }
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

    pub fn read(filename: &str) -> Result<Class, std::io::Error> {
        let mut file = std::io::BufReader::new(File::open(filename)?);

        let mut parser = EventReader::new(file);

        let mut class = Class::new();

        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                }) => {
                    if name.local_name == "compounddef" {
                        class.read_content(&mut parser)?;
                    }
                }
                Ok(XmlEvent::EndElement { ref name }) => {}
                Ok(XmlEvent::EndDocument) => break,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        e.clone(),
                    ))
                }
                _ => {}
            }
        };

        Ok(class)
    }

}

use crate::common::*;

use std::fs::File;
use std::io::BufReader;
use xml::reader::{EventReader, XmlEvent};
use xml::attribute::OwnedAttribute;

#[derive(Debug)]
pub struct Function {
    pub access: Access,
    pub name: String,
    pub is_static: bool,
    pub ret_type: Option<String>,
    pub ret_description: Option<String>,
    pub brief: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<Parameter>,
    pub location: Option<Location>,
}

impl Function {
    pub fn new() -> Function {
        Function {
            access: Access::Unknown,
            name: String::new(),
            is_static: false,
            ret_type: Option::None,
            ret_description: Option::None,
            brief: Option::None,
            description: Option::None,
            parameters: Vec::new(),
            location: Option::None,
        }
    }

    pub fn read_parameter_item(&mut self,
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<(), std::io::Error> {
        let mut param_name: Option<String> = Option::None;
        let mut direction:  Option<Direction> = Option::None;
        let mut description: Option<String> = Option::None;
        let mut depth = 0;
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                }) => match name.local_name.as_str() {
                    "parametername" => {
                        direction = match read_xml_attribute(attributes, "direction") {
                            Some(val) => direction_from_str(val.as_str()),
                            None => Option::None,
                        };
                        param_name = Some(read_characters_only(parser)?);
                    }
                    "parameterdescription" => description = Some(read_description(parser)?),
                    _ => depth += 1,
                },
                Ok(XmlEvent::EndElement { ref name }) => {
                    if depth == 0 {
                        if name.local_name != "parameteritem" {
                            println!("Inconsistency {}, expected parameteritem", name.local_name.as_str());
                        }
                        break;
                    }
                    depth -= 1;
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

        if param_name.is_some(){
            let mut param = match self.parameters.iter_mut().find(|r| r.name == param_name) {
                Some(val) => val,
                None => {
                    let mut p = Parameter::new();
                    p.name = param_name;
                    self.parameters.push(p);
                    self.parameters.last_mut().unwrap()
                }
            };
            param.description = description;
            param.direction = direction;
        }

        Ok(())
    }

    pub fn read_return(&mut self,
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<(), std::io::Error> {
        let mut depth = 0;
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                }) => match name.local_name.as_str() {
                    "para" => self.ret_description = Some(read_characters_only(parser)?),
                    _ => depth += 1,
                },
                Ok(XmlEvent::EndElement { ref name }) => {
                    if depth == 0 {
                        if name.local_name != "simplesect" {
                            println!("Inconsistency {}, expected simplesect", name.local_name.as_str());
                        }
                        break;
                    }
                    depth -= 1;
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

    pub fn read__detailed_description(&mut self,
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<(), std::io::Error> {
        let mut depth = 0;
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                }) => match name.local_name.as_str() {
                    "parameteritem" => self.read_parameter_item(parser)?,
                    "simplesect" => {
                        let kind = read_xml_attribute(attributes, "kind");
                        if kind.is_some() && kind.unwrap() == "return" {
                           self.read_return(parser)?;
                        }
                        else{
                            depth += 1;
                        }
                    }
                    _ => depth += 1,
                },
                Ok(XmlEvent::EndElement { ref name }) => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
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

    pub fn read_param(
        &mut self,
        parser: &mut EventReader<BufReader<File>>,
    ) -> Result<(), std::io::Error> {
        let mut depth = 0;
        let mut ctype: Option<String> = Option::None;
        let mut declname: Option<String> = Option::None;
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                }) => match name.local_name.as_str() {
                    "type" => ctype = Some(read_characters_only(parser)?),
                    "declname" => declname = Some(read_characters_only(parser)?),
                    _ => depth += 1,
                },
                Ok(XmlEvent::EndElement { ref name }) => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
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

        if declname.is_some() && ctype.is_some() {
            let mut param = match self.parameters.iter_mut().find(|r| r.name == declname) {
                Some(val) => val,
                None => {
                    let mut p = Parameter::new();
                    p.name = declname;
                    self.parameters.push(p);
                    self.parameters.last_mut().unwrap()
                }
            };
            param.ctype = Some(ctype.unwrap());
        }
        Ok(())
    }

    pub fn read(
        parser: &mut EventReader<BufReader<File>>,
        xml_attributes: &Vec<OwnedAttribute>
    ) -> Result<Function, std::io::Error> {

        let mut func = Function::new();
        func.is_static = read_xml_attribute(xml_attributes, "static").unwrap_or_default() == "yes";
        func.access = match read_xml_attribute(xml_attributes, "prot"){
            Some(val)=> access_from_str(val.as_str()).unwrap_or(Access::Private),
            None => Access::Private,
        };

        let mut depth = 0;
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                }) => {
                    if depth != 0 {
                        //println!("ooops {}", name.local_name.as_str());
                        depth += 1
                    }else {
                        match name.local_name.as_str() {
                            "type" => func.ret_type = Some(read_characters_only(parser)?),
                            "name" => func.name = read_characters_only(parser)?,
                            "briefdescription" => func.brief = Some(read_description(parser)?),
                            "detaileddescription" => func.read__detailed_description(parser)?,
                            "param" => func.read_param(parser)?,
                            "location" => {
                                func.location = Some(Location::read(attributes));
                                depth +=1;
                            },
                            _ => depth +=1,
                        }
                    }
                },
                Ok(XmlEvent::EndElement { ref name }) => {
                    if depth == 0{
                        if name.local_name != "memberdef" {
                            println!("Inconsistency {}, expected memberdef", name.local_name.as_str());
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

        Ok(func)
    }
}

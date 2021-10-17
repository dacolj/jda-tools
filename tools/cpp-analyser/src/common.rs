use std::fs::File;
use std::io::BufReader;
use xml::{
    attribute::OwnedAttribute,
    reader::{EventReader, XmlEvent},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Access {
    Unknown,
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Unknown,
    In,
    Out,
    InOut,
}

#[derive(Debug)]
pub struct Location {
    pub file: Option<String>,
    pub line: i32,
}

#[derive(Debug)]
pub struct Parameter {
    pub name: Option<String>,
    pub ctype: Option<String>,
    pub description: Option<String>,
    pub direction: Option<Direction>,
}

impl Parameter {
    pub fn new() -> Parameter {
        Parameter {
            name: Option::None,
            ctype: Option::None,
            description: Option::None,
            direction: Option::None,
        }
    }
}

pub fn read_characters_only(
    parser: &mut EventReader<BufReader<File>>,
) -> Result<String, std::io::Error> {
    let mut a: String = String::new();
    let mut depth = 0;
    loop {
        match parser.next() {
            Ok(XmlEvent::Characters(ref chars)) => {
                if depth == 0 {
                    a = chars.to_string().clone();
                }
            }
            Ok(XmlEvent::StartElement {  ref name, .. }) => {
                depth += 1;
            }
            Ok(XmlEvent::EndElement {  ref name }) => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            Ok(XmlEvent::CData(ref s)) => println!("CDATA {}", s),
            Ok(XmlEvent::Comment(ref s)) => println!("Comment {}", s),
            Ok(XmlEvent::Whitespace(ref s)) => {},// println!("Whitespace {}", s),
            Ok(XmlEvent::EndDocument) => println!("end doc"),
            Ok(XmlEvent::ProcessingInstruction { ref name, ref data }) => {
                println!("Processing instruction {} {:?}", name, data)
            }
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.clone(),
                ))
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "End element or value expected",
                ))
            }
        }
    }

    Ok(a)
}

pub fn read_description(
    parser: &mut EventReader<BufReader<File>>,
) -> Result<String, std::io::Error> {
    let mut a: String = String::new();
    loop {
        let mut depth = 0;
        match parser.next() {
            Ok(XmlEvent::StartElement {
                ref name,
                ref attributes,
                ..
            }) => {
                if name.local_name == "para" {
                    a = format!("{}\n{}", a, read_characters_only(parser)?);
                } else {
                    depth += 1;
                }
            }
            Ok(XmlEvent::EndElement { ref name }) => {
                if depth == 0 {
                    if name.local_name != "detaileddescription"
                        && name.local_name != "briefdescription"
                        && name.local_name != "parameterdescription"
                    {
                        println!(
                            "Inconsistency {}, expected briefdescription or detailed description",
                            name.local_name.as_str()
                        );
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

    Ok(a)
}

impl Location {
    fn new() -> Location {
        Location {
            file: Option::None,
            line: -1,
        }
    }

    pub fn read(attributes: &Vec<OwnedAttribute>) -> Location {
        let line = match attributes.iter().find(|&r| r.name.local_name == "line") {
            Some(val) => val.value.to_string().parse::<i32>().unwrap_or(-2),
            None => -1,
        };

        let file = match attributes.iter().find(|&r| r.name.local_name == "file") {
            Some(val) => Some(val.value.to_string().clone()),
            None => Option::None,
        };

        Location {
            file: file,
            line: line,
        }
    }
}

pub fn read_xml_attribute(attributes: &Vec<OwnedAttribute>, key: &str) -> Option<String> {
    match attributes.iter().find(|&r| r.name.local_name == key) {
        Some(val) => Option::Some(val.value.to_string().clone()),
        None => Option::None,
    }
}

pub fn direction_from_str(text: &str) -> Option<Direction> {
    match text {
        "in" => Some(Direction::In),
        "out" => Some(Direction::Out),
        "inout" => Some(Direction::InOut),
        _ => Option::None,
    }
}

pub fn access_from_str(text: &str) -> Option<Access> {
    if text.contains("public") {
        Some(Access::Public)
    } else if text.contains("private") {
        Some(Access::Private)
    } else if text.contains("protected") {
        Some(Access::Protected)
    } else {
        Option::None
    }
}

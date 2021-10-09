use std::fs::File;
use std::io::BufReader;
use xml::{
    attribute::OwnedAttribute,
    reader::{EventReader, XmlEvent},
};

#[derive(Debug, Clone, Copy)]
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

pub struct Location {
    file: Option<String>,
    line: i32,
}

pub struct Parameter {
    name: String,
    description: String,
    direction: Direction,
}

pub fn read_characters_only(
    parser: &mut EventReader<BufReader<File>>,
) -> Result<String, std::io::Error> {
    let mut a: String = String::new();
    loop {
        let mut depth = 0;
        match parser.next() {
            Ok(XmlEvent::Characters(ref chars)) => {
                if depth == 0 {
                    a = chars.to_string().clone()
                }
            }
            Ok(XmlEvent::StartElement { .. }) => {
                depth += 1;
            }
            Ok(XmlEvent::EndElement { .. }) => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            Ok(XmlEvent::CData(ref s)) => println!("CDATA {}", s),
            Ok(XmlEvent::Comment(ref s)) => println!("Comment {}", s),
            Ok(XmlEvent::Whitespace(ref s)) => println!("Whitespace {}", s),
            Ok(XmlEvent::EndDocument) => println!("end doc"),
            Ok(XmlEvent::ProcessingInstruction { ref name, ref data}) => println!("Whitespace {} {:?}", name, data),
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

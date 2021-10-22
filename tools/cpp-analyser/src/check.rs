use crate::attribute::*;
use crate::class::*;
use crate::common::*;
use std::fs::File;
use std::io::prelude::*;
pub struct CsvErrorWriter {
    pub file: File,
}
pub trait ErrorWriter {
    fn append(&mut self, error: String, location: &Option<Location>) -> Result<(), std::io::Error>;
}

impl CsvErrorWriter {
    pub fn new(filename: &str) -> Result<CsvErrorWriter, std::io::Error> {
        Ok(CsvErrorWriter {
            file: File::create(filename)?,
        })
    }
}
impl ErrorWriter for CsvErrorWriter {
    fn append(&mut self, error: String, location: &Option<Location>) -> Result<(), std::io::Error> {
        self.file.write_fmt(format_args!("{};", error))?;
        match location {
            Some(loc) => self
                .file
                .write_fmt(format_args!("{};{};\n", loc.file, loc.line))?,
            None => self.file.write_all(b";;\n")?,
        };
        Ok(())
    }
}

pub struct VisualErrorWriter {}

impl ErrorWriter for VisualErrorWriter {
    fn append(&mut self, error: String, location: &Option<Location>) -> Result<(), std::io::Error> {
        match location {
            Some(loc) => println!(
                "{filename}({line}):{level}:{text}\n",
                filename = loc.file,
                line = loc.line,
                level = "warning",
                text = error
            ),
            None => println!(":{level}:{text}\n", level = "warning", text = error),
        };

        Ok(())
    }
}

fn check_attribute_name(attribute: &Attribute, class_name: &str) {
    if attribute.is_static {
        if attribute.ctype.starts_with("const") {
            if !attribute.name.chars().next().unwrap().is_uppercase() {
                println!(
                    "Static const attribute {} of class {} should start with an upper case letter",
                    attribute.name, class_name
                );
            }
        } else {
            if !attribute.name.starts_with("_s") {
                println!(
                    "Static attribute {} of class {} should start with a s_",
                    attribute.name, class_name
                );
            }
        }
    } else {
        if !attribute.name.starts_with("m_") {
            println!(
                "Attribute {} of class {} should start with a m_",
                attribute.name, class_name
            );
        }
    }
}

pub fn check_class(class: &Class, error_writer: &mut ErrorWriter) -> Result<(), std::io::Error> {
    println!("Checking class {}...", class.name);
    for a in &class.attributes {
        check_attribute_name(&a, class.name.as_str());
        if a.brief == Option::None {
            error_writer.append(
                format!(
                    "Attribute {} of class {} should have a description",
                    a.name, class.name
                ),
                &a.location,
            )?;
        }
        if a.access == Access::Public {
            // add is_const
            error_writer.append(
                format!(
                    "Attribute {} of class {} should not be public",
                    a.name, class.name
                ),
                &a.location,
            )?;
        }
    }

    for f in &class.functions {
        if f.description == Option::None && f.brief == Option::None {
            error_writer.append(
                format!(
                    "Function {} of class {} should have a description",
                    f.name, class.name
                ),
                &f.location,
            )?;
        }
        if f.ret_type.is_some() {
            let ret_type = f.ret_type.as_ref().unwrap();
            if !ret_type.contains("void") && ret_type != "" && f.ret_description.is_none() {
                error_writer.append(
                    format!(
                    "Function {} of class {} should have a description for the returned type ({})",
                    f.name, class.name, ret_type
                ),
                    &f.location,
                )?;
            }
        }

        for p in &f.parameters {
            if p.ctype == Option::None {
                error_writer.append(
                    format!(
                        "Parameter {} of function {} of class {} doesn't exist",
                        p.name.as_ref().unwrap(),
                        f.name,
                        class.name
                    ),
                    &f.location,
                )?;
            } else if p.description == Option::None {
                error_writer.append(
                    format!(
                        "Parameter {} of function {} of class {} should have a description",
                        p.name.as_ref().unwrap(),
                        f.name,
                        class.name
                    ),
                    &f.location,
                )?;
            } else if !p.name.as_ref().unwrap().starts_with("p_") {
                error_writer.append(
                    format!(
                        "Parameter {} of function {} of class {} should start with p_",
                        p.name.as_ref().unwrap(),
                        f.name,
                        class.name
                    ),
                    &f.location,
                )?;
            }
        }
    }

    Ok(())
}

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
    fn name(&self) -> &'static str;
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

    fn name(&self) -> &'static str {
        "CSV"
    }
}

pub struct VisualStudioErrorWriter {}

impl ErrorWriter for VisualStudioErrorWriter {
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

    fn name(&self) -> &'static str {
        "Visual-Studio"
    }
}

fn check_attribute_name(
    error_writer: &mut dyn ErrorWriter,
    attribute: &Attribute,
    class: &Class,
) -> Result<(), std::io::Error> {
    if attribute.is_static {
        if attribute.ctype.starts_with("const") {
            if !attribute.name.chars().next().unwrap().is_uppercase() {
                error_writer.append(
                    format!(
                        "Static const attribute {} of {} {} should start with an upper case letter",
                        attribute.name,
                        class.object_type(),
                        class.name
                    ),
                    &attribute.location,
                )?;
            }
        } else {
            if !attribute.name.starts_with("s_") {
                error_writer.append(
                    format!(
                        "Static attribute {} of {} {} should start with a s_",
                        attribute.name,
                        class.object_type(),
                        class.name
                    ),
                    &attribute.location,
                )?;
            }
        }
    } else {
        if !class.is_struct && !attribute.name.starts_with("m_") {
            error_writer.append(
                format!(
                    "Attribute {} of class {} should start with a m_",
                    attribute.name, class.name
                ),
                &attribute.location,
            )?;
        }
        //  else if class.is_struct && attribute.name.starts_with("m_") {
        //     println!(
        //         "Attribute {} of struct {} should not start with a m_",
        //         attribute.name, class.name
        //     );
        // }
    }
    Ok(())
}

pub fn check_class(
    class: &Class,
    error_writer: &mut dyn ErrorWriter,
) -> Result<(), std::io::Error> {
    //println!("Checking class {}...", class.name);

    assert!(!class.name.is_empty());

    let pos = match class.name.rfind(':'){
       Some(p) => std::cmp::min(p + 1, class.name.len() - 1),
       None => 0,
    };
    if !class.name.chars().nth(pos).unwrap().is_uppercase() {
        error_writer.append(
            format!(
                "Name of class {} should start with an upper case letter",
                class.name
            ),
            &class.location,
        )?;
    }
    if class.brief.is_none() && class.detailed.is_none() {
        error_writer.append(
            format!("Class {} has no description", class.name),
            &class.location,
        )?;
    }

    for enumerated in &class.enums {
        assert!(!enumerated.name.is_empty());

        let pos = match enumerated.name.rfind(':'){
        Some(p) => std::cmp::min(p + 1, enumerated.name.len() - 1),
        None => 0,
        };

        if !enumerated.name.chars().nth(pos).unwrap().is_uppercase() {
            error_writer.append(
                format!(
                    "Name of enum {} should start with an upper case letter",
                    enumerated.full_name
                ),
                &enumerated.location,
            )?;
        }

        if enumerated.brief.is_none() {
            error_writer.append(
                format!("Enum {} has no description", enumerated.full_name),
                &enumerated.location,
            )?;
        }

        for value in &enumerated.values {
            if value.brief.is_none() {
                error_writer.append(
                    format!(
                        "Value {} of enum {} has not decription",
                        value.name, enumerated.full_name
                    ),
                    &enumerated.location,
                )?;
            }
            if !value.name.chars().next().unwrap().is_uppercase() {
                error_writer.append(
                    format!(
                        "Value {} of enum {} should start with an upper case letter",
                        value.name, enumerated.full_name
                    ),
                    &enumerated.location,
                )?;
            }
        }
    }

    for a in &class.attributes {
        check_attribute_name(error_writer, &a, &class)?;
        if a.brief == Option::None && a.detailed == Option::None {
            error_writer.append(
                format!(
                    "Attribute {} of class {} should have a description",
                    a.name, class.name
                ),
                &a.location,
            )?;
        }
        if a.access == Access::Public && !class.is_struct {
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
        if f.detailed == Option::None && f.brief == Option::None {
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

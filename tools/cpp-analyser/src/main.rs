mod attribute;
mod class;
mod common;
mod function;
use class::*;

fn main() {
    println!("Hello, world!");

    let a = "";
    let c = Class::read(r"C:\Workspace\Jean\Other\DoxygenSampleProject\xml\class_foo.xml");
    match c {
        Ok(v) => println!("working with version:"),
        Err(e) => println!("error parsing header: {:?}", e),
    };
    println!("ok");
    println!("ok2");
}

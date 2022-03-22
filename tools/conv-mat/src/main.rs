use ::std::*;
use clap::{Arg, Command};
use clipboard_win::{formats, Clipboard, Setter};
use serde_json::Value;
use std::io::prelude::*;
use std::str::FromStr;

struct Matrix {
    data: [f64; 16],
}

impl Matrix {
    fn print_blender(&self) -> String {
        return format!(
            "Matrix([[{}, {}, {}, {}],[{}, {}, {}, {}],[{}, {}, {}, {}],[{}, {}, {}, {}]])",
            self.data[0],
            self.data[1],
            self.data[2],
            self.data[3],
            self.data[4],
            self.data[5],
            self.data[6],
            self.data[7],
            self.data[8],
            self.data[9],
            self.data[10],
            self.data[11],
            self.data[12],
            self.data[13],
            self.data[14],
            self.data[15]
        );
    }

    fn print_cc(&self) -> String {
        return format!(
            "{} {} {} {}\n{} {} {} {}\n{} {} {} {}\n{} {} {} {}",
            self.data[0],
            self.data[1],
            self.data[2],
            self.data[3],
            self.data[4],
            self.data[5],
            self.data[6],
            self.data[7],
            self.data[8],
            self.data[9],
            self.data[10],
            self.data[11],
            self.data[12],
            self.data[13],
            self.data[14],
            self.data[15]
        );
    }

    fn print_array(&self) -> String {
        return format!(
            "[{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}]",
            self.data[0],
            self.data[1],
            self.data[2],
            self.data[3],
            self.data[4],
            self.data[5],
            self.data[6],
            self.data[7],
            self.data[8],
            self.data[9],
            self.data[10],
            self.data[11],
            self.data[12],
            self.data[13],
            self.data[14],
            self.data[15]
        );
    }

    fn print_numpy(&self) -> String {
        return format!(
            "np.array([[{}, {}, {}, {}],[{}, {}, {}, {}],[{}, {}, {}, {}],[{}, {}, {}, {}]])",
            self.data[0],
            self.data[1],
            self.data[2],
            self.data[3],
            self.data[4],
            self.data[5],
            self.data[6],
            self.data[7],
            self.data[8],
            self.data[9],
            self.data[10],
            self.data[11],
            self.data[12],
            self.data[13],
            self.data[14],
            self.data[15]
        );
    }

    fn print_json(&self) -> String {
        return format!(
            "{{\"colX\":{{\"i\":{}, \"j\":{}, \"k\":{} }}, \"colY\":{{\"i\":{}, \"j\":{}, \"k\":{} }}, \"colZ\":{{\"i\":{}, \"j\":{}, \"k\":{} }}, \"translation\":{{\"i\":{}, \"j\":{}, \"k\":{} }} }}",
            self.data[0],
            self.data[4],
            self.data[8],
            self.data[1],
            self.data[5],
            self.data[9],
            self.data[2],
            self.data[6],
            self.data[10],
            self.data[3],
            self.data[7],
            self.data[11]
        );
    }

    fn new(
        d00: f64,
        d01: f64,
        d02: f64,
        d03: f64,
        d10: f64,
        d11: f64,
        d12: f64,
        d13: f64,
        d20: f64,
        d21: f64,
        d22: f64,
        d23: f64,
        d30: f64,
        d31: f64,
        d32: f64,
        d33: f64,
    ) -> Matrix {
        Matrix {
            data: [
                d00, d01, d02, d03, d10, d11, d12, d13, d20, d21, d22, d23, d30, d31, d32, d33,
            ],
        }
    }
    fn transpose(&self) -> Matrix {
        let d = &self.data;
        Matrix {
            data: [
                d[0], d[4], d[8], d[12], d[1], d[5], d[9], d[13], d[2], d[6], d[10], d[14], d[3],
                d[7], d[11], d[15],
            ],
        }
    }

    fn identity() -> Matrix {
        Matrix {
            data: [
                1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
            ],
        }
    }

    fn from_vec(vec: Vec<f64>) -> Option<Matrix> {
        if vec.len() != 12 && vec.len() != 16 {
            return Option::None;
        }

        let mut mat = Matrix::identity();
        let mut id = 0;
        for val in vec.iter() {
            mat.data[id] = *val;
            id += 1;
        }
        return Some(mat);
    }
}

fn try_text(data: &str) -> Option<Matrix> {
    let esc_data: String = data
        .chars()
        .map(|x| match x {
            ',' => ' ',
            '[' => ' ',
            ']' => ' ',
            '(' => ' ',
            ')' => ' ',
            '\n' => ' ',
            '\r' => ' ',
            _ => x,
        })
        .collect();

    let mut vec: Vec<f64> = Vec::new();
    vec.reserve(16);
    for s in esc_data.split(' ') {
        if s.len() != 0 {
            match s.parse::<f64>() {
                Ok(val) => vec.push(val),
                Err(_) => return Option::None,
            }
        }
    }
    Matrix::from_vec(vec)
}

fn try_json(data: &str) -> Option<Matrix> {
    let res = match data.chars().nth(0).unwrap() != '{' {
        true => serde_json::from_str(format!("{{{}}}", data).as_str()),
        false => serde_json::from_str(data),
    };

    if res.is_err() {
        return Option::None;
    }

    let v: Value = res.unwrap();

    match v.as_object() {
        Some(val) => {
            if val.contains_key("colX")
                && val.contains_key("colY")
                && val.contains_key("colZ")
                && val.contains_key("translation")
            {
                let col_x = &val["colX"];
                let col_y = &val["colY"];
                let col_z = &val["colZ"];
                let col_tr = &val["translation"];
                return Some(Matrix::new(
                    col_x["i"].as_f64()?,
                    col_y["i"].as_f64()?,
                    col_z["i"].as_f64()?,
                    col_tr["i"].as_f64()?,
                    col_x["j"].as_f64()?,
                    col_y["j"].as_f64()?,
                    col_z["j"].as_f64()?,
                    col_tr["j"].as_f64()?,
                    col_x["k"].as_f64()?,
                    col_y["k"].as_f64()?,
                    col_z["k"].as_f64()?,
                    col_tr["k"].as_f64()?,
                    0.,
                    0.,
                    0.,
                    1.,
                ));
            }
        }
        None => return Option::None,
    }
    Option::None
}

fn get_stdin(msg: &str) -> String {
    fn print_new_line(_: io::Error) {
        print!("\n");
    }

    let mut buffer = String::new();
    print!("{} ", msg);
    io::stdout().flush().unwrap_or_else(print_new_line);
    let stdin = io::stdin();
    stdin.read_line(&mut buffer).unwrap_or_default();
    let mut res = buffer.to_string();
    if res.ends_with('\n') {
        res.pop();
        if res.ends_with('\r') {
            res.pop();
        }
    }
    res
}

fn main() {
    let matches = Command::new("conv-mat")
        .arg(
            Arg::new("output")
                .help("Output format")
                .short('o')
                .takes_value(true),
        )
        .arg(
            Arg::new("input")
                .help("Input text")
                .short('i')
                .takes_value(true)
                .allow_hyphen_values(true),
        )
        .arg(
            Arg::new("prefix")
                .help("Prefix")
                .short('p')
                .takes_value(true),
        )
        .arg(
            Arg::new("clipboard")
                .help("Copy output to cliboard")
                .short('c')
                .takes_value(false),
        )
        .arg(
            Arg::new("transpose")
                .help("Transpose matrix")
                .short('t')
                .takes_value(false),
        )
        .get_matches();

    let data = match matches.value_of("input") {
        Some(val) => String::from_str(val).unwrap(),
        None => {
            let mut data = String::new();
            loop {
                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer).unwrap();
                let tmp_buff = String::from_str(buffer.as_str().trim()).unwrap(); //replace("\r\n", "");
                if tmp_buff.is_empty() {
                    break;
                }
                data.push_str(buffer.as_str());
            }
            data
        }
    };

    let mut a = match try_json(data.as_str()) {
        Some(mat) => mat,
        None => match try_text(data.as_str()) {
            Some(mat) => mat,
            None => {
                println!("Cannot parse given text");
                return;
            }
        },
    };
    if matches.is_present("transpose") {
        a = a.transpose();
    }

    let output_format: String = match matches.value_of("output") {
        Some(val) => val.to_lowercase(),
        None => get_stdin("Output format?").to_lowercase(),
    };

    let out_mat = match output_format.as_str() {
        "blender" => a.print_blender(),
        "np" | "numpy" => a.print_numpy(),
        "json" => a.print_json(),
        "arr" | "array" => a.print_array(),
        "cc" => a.print_cc(),
        _ => {
            println!("Unknown format {}", output_format);
            a.print_cc()
        }
    };

    let out_str = match matches.value_of("prefix") {
        Some(val) => format!("{}{}", val, out_mat),
        None => out_mat,
    };

    println!("{}", out_str);

    if matches.is_present("clipboard") {
        let _clip = Clipboard::new_attempts(10).expect("Open clipboard");
        formats::Unicode
            .write_clipboard(&out_str.as_str())
            .expect("Write sample");
    }
}

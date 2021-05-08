use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::u32;

use clap::{App, Arg};

fn parse_file(file_name: &str) -> HashMap<u32, u32> {
    let mut out_map: HashMap<u32, u32> = HashMap::new();
    let disasm_regex = Regex::new(
        r"([0-9a-fA-F]+)\s([0-9a-fA-F]{8})\s([0-9a-fA-F]{8})\s([0-9a-fA-F]{8})\s([0-9a-fA-F]{8})",
    )
    .unwrap();
    let mut contents = String::new();

    let mut f = File::open(file_name).unwrap();
    f.read_to_string(&mut contents).unwrap();
    for line in (&contents).lines() {
        let captures = disasm_regex.captures(line);
        match captures {
            Some(c) => {
                //println!("{:#?}", c);
                // parse starting address
                let start_address = u32::from_str_radix(c.get(1).unwrap().as_str(), 16).unwrap();
                for i in 2..6 {
                    out_map.insert(
                        start_address + 4 * (i - 2),
                        u32::from_str_radix(c.get(i as usize).unwrap().as_str(), 16).unwrap(),
                    );
                }
            }
            None => {}
        }
    }
    out_map
}

fn dump(code_map: &mut HashMap<u32, u32>, out_file: Option<&str>, to_json: bool) {
    let mut keys: Vec<u32> = Vec::new();
    let mut contents = String::new();
    for k in code_map.keys() {
        keys.push(*k);
    }
    keys.sort();

    if to_json {
        let last_index = keys[keys.len() - 1] + 4;
        keys.push(last_index);
        code_map.insert(last_index, 0xdeadbeef);

        contents.push_str("[\n");
        for i in 0..keys.len() {
            contents.push_str(&format!(
                "{}{}\n",
                *((code_map.get_key_value(&(keys[i])).unwrap()).1),
                if i == keys.len() - 1 { "" } else { "," }
            ));
        }
        contents.push_str("]");
    } else {
        contents.push_str(&format!("{}\n", keys.len()));
        for k in keys {
            contents.push_str(&format!("{}\n", *((code_map.get_key_value(&k).unwrap()).1)));
        }
    }

    match out_file {
        Some(fname) => {
            let mut file = File::create(Path::new(fname)).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        None => {
            println!("{}", contents);
        }
    }
}

fn main() {
    let matches = App::new("compile_to_rout")
        .version("1.0")
        .about("takes in a dump file and outputs the file in .rout format")
        .arg(
            Arg::with_name("infile")
                .short("i")
                .long("infile")
                .takes_value(true)
                .required(true)
                .help("path of input dump file"),
        )
        .arg(
            Arg::with_name("outfile")
                .short("o")
                .long("outfile")
                .takes_value(true)
                .required(false)
                .help("path of output file"),
        )
        .arg(
            Arg::with_name("json")
                .short("j")
                .long("json")
                .takes_value(false)
                .required(false)
                .help("output to a json list"),
        )
        .get_matches();

    let source_file = matches.value_of("infile").unwrap();

    let mut out_map = parse_file(&source_file);
    dump(
        &mut out_map,
        matches.value_of("outfile"),
        matches.is_present("json"),
    );
}

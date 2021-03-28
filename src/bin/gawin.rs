use ansi_term::Color;
use ansi_term::Colour::{Cyan, Green, Purple, Red};
use clap::{App, Arg};
use ctftools::readers;
use std::fs;
use std::{env, panic};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

fn main() {
    #[cfg(target_os = "windows")]
    let _ = ansi_term::enable_ansi_support();

    let mut app = App::new("Gawin")
        .version(VERSION)
        .author(AUTHORS)
        .about("Dump various file headers")
        .arg(
            Arg::with_name("type")
                .short("t")
                .long("type")
                .value_name("FILETYPE")
                .help("What file type to interpret the input as")
                .multiple(true)
                .default_value("all")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("input")
                .value_name("FILE")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        );

    if env::args_os().len() < 2 {
        let _ = app.print_help();
        return;
    }

    let matches = app.get_matches();

    let input = matches.value_of("input").unwrap();

    print!("Dumping {}... ", Color::Cyan.paint(input));
    let bytes = match fs::read(input) {
        Ok(b) => {
            println!("{} byte(s)", b.len());
            b
        }
        Err(e) => {
            println!("{}\n{}", Red.paint("Failed!"), e);
            return;
        }
    };

    let arg_types: Vec<&str> = matches.values_of("type").unwrap().collect();
    let types = if arg_types.contains(&"all") {
        readers::ALL.to_vec()
    } else {
        arg_types
    };

    for reader_type in types {
        match readers::get_reader(reader_type) {
            None => println!("Unknown reader {}... Skipping!", reader_type),
            Some(read) => {
                println!(
                    "\n===[ {} ]=======================================\n",
                    Cyan.paint(format!("{:<8}", reader_type.to_uppercase()))
                );
                let result = panic::catch_unwind(|| match read(&bytes) {
                    Ok((start, end)) => println!(
                        "{} {} to {}!",
                        Green.paint("Matched"),
                        Purple.paint(start.to_string()),
                        Purple.paint(end.to_string())
                    ),
                    Err(e) => println!("{}: {}", "Failed", e),
                });
                match result {
                    Ok(()) => {}
                    Err(_) => println!("{}", "Reader error"),
                }
            }
        }
    }
}

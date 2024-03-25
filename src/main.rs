#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::Read;
use flate2::read::ZlibDecoder;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p' )]
        pretty_print: bool,

        object_hash: String,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Init=> {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        },
        Command::CatFile { pretty_print, object_hash } => {
            //get the first two characters of the hash
            let prefix = &object_hash[0..2];
            //get the rest of the hash
            let suffix = &object_hash[2..];
            //get the file path
            let file_path = format!(".git/objects/{}/{}", prefix, suffix);
            //read the file
            let contents = fs::read(file_path).unwrap();
            //decompress the file
            let mut decoded_data_decoder =  ZlibDecoder::new(contents.as_slice());
            let mut decoded_data = String::new();
            decoded_data_decoder.read_to_string(&mut decoded_data).unwrap();

            if pretty_print {
                println!("{}", decoded_data);
            } else {
                println!("{:?}", decoded_data);
            }
        }
    }
}

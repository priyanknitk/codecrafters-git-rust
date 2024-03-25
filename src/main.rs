use flate2::read::ZlibDecoder;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::Read;

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
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Init => init(),
        Command::CatFile {
            pretty_print,
            object_hash,
        } => cat_file(&object_hash, pretty_print),
    }
}

fn init() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
    println!("Initialized git directory")
}

fn cat_file(object_hash: &str, pretty_print: bool) {
    //get the first two characters of the hash
    let prefix = &object_hash[0..2];
    //get the rest of the hash
    let suffix = &object_hash[2..];
    //get the file path
    let file_path = format!(".git/objects/{}/{}", prefix, suffix);
    //read the file
    let contents = fs::read(file_path).unwrap();
    //decompress the file
    let mut decoded_data_decoder = ZlibDecoder::new(contents.as_slice());
    let mut decoded_data = String::new();
    decoded_data_decoder
        .read_to_string(&mut decoded_data)
        .unwrap();

    //remove the header from the decoded data
    let decoded_data = decoded_data.splitn(2, '\0').collect::<Vec<&str>>()[1];

    if pretty_print {
        print!("{}", decoded_data);
    } else {
        print!("{:?}", decoded_data);
    }
}

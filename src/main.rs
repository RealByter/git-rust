use clap::{ArgGroup, Parser, Subcommand};
use flate2::read::ZlibDecoder;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::Read;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    #[command(group(ArgGroup::new("options").required(true).multiple(false).args(["pretty_print", "show_type", "show_size"])))]
    CatFile {
        #[clap(short = 'p', conflicts_with_all = &["show_type", "show_size"], long_help = "")]
        pretty_print: bool,
        #[clap(short = 't', conflicts_with_all = &["pretty_print", "show_size"], long_help = "")]
        show_type: bool,
        #[clap(short = 's', conflicts_with_all = &["pretty_print", "show_type"], long_help = "")]
        show_size: bool,
        object_hash: String,
    },
}

fn main() {
    // let args: Vec<String> = env::args().collect();
    let args = Args::parse();
    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory");
        }
        Command::CatFile {
            pretty_print,
            show_type,
            show_size,
            object_hash,
        } => {
            let content = fs::read(format!(
                ".git/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..]
            ))
            .unwrap();
            let mut d = ZlibDecoder::new(&content as &[u8]);
            let mut s = String::new();
            d.read_to_string(&mut s).unwrap();
            let p: Vec<&str> = s.split('\0').collect();
            if pretty_print {
                println!("{}", p[1]);
            } else if show_type {
                let p: Vec<&str> = p[0].split(' ').collect();
                println!("{}", p[0]);
            } else if show_size {
                let p: Vec<&str> = p[0].split(' ').collect();
                println!("{}", p[1]);
            }
        }
    }
}

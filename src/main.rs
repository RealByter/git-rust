use clap::{ArgGroup, Parser, Subcommand};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use hex;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{ErrorKind, Read, Write};

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
    HashObject {
        #[clap(short = 'w', required = true)]
        write: bool,
        file: String,
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
            // Read the content of the object
            let content = fs::read(format!(
                ".git/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..]
            ))
            .unwrap();

            // Zlib decode
            let mut d = ZlibDecoder::new(&content as &[u8]);
            let mut s = String::new();
            d.read_to_string(&mut s).unwrap();

            // Output the correct information
            let p: Vec<&str> = s.split('\0').collect();
            if pretty_print {
                print!("{}", p[1]);
            } else if show_type {
                let p: Vec<&str> = p[0].split(' ').collect();
                print!("{}", p[0]);
            } else if show_size {
                let p: Vec<&str> = p[0].split(' ').collect();
                print!("{}", p[1]);
            }
        }
        Command::HashObject { write, file } => {
            if write {
                // Get the content and add the headers
                let file_content = fs::read_to_string(file).unwrap();
                let content = format!("blob {}\0{}", file_content.len(), file_content);

                // Create the sha1 hash
                let mut hasher = Sha1::new();
                hasher.update(&content);
                let result = hasher.finalize();
                let hex_hash = hex::encode(result);

                // Zlib encode the content
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&content.as_bytes()).unwrap();
                let zlib_hash = encoder.finish().unwrap();

                // if the folder exists, you can still write to it
                match fs::create_dir(format!(".git/objects/{}", &hex_hash[..2])) {
                    Ok(()) => {
                        fs::write(
                            format!(".git/objects/{}/{}", &hex_hash[..2], &hex_hash[2..]),
                            zlib_hash,
                        )
                        .unwrap();
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::AlreadyExists {
                            fs::write(
                                format!(".git/objects/{}/{}", &hex_hash[..2], &hex_hash[2..]),
                                zlib_hash,
                            )
                            .unwrap();
                        } else {
                            panic!("Failed to write to file");
                        }
                    }
                }

                print!("{}", hex_hash);
            }
        }
    }
}

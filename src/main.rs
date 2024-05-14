use flate2::read::ZlibDecoder;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::Read;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
        println!("Initialized git directory")
    } else if args[1] == "cat-file" {
        let content =
            fs::read(format!(".git/objects/{}/{}", &args[3][..2], &args[3][2..])).unwrap();
        let mut d = ZlibDecoder::new(&content as &[u8]);
        let mut s = String::new();
        d.read_to_string(&mut s).unwrap();
        if args[2] == "-p" {
            let p: Vec<&str> = s.split('\0').collect();
            print!("{}", p[1]);
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}

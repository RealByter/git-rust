pub mod git {
    use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
    use hex;
    use sha1::{Digest, Sha1};
    use std::fs;
    use std::io::{ErrorKind, Read, Write};
    use std::path::Path;

    pub enum ObjectType {
        Blob,
        Tree,
    }

    pub fn init() {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
        println!("Initialized git directory");
    }

    pub fn cat_file(pretty_print: bool, show_type: bool, show_size: bool, object_hash: String) {
        // Read the content of the object
        let content = fs::read(format!(
            ".git/objects/{}/{}",
            &object_hash[..2],
            &object_hash[2..]
        ))
        .unwrap();

        // Zlib decode
        let mut decoder = ZlibDecoder::new(&content as &[u8]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();

        // print!("{}", unsafe {
        //     &String::from_utf8_unchecked(decompressed.clone())
        // });

        let parts: Vec<_> = decompressed
            .split(|&b| b == 0)
            .map(|p| p.to_vec())
            .collect();

        // Output the correct information
        if pretty_print {
            let content = String::from_utf8_lossy(&parts[1]);
            // print!("{}", p[1]);
            print!("{}", content);
        } else if show_type {
            // let p: Vec<&str> = p[0].split(' ').collect();
            // print!("{}", p[0]);
            let header = String::from_utf8_lossy(&parts[0]);
            let header_parts: Vec<_> = header.split(' ').collect();
            print!("{}", header_parts[0]);
        } else if show_size {
            let header = String::from_utf8_lossy(&parts[0]);
            let header_parts: Vec<_> = header.split(' ').collect();
            print!("{}", header_parts[1]);
        }
    }

    fn hash_object(file: &Path, object_type: ObjectType) -> String {
        // Get the content and add the headers
        let content: String;
        match object_type {
            ObjectType::Blob => {
                let file_content = fs::read_to_string(file).unwrap();
                content = format!("{} {}\0{}", "blob", file_content.len(), file_content);
            }
            ObjectType::Tree => {
                let file_content = recursively_write_to_tree(Path::new(&file));
                content = format!("{} {}\0{}", "tree", file_content.len(), file_content);
            }
        }

        // Create the sha1 hash
        let mut hasher = Sha1::new();
        hasher.update(&content);
        let result = hasher.finalize();
        let hex_hash = hex::encode(&result);

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

        hex_hash
    }

    pub fn hash_blob(write: bool, file: &str, object_type: ObjectType) {
        if write {
            print!("{}", hash_object(Path::new(file), object_type))
        }
    }

    pub fn ls_tree() {
        println!("Not supported yet");
    }

    pub fn write_tree() {
        print!("{}", hash_object(Path::new("."), ObjectType::Tree));
    }

    fn recursively_write_to_tree(path: &Path) -> String {
        let mut content = String::new();
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let file_name = path.file_name().unwrap();
                    if path.is_dir() {
                        if file_name == ".git" {
                            continue;
                        } else {
                            content += "040000 ";
                        }
                    } else {
                        content += "100644 ";
                    }
                    content += file_name.to_str().unwrap();
                    content += "\0";
                    // content += &String::from_utf8_lossy(&recursively_write_to_tree(&path));
                    content += unsafe {
                        &String::from_utf8_unchecked(
                            hex::decode(hash_object(path.as_path(), ObjectType::Tree)).unwrap(),
                        )
                    }
                }
            }
        }

        content
    }
}

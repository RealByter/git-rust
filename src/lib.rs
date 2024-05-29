pub mod git {
    use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
    use hex;
    use sha1::{Digest, Sha1};
    use std::io::{ErrorKind, Read, Write};
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::{env, fs};

    const FILE: &str = "100644";
    const FOLDER: &str = "040000";
    const OBJECT_TYPE_LENGTH: usize = 4;
    const TREE_OBJ_TYPE_LENGTH: usize = 6;
    const SHA_HASH_LENGTH: usize = 20;
    const SPACES_PER_TAB: usize = 2;

    enum ObjectType {
        Blob,
        Tree,
        Commit,
    }

    pub fn init() {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
        println!("Initialized git directory");
    }

    fn read_object(object_hash: String) -> Vec<u8> {
        let content = fs::read(format!(
            ".git/objects/{}/{}",
            &object_hash[..2],
            &object_hash[2..]
        ))
        .unwrap();
        let mut decoder = ZlibDecoder::new(&content as &[u8]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        decompressed
    }

    pub fn cat_file(pretty_print: bool, show_type: bool, show_size: bool, object_hash: String) {
        // Read the content of the object
        let decompressed = read_object(object_hash);
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

    fn hash_object(file_content: String, object_type: ObjectType) -> String {
        // Get the content and add the headers
        let content = match object_type {
            ObjectType::Blob => {
                format!("blob {}\0{}", file_content.len(), file_content)
            }
            ObjectType::Tree => {
                format!("tree {}\0{}", file_content.len(), file_content)
            }
            ObjectType::Commit => {
                format!("commit {}\0{}", file_content.len(), file_content)
            }
        };

        // Create the sha1 hash
        let mut hasher = Sha1::new();
        hasher.update(&content);
        let result = hasher.finalize();
        let hex_hash = hex::encode(result);

        // Zlib encode the content
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(content.as_bytes()).unwrap();
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

    pub fn hash_blob(write: bool, file: &str) {
        if write {
            print!(
                "{}",
                hash_object(fs::read_to_string(file).unwrap(), ObjectType::Blob)
            )
        }
    }

    pub fn ls_tree(name_only: bool, tree_hash: String) {
        if name_only {
            let curr_dir = env::current_dir().unwrap();
            println!("{}/", curr_dir.file_name().unwrap().to_str().unwrap());
            parse_tree_object(tree_hash, SPACES_PER_TAB)
        }
    }

    fn parse_tree_object(tree_hash: String, spaces: usize) {
        let mut size_used: usize = 0;
        let mut content_iter = read_object(tree_hash).into_iter();

        let tree_check: String = content_iter
            .by_ref()
            .take(OBJECT_TYPE_LENGTH)
            .map(|b| b as char)
            .collect();
        if tree_check != "tree" {
            return;
        }

        content_iter.next(); // skip the space

        let size_str: String = content_iter
            .by_ref()
            .take_while(|&b| b != 0)
            .map(|b| b as char)
            .collect();
        let size: usize = size_str.parse().unwrap();

        while size_used < size {
            size_used += parse_tree_entry(&mut content_iter, spaces);
        }
    }

    fn parse_tree_entry(content_iter: &mut std::vec::IntoIter<u8>, spaces: usize) -> usize {
        let mut size_used: usize = 0;
        let object_type: String = content_iter
            .take(TREE_OBJ_TYPE_LENGTH)
            .map(|b| b as char)
            .collect();
        content_iter.next();
        size_used += TREE_OBJ_TYPE_LENGTH + 1; // the space

        let object_name: String = content_iter
            .take_while(|&b| {
                size_used += 1;
                b != 0
            })
            .map(|b| b as char)
            .collect();

        print!("{}- ", " ".repeat(spaces));
        if object_type == FILE {
            content_iter.nth(SHA_HASH_LENGTH - 1); // starts at 0
            println!("{}", object_name);
        } else if object_type == FOLDER {
            let sha_hash: Vec<_> = content_iter.take(SHA_HASH_LENGTH).collect();
            let hex_hash = hex::encode(sha_hash);
            println!("{}/", object_name);
            parse_tree_object(hex_hash, spaces + SPACES_PER_TAB);
        }
        size_used += SHA_HASH_LENGTH;

        size_used
    }

    pub fn write_tree() {
        print!(
            "{}",
            hash_object(recursively_write_to_tree(Path::new(".")), ObjectType::Tree)
        );
    }

    fn recursively_write_to_tree(path: &Path) -> String {
        let mut content = String::new();
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.into_iter().flatten() {
                let path = entry.path();
                let file_name = path.file_name().unwrap();
                let object_type: ObjectType;
                if path.is_dir() {
                    if file_name == ".git" {
                        continue;
                    } else {
                        content += FOLDER;
                        content += " ";
                        object_type = ObjectType::Tree
                    }
                } else {
                    content += FILE;
                    content += " ";
                    object_type = ObjectType::Blob
                }
                content += file_name.to_str().unwrap();
                content += "\0";

                content += unsafe {
                    &String::from_utf8_unchecked(
                        hex::decode(hash_object(
                            match object_type {
                                ObjectType::Blob => fs::read_to_string(path.as_path()).unwrap(),
                                ObjectType::Tree => recursively_write_to_tree(path.as_path()),
                                _ => String::new(), // we don't care about anything else
                            },
                            object_type,
                        ))
                        .unwrap(),
                    )
                }
            }
        }

        content
    }

    pub fn commit_tree(message: String, tree_hash: String, parent_hash: Option<String>) {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();

        // TODO: make sure the parent is an actual commit
        let parents = match parent_hash {
            Some(parent_hash) => format!("parent {}\n", parent_hash),
            None => String::new(),
        };
        let author = format!(
            "author realbyter <justavishay@gmail.com> {} +0003",
            since_the_epoch.as_secs()
        );
        let commiter = format!(
            "commiter realbyter <justavishay@gmail.com> {} +0003",
            since_the_epoch.as_secs()
        );
        let content = format!(
            "tree {}\n{}{}\n{}\n\n{}",
            tree_hash, parents, author, commiter, message
        );
        print!("{}", hash_object(content, ObjectType::Commit));
    }
}

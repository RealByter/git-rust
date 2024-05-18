use clap::{ArgGroup, Parser, Subcommand};
use git_starter_rust::git;

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
    LsTree,
    WriteTree,
}

fn main() {
    // let args: Vec<String> = env::args().collect();
    let args = Args::parse();
    match args.command {
        Command::Init => git::init(),

        Command::CatFile {
            pretty_print,
            show_type,
            show_size,
            object_hash,
        } => git::cat_file(pretty_print, show_type, show_size, object_hash),
        Command::HashObject { write, file } => git::hash_blob(write, &file, git::ObjectType::Blob),
        Command::LsTree => git::ls_tree(),
        Command::WriteTree => git::write_tree(),
    }
}

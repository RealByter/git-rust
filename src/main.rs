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
    LsTree {
        #[clap(short = 'n', required = true)] // remove the short
        name_only: bool,
        tree_hash: String,
    },
    WriteTree,
    CommitTree {
        #[clap(short = 'm')]
        message: String,
        #[clap(short = 'p')]
        parent_hash: Option<String>,
        tree_hash: String,
    },
    Log {
        commit_hash: String,
    }
}

fn main() {
    let args = Args::parse();
    match args.command {
        Command::Init => git::init(),

        Command::CatFile {
            pretty_print,
            show_type,
            show_size,
            object_hash,
        } => git::cat_file(pretty_print, show_type, show_size, object_hash),
        Command::HashObject { write, file } => git::hash_blob(write, &file),
        Command::LsTree {
            name_only,
            tree_hash,
        } => git::ls_tree(name_only, tree_hash),
        Command::WriteTree => git::write_tree(),
        Command::CommitTree {
            message,
            parent_hash,
            tree_hash,
        } => git::commit_tree(message, tree_hash, parent_hash),
        Command::Log { commit_hash } => git::log(commit_hash),
    }
}

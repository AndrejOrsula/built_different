//! CLI tool for creating patches between two paths.
use built_different::{create_file_patch, create_file_patches};
use clap::Parser;

enum PatchMode {
    File,
    Directory,
}

fn main() {
    let args = Args::parse();

    let input_path = std::path::Path::new(&args.input_path);
    let modified_path = std::path::Path::new(&args.modified_path);
    let output_path = std::path::Path::new(&args.output_path);

    // Make sure the original and modified paths exist
    match (
        input_path.try_exists().unwrap(),
        modified_path.try_exists().unwrap(),
    ) {
        (false, false) => {
            eprintln!(
                "Neither the original path ({}) nor the modified path ({}) exist",
                input_path.display(),
                modified_path.display()
            );
            std::process::exit(1);
        }
        (false, true) => {
            eprintln!(
                "The original path ({}) does not exist",
                input_path.display()
            );
            std::process::exit(1);
        }
        (true, false) => {
            eprintln!(
                "The modified path ({}) does not exist",
                modified_path.display()
            );
            std::process::exit(1);
        }
        _ => {}
    }

    // Make sure that both paths are either files or directories
    let mode = match (input_path.is_file(), modified_path.is_file()) {
        (true, true) => PatchMode::File,
        (false, false) => PatchMode::Directory,
        (true, false) => {
            eprintln!(
                "The original path ({}) is a file, but the modified path ({}) is a directory. Make sure that both paths are either files or directories.",
                input_path.display(),
                modified_path.display()
            );
            std::process::exit(1);
        }
        (false, true) => {
            eprintln!(
                "The original path ({}) is a directory, but the modified path ({}) is a file. Make sure that both paths are either files or directories.",
                input_path.display(),
                modified_path.display()
            );
            std::process::exit(1);
        }
    };

    // Create the patch(es)
    match mode {
        PatchMode::File => {
            create_file_patch(input_path, modified_path, output_path).unwrap();
        }
        PatchMode::Directory => {
            create_file_patches(input_path, modified_path, output_path).unwrap();
        }
    }
}

/// Arguments for the CLI
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// The filepath or directory containing the original files
    #[clap(long, short)]
    input_path: String,
    /// The filepath or directory containing the modified files
    #[clap(long, short)]
    modified_path: String,
    /// The filepath or directory to output the patches to
    #[clap(long, short)]
    output_path: String,
}

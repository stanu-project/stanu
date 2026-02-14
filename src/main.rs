use std::path::PathBuf;
use std::process;

use stanu::{debug_tree, parse_directory, parse_file};
use stanu::syntax_kind::SyntaxNode;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: stanu <directory-or-file>");
        process::exit(1);
    }

    let path = PathBuf::from(&args[1]);

    if path.is_file() {
        let source = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Failed to read {}: {}", path.display(), e);
            process::exit(1);
        });
        let (green, errors) = parse_file(&source);
        println!("=== {} ===", path.display());
        let node = SyntaxNode::new_root(green);
        println!("{node:#?}");
        if !errors.is_empty() {
            println!("Errors:");
            for err in &errors {
                println!("  {err}");
            }
        }
    } else if path.is_dir() {
        let results = parse_directory(&path);
        for result in &results {
            println!("=== {} ===", result.path.display());
            println!("{}", debug_tree(&result.green));
            if !result.errors.is_empty() {
                println!("Errors:");
                for err in &result.errors {
                    println!("  {err}");
                }
            }
            println!();
        }
        println!(
            "Parsed {} files, {} with errors",
            results.len(),
            results.iter().filter(|r| !r.errors.is_empty()).count()
        );
    } else {
        eprintln!("{} is not a file or directory", path.display());
        process::exit(1);
    }
}

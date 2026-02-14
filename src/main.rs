use std::path::PathBuf;
use std::process;

use rayon::prelude::*;
use walkdir::WalkDir;

use stanu::formatter::{format_file, FormatStatus};
use stanu::syntax_kind::SyntaxNode;
use stanu::{debug_tree, parse_directory, parse_file};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: stanu <command> [options] <path>");
        eprintln!("Commands:");
        eprintln!("  fmt [--check] <path>   Format HCL files");
        eprintln!("  parse <path>           Parse and dump syntax tree");
        process::exit(1);
    }

    match args[1].as_str() {
        "fmt" => cmd_fmt(&args[2..]),
        "parse" => cmd_parse(&args[2..]),
        _ => {
            // Backward compat: treat as path for parse
            cmd_parse(&args[1..])
        }
    }
}

fn cmd_fmt(args: &[String]) {
    let mut check_only = false;
    let mut paths = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--check" => check_only = true,
            "--fix" => check_only = false,
            _ => paths.push(PathBuf::from(arg)),
        }
    }

    if paths.is_empty() {
        eprintln!("Usage: stanu fmt [--check|--fix] <path>");
        process::exit(1);
    }

    let mut all_files: Vec<PathBuf> = Vec::new();
    for path in &paths {
        if path.is_file() {
            all_files.push(path.clone());
        } else if path.is_dir() {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let p = entry.into_path();
                if matches!(
                    p.extension().and_then(|s| s.to_str()),
                    Some("tf") | Some("hcl")
                ) {
                    all_files.push(p);
                }
            }
        } else {
            eprintln!("{} is not a file or directory", path.display());
            process::exit(1);
        }
    }

    let results: Vec<(PathBuf, FormatStatus)> = all_files
        .par_iter()
        .filter_map(|path| match format_file(path, check_only) {
            Ok(status) => Some((path.clone(), status)),
            Err(e) => {
                eprintln!("Error processing {}: {}", path.display(), e);
                None
            }
        })
        .collect();

    let mut has_changes = false;
    for (path, status) in &results {
        match status {
            FormatStatus::Changed => {
                println!("{}", path.display());
                has_changes = true;
            }
            FormatStatus::Skipped => {
                eprintln!("Skipped {} (parse errors)", path.display());
            }
            FormatStatus::Unchanged => {}
        }
    }

    if check_only && has_changes {
        process::exit(1);
    }
}

fn cmd_parse(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: stanu parse <path>");
        process::exit(1);
    }

    let path = PathBuf::from(&args[0]);

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

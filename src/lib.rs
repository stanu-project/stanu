pub mod error;
pub mod formatter;
pub mod lexer;
pub mod parser;
pub mod syntax_kind;

use std::path::{Path, PathBuf};

use rayon::prelude::*;
use rowan::GreenNode;
use walkdir::WalkDir;

use crate::error::ParseError;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::syntax_kind::SyntaxNode;

pub struct FileParseResult {
    pub path: PathBuf,
    pub green: GreenNode,
    pub errors: Vec<ParseError>,
}

pub fn parse_file(source: &str) -> (GreenNode, Vec<ParseError>) {
    let tokens = Lexer::new(source).tokenize();
    let parser = Parser::new(tokens, source);
    parser.parse()
}

pub fn parse_directory(dir: &Path) -> Vec<FileParseResult> {
    let files: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            matches!(path.extension().and_then(|s| s.to_str()), Some("tf") | Some("hcl"))
        })
        .map(|e| e.into_path())
        .collect();

    files
        .par_iter()
        .filter_map(|path| {
            let source = std::fs::read_to_string(path).ok()?;
            let (green, errors) = parse_file(&source);
            Some(FileParseResult {
                path: path.clone(),
                green,
                errors,
            })
        })
        .collect()
}

pub fn debug_tree(green: &GreenNode) -> String {
    let node = SyntaxNode::new_root(green.clone());
    format!("{:#?}", node)
}

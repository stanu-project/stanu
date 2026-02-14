use std::fs;
use std::path::Path;

use stanu::{parse_directory, parse_file};
use stanu::syntax_kind::SyntaxNode;

#[test]
fn parse_fixture_simple_tf() {
    let source = fs::read_to_string("tests/fixtures/simple.tf").unwrap();
    let (green, errors) = parse_file(&source);
    let node = SyntaxNode::new_root(green);
    let reconstructed = node.text().to_string();
    assert_eq!(reconstructed, source, "Lossless round-trip failed for simple.tf");
    assert!(errors.is_empty(), "Unexpected errors in simple.tf: {errors:?}");
}

#[test]
fn parse_fixture_expressions_tf() {
    let source = fs::read_to_string("tests/fixtures/expressions.tf").unwrap();
    let (green, errors) = parse_file(&source);
    let node = SyntaxNode::new_root(green);
    let reconstructed = node.text().to_string();
    assert_eq!(reconstructed, source, "Lossless round-trip failed for expressions.tf");
    assert!(errors.is_empty(), "Unexpected errors in expressions.tf: {errors:?}");
}

#[test]
fn parse_fixture_heredoc_tf() {
    let source = fs::read_to_string("tests/fixtures/heredoc.tf").unwrap();
    let (green, errors) = parse_file(&source);
    let node = SyntaxNode::new_root(green);
    let reconstructed = node.text().to_string();
    assert_eq!(reconstructed, source, "Lossless round-trip failed for heredoc.tf");
    assert!(errors.is_empty(), "Unexpected errors in heredoc.tf: {errors:?}");
}

#[test]
fn parse_fixture_errors_tf_recovers() {
    let source = fs::read_to_string("tests/fixtures/errors.tf").unwrap();
    let (green, errors) = parse_file(&source);
    let node = SyntaxNode::new_root(green);
    // Should have errors for the malformed line
    assert!(!errors.is_empty(), "Expected errors in errors.tf");
    // But should still produce a tree
    let debug = format!("{node:#?}");
    assert!(debug.contains("SOURCE_FILE"));
    // And should preserve all text (lossless)
    let reconstructed = node.text().to_string();
    assert_eq!(reconstructed, source, "Lossless round-trip failed for errors.tf");
}

#[test]
fn parallel_parse_fixtures_directory() {
    let path = Path::new("tests/fixtures");
    let results = parse_directory(path);
    assert!(results.len() >= 3, "Expected at least 3 fixture files, got {}", results.len());
    for result in &results {
        let node = SyntaxNode::new_root(result.green.clone());
        let source = fs::read_to_string(&result.path).unwrap();
        let reconstructed = node.text().to_string();
        assert_eq!(
            reconstructed, source,
            "Lossless round-trip failed for {}",
            result.path.display()
        );
    }
}

use std::path::Path;
use stanu::parse_file;
use stanu::syntax_kind::SyntaxNode;
use walkdir::WalkDir;

/// Parse all .tf files in a directory, checking lossless round-trip and collecting errors.
fn test_corpus(dir: &Path) -> (usize, Vec<String>) {
    let mut total = 0;
    let mut failures = Vec::new();

    if !dir.exists() {
        return (0, vec![format!("Directory {} does not exist - skipping", dir.display())]);
    }

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "tf" || s == "hcl")
                .unwrap_or(false)
        })
        // Skip .terraform directories (downloaded modules, lock files, etc.)
        .filter(|e| !e.path().to_string_lossy().contains(".terraform"))
    {
        let path = entry.path();
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("{}: read error: {}", path.display(), e));
                continue;
            }
        };

        total += 1;

        let (green, errors) = parse_file(&source);
        let node = SyntaxNode::new_root(green);
        let reconstructed = node.text().to_string();

        // Check 1: Lossless round-trip
        if reconstructed != source {
            let first_diff = source
                .chars()
                .zip(reconstructed.chars())
                .position(|(a, b)| a != b)
                .unwrap_or(source.len().min(reconstructed.len()));
            failures.push(format!(
                "{}: ROUND-TRIP FAILURE at byte {} (source len={}, reconstructed len={})\n  source[..60]:      {:?}\n  reconstructed[..60]: {:?}",
                path.display(),
                first_diff,
                source.len(),
                reconstructed.len(),
                &source[first_diff..source.len().min(first_diff + 60)],
                &reconstructed[first_diff..reconstructed.len().min(first_diff + 60)],
            ));
            continue; // skip error check if round-trip fails
        }

        // Check 2: No parse errors
        if !errors.is_empty() {
            failures.push(format!(
                "{}: {} parse error(s):\n{}",
                path.display(),
                errors.len(),
                errors
                    .iter()
                    .take(5)
                    .map(|e| format!("  {}", e))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
    }

    (total, failures)
}

#[test]
fn corpus_terraform_aws_vpc() {
    let dir = Path::new("/tmp/tf-corpus/terraform-aws-vpc");
    let (total, failures) = test_corpus(dir);
    if total == 0 {
        eprintln!("SKIPPED: terraform-aws-vpc not found at {}", dir.display());
        return;
    }
    eprintln!("Parsed {} .tf files from terraform-aws-vpc", total);
    if !failures.is_empty() {
        panic!(
            "{}/{} files failed:\n\n{}",
            failures.len(),
            total,
            failures.join("\n\n")
        );
    }
}

#[test]
fn corpus_terraform_aws_eks() {
    let dir = Path::new("/tmp/tf-corpus/terraform-aws-eks");
    let (total, failures) = test_corpus(dir);
    if total == 0 {
        eprintln!("SKIPPED: terraform-aws-eks not found at {}", dir.display());
        return;
    }
    eprintln!("Parsed {} .tf files from terraform-aws-eks", total);
    if !failures.is_empty() {
        panic!(
            "{}/{} files failed:\n\n{}",
            failures.len(),
            total,
            failures.join("\n\n")
        );
    }
}

#[test]
fn corpus_terraform_aws_iam() {
    let dir = Path::new("/tmp/tf-corpus/terraform-aws-iam");
    let (total, failures) = test_corpus(dir);
    if total == 0 {
        eprintln!("SKIPPED: terraform-aws-iam not found at {}", dir.display());
        return;
    }
    eprintln!("Parsed {} .tf files from terraform-aws-iam", total);
    if !failures.is_empty() {
        panic!(
            "{}/{} files failed:\n\n{}",
            failures.len(),
            total,
            failures.join("\n\n")
        );
    }
}

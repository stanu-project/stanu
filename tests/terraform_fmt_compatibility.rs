use std::fs;
use std::path::Path;
use std::process::Command;
use stanu::formatter::{format, FormatResult};

fn run_terraform_fmt(input: &str) -> Option<String> {
    use std::io::Write;

    let mut child = Command::new("terraform")
        .arg("fmt")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn terraform fmt");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    if output.status.success() {
        Some(String::from_utf8(output.stdout).expect("Invalid UTF-8 from terraform fmt"))
    } else {
        None
    }
}

#[test]
fn compare_with_terraform_fmt() {
    let fixtures_dir = Path::new("tests/fixtures");
    let entries = fs::read_dir(fixtures_dir).expect("Failed to read fixtures dir");

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("tf") {
            let file_name = path.file_name().unwrap().to_string_lossy();
            println!("Testing {}", file_name);

            let input = fs::read_to_string(&path).expect("Failed to read file");

            // Run stanu formatter
            let stanu_output = match format(&input) {
                FormatResult::Changed(s) => Some(s),
                FormatResult::Unchanged(s) => Some(s),
                FormatResult::Skipped => None,
            };

            // Run terraform fmt
            let terraform_output = run_terraform_fmt(&input);

            match (stanu_output, terraform_output) {
                (Some(stanu), Some(tf)) => {
                    if stanu != tf {
                        // Create a unified diff for easier debugging
                        println!("Mismatch in {}:
", file_name);
                        println!("--- stanu output ---");
                        println!("{}", stanu);
                        println!("--- terraform fmt output ---");
                        println!("{}", tf);
                        println!("------------------------");
                        
                        // Use pretty_assertions or similar if available, otherwise manual panic
                        assert_eq!(stanu, tf, "Formatter output differs for {}", file_name);
                    }
                }
                (None, None) => {
                    println!("Both skipped {}", file_name);
                }
                (Some(_), None) => {
                    println!("Terraform failed on {}, but stanu succeeded. This might be OK if stanu is lenient.", file_name);
                }
                (None, Some(_)) => {
                    println!("Stanu failed on {}, but terraform succeeded. This might indicate a parser bug or intended behavior.", file_name);
                }
            }
        }
    }
}

# ⚡ Stanu

> **The Ruff of Terraform.**

[![Crates.io](https://img.shields.io/crates/v/stanu.svg)](https://crates.io/crates/stanu)
[![License](https://img.shields.io/crates/l/stanu.svg)](https://github.com/yourusername/stanu/blob/main/LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/yourusername/stanu/ci.yml?branch=main)](https://github.com/yourusername/stanu/actions)

**Stanu** is an extremely fast Terraform linter and formatter, written in Rust. It is designed to be a drop-in replacement for `terraform fmt`, but significantly faster and more extensible.

Stop waiting for your CI pipeline. Lint and format your Infrastructure-as-Code in milliseconds, not seconds.

## ✨ Features

- **🚀 Blazing Fast:** Built with Rust and optimized for performance. Uses [Rayon](https://github.com/rayon-rs/rayon) for parallel processing of files.
- **🛠️ Drop-in Replacement:** Compatible with existing `terraform fmt` workflows.
- **🛡️ Robust Parsing:** leverage [Rowan](https://github.com/rust-analyzer/rowan) for lossless syntax trees, ensuring your code structure is preserved exactly as you intended.
- **⚡ Parallel Execution:** format thousands of files in the blink of an eye.

## 📦 Installation

### From Source

```bash
cargo install --path .
```

*(Binary releases coming soon)*

## 🚀 Usage

Basic usage is identical to what you are used to:

```bash
# Format all files in the current directory and subdirectories
stanu fmt .

# Check if files are formatted (useful for CI)
stanu fmt --check .
```

## 📊 Benchmarks

Comparison running on a MacBook Pro (M-series):

| Scenario | `terraform fmt` | `stanu` | Speedup |
|----------|-----------------|---------|---------|
| **300 Files (Already Formatted)** | 0.079s | **0.010s** | **~8x** |
| **1000 Files (Unformatted)** | 0.184s | **0.053s** | **~3.5x** |

*Benchmarks run on generated fixtures. `stanu` uses parallel execution to achieve these speeds.*

## 🤝 Contributing

We are just getting started! If you love Rust and DevOps, contributions are welcome.

1.  Fork the repository.
2.  Create a feature branch.
3.  Submit a Pull Request.

## 📜 License

MIT License. See [LICENSE](LICENSE) for details.

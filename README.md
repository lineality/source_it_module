#### source_it
A Rust module for embedding source code directly into compiled binaries:
enabling self-contained open-source distribution.
Any application with source_it added can reproduce its own source code files.

- cli,     Command line application
- module,  Add it to rust terminal applications
- vanilla, Vanilla Rust (no 3rd party dependencies)
- MIT,     Do what you will with it.

### Problem to solve:
Public repositories may not be available, may not be known,
or may disappear in time. GitHub & gitlab might not exist in 20 years.
Links break. Dependencies vanish. Websites are...not always current.
And long-term data storage is (as of 2025) not cared about.

### Solution:
Embed your source code directly in your binary.
When users run 'your_app --source', they get the complete source code.


# Installation & Use
### 1. Copy the source_it_module.rs file into your project:

### 2. Set Up in main.rs
- List file each to include. You MUST list each file. There is no auto-detection or auto-inclusion.

#### Example:
```rust
// In main.rs:
mod source_it_module;
use source_it_module::{SourcedFile, handle_sourceit_command};

// Developer explicitly lists files to embed
const SOURCE_FILES: &[SourcedFile] = &[
    SourcedFile::new("Cargo.toml", include_str!("../Cargo.toml")),
    SourcedFile::new("src/main.rs", include_str!("main.rs")),
    SourcedFile::new(
        "src/source_it_module.rs",
        include_str!("source_it_module.rs"),
    ),
    // SourcedFile::new("src/lib.rs", include_str!("lib.rs")),
    // SourcedFile::new("src/algorithms/fft.rs", include_str!("algorithms/fft.rs")),
    // SourcedFile::new("README.md", include_str!("../README.md")),
    // SourcedFile::new("LICENSE", include_str!("../LICENSE")),
    SourcedFile::new(".gitignore", include_str!("../.gitignore")),
];

fn main() {
    // get arguments
    let args: Vec<String> = std::env::args().collect();

    // if --source, then source it...
    if args.contains(&"--source".to_string()) {
        match handle_sourceit_command("my_fft_tool", None, SOURCE_FILES) {
            // print the file path where the source now is...
            Ok(path) => println!("Source extracted to: {}", path.display()),
            Err(e) => eprintln!("Failed to extract source: {}", e),
        }
        return;
    }

    // ...codestuff...
}

```
### 3. Use source_it
when installed (or alias)
```bash
my_app --source
```
or, in rust
```bash
cargo run -- --source
```
or, uninstalled binary
```bash
./my_app --source
```

#### sample output
```
✓ Source code extracted to: /home/user/source_crate_my_app_20241215_143022
  6 files extracted
```

## Notes:
- Text files only - No binary assets
- Compile-time size - All files embedded at compilation
- Manual listing - No automatic file discovery
- No compression - Files stored as-is (rely on binary compression if needed)
- sha256 checksums created for varification on some POSIX operating systems


Many public code repositories may not last for decades or centuries, but good STEM code can outlive unmaintainable systems.

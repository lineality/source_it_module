#### source_it
Rust module for embedding source code directly into compiled binaries:
enabling self-contained open-source distribution.
Any application with source_it added can reproduce its own source code files.

### Problem to solve:
Public repositories may not be available, may not be known,
or may disappear in time. GitHub & gitlab might not exist in 20 years.
Links break. Dependencies vanish. Websites are...not always current.
And long-term data storage is (as of 2025) not cared about.

### Solution:
Embed your source code directly in your binary.
When users run 'your_app --source', they get the complete source code.


# Installation & Use
### 1. Copy the source_it.rs file into your project:

### 2. Set Up in main.rs

#### Example:
- List each to include. You MUST list each file,
there is no auto-detection or auto-inclusion.

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
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--source".to_string()) {
        match handle_sourceit_command("my_fft_tool", None, SOURCE_FILES) {
            Ok(path) => println!("Source extracted to: {}", path.display()),
            Err(e) => eprintln!("Failed to extract source: {}", e),
        }
        return;
    }

    // ...codestuff...
}

```
### 3. Use source_it
```bash
my_app --source
```
or
```bash
cargo run -- --source
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


In 20 years, GitHub might be gone, but good STEM code can outlive unmaintainable systems.

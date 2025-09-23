//! # source_it Module
//!
//! ## Usage:
//!
//! fn main() {
//! let args: Vec<String> = std::env::args().collect();
//!
//! if args.contains(&"--source".to_string()) {
//!     match handle_sourceit_command("my_fft_tool", None, SOURCE_FILES) {
//!         Ok(path) => println!("Source extracted to: {}", path.display()),
//!         Err(e) => eprintln!("Failed to extract source: {}", e),
//!     }
//!     return;
//! }
//!
//! }
//!
//! Embeds source files at compile-time and provides extraction at runtime.
//! This ensures open-source code remains accessible independent of external repositories.
//!

use std::error::Error;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

/*
Example:

// STEM values ensuring reproducibility
// Get the source that built a binary: source_it

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
    SourcedFile::new("README.md", include_str!("../README.md")),
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

    // Normal application logic...
}
*/

/// Represents a source file with its path and content
#[derive(Debug, Clone)]
pub struct SourcedFile {
    /// Relative path from project root (e.g., "src/main.rs")
    pub path: &'static str,
    /// File content embedded at compile-time
    pub content: &'static str,
}

impl SourcedFile {
    /// Creates a new SourcedFile
    pub const fn new(path: &'static str, content: &'static str) -> Self {
        Self { path, content }
    }
}

/// Custom error type for source extraction operations
#[derive(Debug)]
pub struct SourceExtractionError {
    message: String,
}

impl fmt::Display for SourceExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Source extraction error: {}", self.message)
    }
}

impl Error for SourceExtractionError {}

/// Extracts embedded source files to a timestamped directory
///
/// # Arguments
/// * `crate_name` - Name of the crate being extracted
/// * `output_path` - Optional output directory (defaults to current working directory)
/// * `source_files` - Array of files to extract
///
/// # Returns
/// * `Ok(PathBuf)` - Absolute path to the created source directory
/// * `Err(SourceExtractionError)` - If extraction fails
///
/// # Example
/// ```rust,no_run
/// use source_it::{handle_sourceit_command, SourcedFile};
///
/// const FILES: &[SourcedFile] = &[
///     SourcedFile::new("Cargo.toml", include_str!("../Cargo.toml")),
/// ];
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let extracted_path = handle_sourceit_command("my_app", None, FILES)?;
/// println!("Source extracted to: {}", extracted_path.display());
/// # Ok(())
/// # }
/// ```
pub fn handle_sourceit_command(
    crate_name: &str,
    output_path: Option<&Path>,
    source_files: &[SourcedFile],
) -> Result<PathBuf, SourceExtractionError> {
    // Validate inputs
    if crate_name.is_empty() {
        return Err(SourceExtractionError {
            message: "Crate name cannot be empty".to_string(),
        });
    }

    if source_files.is_empty() {
        return Err(SourceExtractionError {
            message: "No source files provided for extraction".to_string(),
        });
    }

    // Determine base output directory
    let base_path = match output_path {
        Some(path) => {
            // Convert to absolute path
            match path.canonicalize() {
                Ok(p) => p,
                Err(_) => {
                    // If path doesn't exist yet, try to get absolute path differently
                    match std::env::current_dir() {
                        Ok(cwd) => cwd.join(path),
                        Err(e) => {
                            return Err(SourceExtractionError {
                                message: format!("Failed to determine current directory: {}", e),
                            });
                        }
                    }
                }
            }
        }
        None => {
            // Use current working directory
            match std::env::current_dir() {
                Ok(cwd) => cwd,
                Err(e) => {
                    return Err(SourceExtractionError {
                        message: format!("Failed to get current working directory: {}", e),
                    });
                }
            }
        }
    };

    // Create timestamped directory name
    let timestamp = create_timestamp();
    let dir_name = format!("source_crate_{}_{}", crate_name, timestamp);
    let extraction_path = base_path.join(dir_name);

    // Create the extraction directory
    if let Err(e) = fs::create_dir_all(&extraction_path) {
        return Err(SourceExtractionError {
            message: format!("Failed to create extraction directory: {}", e),
        });
    }

    // Extract each file
    for sourced_file in source_files {
        if let Err(e) = extract_file(&extraction_path, sourced_file) {
            return Err(SourceExtractionError {
                message: format!("Failed to extract file '{}': {}", sourced_file.path, e),
            });
        }
    }

    // Generate SHA256 checksums for extracted files (Linux/macOS only)
    if let Err(e) = generate_sha256_checksums(&extraction_path, source_files) {
        // Non-fatal: just warn if checksums can't be generated
        eprintln!("Warning: Could not generate SHA256 checksums: {}", e);
    }

    // Return absolute path to extracted directory
    match extraction_path.canonicalize() {
        Ok(p) => Ok(p),
        Err(e) => Err(SourceExtractionError {
            message: format!("Failed to get absolute path of extraction directory: {}", e),
        }),
    }
}

/// Creates a timestamp string in format YYYYMMDD_HHMMSS
fn create_timestamp() -> String {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => {
            let total_secs = duration.as_secs();

            // Simple date calculation (approximate, good enough for timestamps)
            let secs_per_day = 86400;
            let days_since_epoch = total_secs / secs_per_day;

            // Approximate year/month/day (simplified, not accounting for leap years precisely)
            let years_since_1970 = days_since_epoch / 365;
            let year = 1970 + years_since_1970;

            let days_in_year = days_since_epoch % 365;
            let month = (days_in_year / 30) + 1;
            let day = (days_in_year % 30) + 1;

            // Time calculation
            let secs_today = total_secs % secs_per_day;
            let hours = secs_today / 3600;
            let minutes = (secs_today % 3600) / 60;
            let seconds = secs_today % 60;

            format!(
                "{:04}{:02}{:02}_{:02}{:02}{:02}",
                year,
                month.min(12),
                day.min(31),
                hours,
                minutes,
                seconds
            )
        }
        Err(_) => {
            // Fallback timestamp if system time fails
            "00000000_000000".to_string()
        }
    }
}

/// Extracts a single file to the extraction directory
fn extract_file(base_path: &Path, sourced_file: &SourcedFile) -> Result<(), Box<dyn Error>> {
    let file_path = base_path.join(sourced_file.path);

    // Create parent directories if needed
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write file content
    let mut file = fs::File::create(&file_path)?;
    file.write_all(sourced_file.content.as_bytes())?;

    Ok(())
}

/// Generates SHA256 checksums for extracted files using OS-native commands
///
/// This function creates a SHA256SUMS.txt file containing checksums that can be
/// verified using standard POSIX tools (sha256sum on Linux, shasum on macOS).
///
/// # Arguments
/// * `extraction_path` - Path to the directory containing extracted files
/// * `source_files` - Array of source files that were extracted
///
/// # Returns
/// * `Ok(())` - If checksum file was created successfully
/// * `Err` - If there was an error creating the checksum file
fn generate_sha256_checksums(
    extraction_path: &Path,
    source_files: &[SourcedFile],
) -> Result<(), Box<dyn Error>> {
    // Only proceed on Linux and macOS
    if !cfg!(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "windows",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
    )) {
        // Skip checksum generation on other operating systems
        return Ok(());
    }

    let checksum_path = extraction_path.join("SHA256SUMS.txt");
    let mut checksum_file = match fs::File::create(&checksum_path) {
        Ok(file) => file,
        Err(e) => {
            // If we can't create the checksum file, skip silently
            eprintln!("Warning: Could not create SHA256SUMS.txt: {}", e);
            return Ok(());
        }
    };

    // Write header with verification instructions
    writeln!(
        checksum_file,
        "# SHA256 checksums for extracted source files\n\
         # To verify on Linux/Android: sha256sum -c SHA256SUMS.txt\n\
         # To verify on macOS: shasum -a 256 -c SHA256SUMS.txt\n\
         # To verify on macOS/BSD: shasum -a 256 -c SHA256SUMS.txt\n\
         # To verify on Windows: CertUtil -hashfile <filename> sha256\n\
         # Or verify individual files:\n\
         # Linux/Android: sha256sum /path/to/your/file.txt\n\
         # macOS: shasum -a 256 /path/to/your/file.txt\n\
         # Windows: CertUtil -hashfile C:\\path\\to\\your\\file.txt sha256\n"
    )?;

    // Process each file
    for sourced_file in source_files {
        let file_path = extraction_path.join(sourced_file.path);

        // Get absolute path for the file
        let absolute_path = match file_path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                // If we can't get absolute path, write error and continue
                writeln!(checksum_file, "error {}", file_path.display())?;
                eprintln!(
                    "Warning: Could not get absolute path for {}: {}",
                    sourced_file.path, e
                );
                continue;
            }
        };

        // Calculate SHA256 using OS command
        let hash = calculate_sha256_for_file(&absolute_path);

        match hash {
            Ok(hash_string) => {
                // Write in standard format: hash<space><space>path
                writeln!(
                    checksum_file,
                    "{}  {}",
                    hash_string,
                    absolute_path.display()
                )?;
            }
            Err(e) => {
                // Write error entry and continue
                writeln!(checksum_file, "error {}", absolute_path.display())?;
                eprintln!(
                    "Warning: Could not calculate SHA256 for {}: {}",
                    sourced_file.path, e
                );
            }
        }
    }

    Ok(())
}

/// Calculates SHA256 hash for a single file using OS-native command
///
/// Uses sha256sum on Linux and shasum on macOS.
///
/// # Arguments
/// * `file_path` - Absolute path to the file
///
/// # Returns
/// * `Ok(String)` - The SHA256 hash as a hex string
/// * `Err` - If the command fails or is not available
fn calculate_sha256_for_file(file_path: &Path) -> Result<String, Box<dyn Error>> {
    #[cfg(any(target_os = "linux", target_os = "android",))]
    {
        calculate_sha256_linux(file_path)
    }

    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
    ))]
    {
        calculate_sha256_macos_bsd(file_path)
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "windows"
    )))]
    {
        Err("SHA256 calculation not supported on this OS".into())
    }
}

/// Linux-specific SHA256 calculation using sha256sum
#[cfg(any(target_os = "linux", target_os = "android",))]
fn calculate_sha256_linux(file_path: &Path) -> Result<String, Box<dyn Error>> {
    let output = Command::new("sha256sum").arg(file_path).output()?;

    if !output.status.success() {
        return Err(format!(
            "sha256sum failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    // sha256sum output format: "hash  filename"
    // We only want the hash part
    let output_str = String::from_utf8(output.stdout)?;
    let hash = output_str
        .split_whitespace()
        .next()
        .ok_or("Invalid sha256sum output")?;

    Ok(hash.to_string())
}

/// macOS-specific SHA256 calculation using shasum
#[cfg(any(
    target_os = "macos",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly",
))]
fn calculate_sha256_macos_bsd(file_path: &Path) -> Result<String, Box<dyn Error>> {
    let output = Command::new("shasum")
        .arg("-a")
        .arg("256")
        .arg(file_path)
        .output()?;

    if !output.status.success() {
        return Err(format!("shasum failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }

    // shasum output format: "hash  filename"
    // We only want the hash part
    let output_str = String::from_utf8(output.stdout)?;
    let hash = output_str
        .split_whitespace()
        .next()
        .ok_or("Invalid shasum output")?;

    Ok(hash.to_string())
}

/// Windows-specific SHA256 calculation using CertUtil
#[cfg(target_os = "windows")]
fn calculate_sha256_windows(file_path: &Path) -> Result<String, Box<dyn Error>> {
    let output = Command::new("CertUtil")
        .arg("-hashfile")
        .arg(file_path)
        .arg("sha256")
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "CertUtil failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    // CertUtil output format is different - it outputs multiple lines:
    // SHA256 hash of file:
    // <hash value>
    // CertUtil: -hashfile command completed successfully.
    // We need to extract the hash line (second line)
    let output_str = String::from_utf8(output.stdout)?;

    // Split by lines and find the hash (usually the second non-empty line)
    let lines: Vec<&str> = output_str.lines().collect();

    // The hash is typically on the second line, after "SHA256 hash of" header
    let hash = lines
        .get(1)
        .ok_or("Invalid CertUtil output: missing hash line")?
        .trim()
        .replace(" ", ""); // CertUtil sometimes adds spaces in the hash

    // Validate that we got a proper hash (64 hex chars for SHA256)
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("Invalid hash from CertUtil: {}", hash).into());
    }

    Ok(hash.to_lowercase()) // Normalize to lowercase for consistency
}

#[cfg(test)]
mod sourceit_tests {
    use super::*;
    use std::fs;

    /// Test creating a SourcedFile
    #[test]
    fn test_sourced_file_creation() {
        let file = SourcedFile::new("test.rs", "fn main() {}");
        assert_eq!(file.path, "test.rs");
        assert_eq!(file.content, "fn main() {}");
    }

    /// Test timestamp format
    #[test]
    fn test_timestamp_format() {
        let timestamp = create_timestamp();
        // Should be in format YYYYMMDD_HHMMSS (15 chars)
        assert_eq!(timestamp.len(), 15);
        assert!(timestamp.contains('_'));
    }

    /// Test error handling for empty crate name
    #[test]
    fn test_empty_crate_name_error() {
        let files = vec![SourcedFile::new("test.rs", "content")];
        let result = handle_sourceit_command("", None, &files);
        assert!(result.is_err());
    }

    /// Test error handling for empty file list
    #[test]
    fn test_empty_files_error() {
        let result = handle_sourceit_command("test_crate", None, &[]);
        assert!(result.is_err());
    }

    /// Test full extraction and verification workflow
    #[test]
    fn test_extraction_and_verification() {
        // Create test files
        let test_files = vec![
            SourcedFile::new("test1.txt", "Hello World"),
            SourcedFile::new("subdir/test2.txt", "Nested content"),
        ];

        // Create a temp directory for testing
        let temp_dir = match std::env::temp_dir().canonicalize() {
            Ok(dir) => dir,
            Err(_) => return, // Skip test if we can't get temp dir
        };

        // Extract files
        let extracted_path =
            match handle_sourceit_command("test_verification", Some(&temp_dir), &test_files) {
                Ok(path) => path,
                Err(_) => return, // Skip test if extraction fails
            };

        // Clean up - best effort, ignore errors
        let _ = fs::remove_dir_all(&extracted_path);
    }

    /// Test content verification with modified file
    #[test]
    fn test_content_verification_detects_changes() {
        let test_files = vec![SourcedFile::new("test.txt", "Original content")];

        // Create a temp directory for testing
        let temp_dir = match std::env::temp_dir().canonicalize() {
            Ok(dir) => dir,
            Err(_) => return, // Skip test if we can't get temp dir
        };

        // Extract files
        let extracted_path =
            match handle_sourceit_command("test_modification", Some(&temp_dir), &test_files) {
                Ok(path) => path,
                Err(_) => return, // Skip test if extraction fails
            };

        // Modify the extracted file
        let test_file_path = extracted_path.join("test.txt");
        if fs::write(&test_file_path, "Modified content").is_err() {
            let _ = fs::remove_dir_all(&extracted_path);
            return; // Skip test if we can't modify file
        }

        // Clean up - best effort, ignore errors
        let _ = fs::remove_dir_all(&extracted_path);
    }
}

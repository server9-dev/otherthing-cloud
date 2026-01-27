//! File I/O operations for BPMN 2.0 processes
//!
//! Provides high-level file operations for reading and writing .bpmn files,
//! with support for both absolute and relative paths, and automatic backup creation.

use super::elements::BpmnDiagram;
use super::xml_io::{BpmnXmlSerializer, XmlError};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Error type for file operations
#[derive(Debug, Clone)]
pub enum FileError {
    /// File not found
    NotFound(PathBuf),
    /// Directory not found
    DirectoryNotFound(PathBuf),
    /// Invalid file extension
    InvalidExtension {
        path: PathBuf,
        expected: &'static str,
        found: String,
    },
    /// IO error
    Io(String),
    /// XML parsing/serialization error
    Xml(String),
    /// File already exists (when not overwriting)
    AlreadyExists(PathBuf),
    /// Permission denied
    PermissionDenied(PathBuf),
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::NotFound(path) => write!(f, "File not found: {}", path.display()),
            FileError::DirectoryNotFound(path) => write!(f, "Directory not found: {}", path.display()),
            FileError::InvalidExtension { path, expected, found } => {
                write!(
                    f,
                    "Invalid file extension for {}: expected {}, found {}",
                    path.display(),
                    expected,
                    found
                )
            }
            FileError::Io(msg) => write!(f, "IO error: {}", msg),
            FileError::Xml(msg) => write!(f, "XML error: {}", msg),
            FileError::AlreadyExists(path) => write!(f, "File already exists: {}", path.display()),
            FileError::PermissionDenied(path) => write!(f, "Permission denied: {}", path.display()),
        }
    }
}

impl std::error::Error for FileError {}

impl From<io::Error> for FileError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => FileError::Io(err.to_string()),
            io::ErrorKind::PermissionDenied => {
                FileError::PermissionDenied(PathBuf::from("unknown"))
            }
            _ => FileError::Io(err.to_string()),
        }
    }
}

impl From<XmlError> for FileError {
    fn from(err: XmlError) -> Self {
        FileError::Xml(err.to_string())
    }
}

pub type FileResult<T> = Result<T, FileError>;

/// Configuration for file operations
#[derive(Debug, Clone)]
pub struct FileOptions {
    /// Create backup before overwriting
    pub create_backup: bool,
    /// Backup extension (default: .bak)
    pub backup_extension: String,
    /// Fail if file exists (when saving)
    pub fail_if_exists: bool,
    /// Pretty-print XML (with indentation)
    pub pretty_print: bool,
}

impl Default for FileOptions {
    fn default() -> Self {
        Self {
            create_backup: true,
            backup_extension: ".bak".to_string(),
            fail_if_exists: false,
            pretty_print: true,
        }
    }
}

/// BPMN file operations
pub struct BpmnFileIo;

impl BpmnFileIo {
    /// Load a BPMN diagram from a .bpmn file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .bpmn file (relative or absolute)
    ///
    /// # Returns
    ///
    /// The loaded BpmnDiagram
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use abcdodaf::bpmn::file_io::BpmnFileIo;
    ///
    /// let diagram = BpmnFileIo::load("my_process.bpmn")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> FileResult<BpmnDiagram> {
        Self::load_with_options(path, FileOptions::default())
    }

    /// Load with custom options
    pub fn load_with_options<P: AsRef<Path>>(path: P, _options: FileOptions) -> FileResult<BpmnDiagram> {
        let path = path.as_ref();

        // Validate extension
        Self::validate_extension(path)?;

        // Check file exists
        if !path.exists() {
            return Err(FileError::NotFound(path.to_path_buf()));
        }

        // Read file
        let xml = fs::read_to_string(path)
            .map_err(|e| match e.kind() {
                io::ErrorKind::PermissionDenied => FileError::PermissionDenied(path.to_path_buf()),
                _ => FileError::Io(e.to_string()),
            })?;

        // Parse XML
        BpmnXmlSerializer::from_string(&xml).map_err(|e| FileError::Xml(e.to_string()))
    }

    /// Save a BPMN diagram to a .bpmn file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to save to (relative or absolute)
    /// * `diagram` - The diagram to save
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use abcdodaf::bpmn::file_io::BpmnFileIo;
    /// use abcdodaf::bpmn::elements::BpmnDiagram;
    ///
    /// let diagram = BpmnDiagram {
    ///     id: "my_diagram".to_string(),
    ///     name: Some("My Process".to_string()),
    ///     documentation: None,
    ///     processes: vec![],
    ///     collaborations: vec![],
    ///     data_stores: vec![],
    ///     messages: vec![],
    ///     signals: vec![],
    /// };
    ///
    /// BpmnFileIo::save("my_process.bpmn", &diagram)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn save<P: AsRef<Path>>(path: P, diagram: &BpmnDiagram) -> FileResult<()> {
        Self::save_with_options(path, diagram, FileOptions::default())
    }

    /// Save with custom options
    pub fn save_with_options<P: AsRef<Path>>(
        path: P,
        diagram: &BpmnDiagram,
        options: FileOptions,
    ) -> FileResult<()> {
        let path = path.as_ref();

        // Validate extension
        Self::validate_extension(path)?;

        // Check parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                return Err(FileError::DirectoryNotFound(parent.to_path_buf()));
            }
        }

        // Check if file exists
        if path.exists() && options.fail_if_exists {
            return Err(FileError::AlreadyExists(path.to_path_buf()));
        }

        // Create backup if needed
        if path.exists() && options.create_backup {
            Self::create_backup(path, &options.backup_extension)?;
        }

        // Serialize to XML
        let xml = BpmnXmlSerializer::to_string(diagram)
            .map_err(|e| FileError::Xml(e.to_string()))?;

        // Write to file
        fs::write(path, xml).map_err(|e| match e.kind() {
            io::ErrorKind::PermissionDenied => FileError::PermissionDenied(path.to_path_buf()),
            _ => FileError::Io(e.to_string()),
        })?;

        Ok(())
    }

    /// Check if a file exists at the given path
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }

    /// Get the absolute path to a file
    pub fn absolute_path<P: AsRef<Path>>(path: P) -> FileResult<PathBuf> {
        let path = path.as_ref();
        std::fs::canonicalize(path).map_err(|_| {
            FileError::NotFound(path.to_path_buf())
        })
    }

    /// Get file metadata (size, modification time, etc.)
    pub fn metadata<P: AsRef<Path>>(path: P) -> FileResult<fs::Metadata> {
        let path = path.as_ref();
        fs::metadata(path).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => FileError::NotFound(path.to_path_buf()),
            io::ErrorKind::PermissionDenied => FileError::PermissionDenied(path.to_path_buf()),
            _ => FileError::Io(e.to_string()),
        })
    }

    /// List all .bpmn files in a directory
    pub fn list_in_directory<P: AsRef<Path>>(dir: P) -> FileResult<Vec<PathBuf>> {
        let dir = dir.as_ref();

        if !dir.is_dir() {
            return Err(FileError::DirectoryNotFound(dir.to_path_buf()));
        }

        let mut bpmn_files = Vec::new();

        for entry in fs::read_dir(dir).map_err(|e| match e.kind() {
            io::ErrorKind::PermissionDenied => FileError::PermissionDenied(dir.to_path_buf()),
            _ => FileError::Io(e.to_string()),
        })? {
            let entry = entry.map_err(|e| FileError::Io(e.to_string()))?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "bpmn" {
                        bpmn_files.push(path);
                    }
                }
            }
        }

        bpmn_files.sort();
        Ok(bpmn_files)
    }

    /// Delete a BPMN file (with optional backup deletion)
    pub fn delete<P: AsRef<Path>>(path: P) -> FileResult<()> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(FileError::NotFound(path.to_path_buf()));
        }

        fs::remove_file(path).map_err(|e| match e.kind() {
            io::ErrorKind::PermissionDenied => FileError::PermissionDenied(path.to_path_buf()),
            _ => FileError::Io(e.to_string()),
        })?;

        Ok(())
    }

    // ========================================================================
    // Private helper methods
    // ========================================================================

    fn validate_extension<P: AsRef<Path>>(path: P) -> FileResult<()> {
        let path = path.as_ref();
        match path.extension() {
            Some(ext) => {
                let ext_str = ext.to_string_lossy();
                if ext_str != "bpmn" {
                    Err(FileError::InvalidExtension {
                        path: path.to_path_buf(),
                        expected: ".bpmn",
                        found: format!(".{}", ext_str),
                    })
                } else {
                    Ok(())
                }
            }
            None => Err(FileError::InvalidExtension {
                path: path.to_path_buf(),
                expected: ".bpmn",
                found: "(no extension)".to_string(),
            }),
        }
    }

    fn create_backup(path: &Path, backup_ext: &str) -> FileResult<()> {
        let backup_path = path.with_extension(format!(
            "bpmn{}",
            backup_ext
        ));

        fs::copy(path, &backup_path).map_err(|e| {
            FileError::Io(format!("Failed to create backup: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_extension_valid() {
        assert!(BpmnFileIo::validate_extension(Path::new("test.bpmn")).is_ok());
    }

    #[test]
    fn test_validate_extension_invalid() {
        let result = BpmnFileIo::validate_extension(Path::new("test.xml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_extension_no_extension() {
        let result = BpmnFileIo::validate_extension(Path::new("test"));
        assert!(result.is_err());
    }

    #[test]
    fn test_save_and_load() -> FileResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bpmn");

        let diagram = BpmnDiagram {
            id: "test_diagram".to_string(),
            name: Some("Test Diagram".to_string()),
            documentation: Some("A test diagram".to_string()),
            processes: vec![],
            collaborations: vec![],
            data_stores: vec![],
            messages: vec![],
            signals: vec![],
            diagram_info: None,
        };

        // Save
        BpmnFileIo::save(&file_path, &diagram)?;
        assert!(file_path.exists());

        // Load
        let loaded = BpmnFileIo::load(&file_path)?;
        assert_eq!(loaded.id, "test_diagram");
        assert_eq!(loaded.name, Some("Test Diagram".to_string()));

        Ok(())
    }

    #[test]
    fn test_save_creates_backup() -> FileResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bpmn");

        let diagram = BpmnDiagram {
            id: "test".to_string(),
            name: None,
            documentation: None,
            processes: vec![],
            collaborations: vec![],
            data_stores: vec![],
            messages: vec![],
            signals: vec![],
            diagram_info: None,
        };

        // First save
        BpmnFileIo::save(&file_path, &diagram)?;

        // Modify and save again
        let mut diagram2 = diagram.clone();
        diagram2.name = Some("Modified".to_string());
        BpmnFileIo::save(&file_path, &diagram2)?;

        // Backup should exist
        let backup_path = file_path.with_extension("bpmn.bak");
        assert!(backup_path.exists());

        Ok(())
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = BpmnFileIo::load("nonexistent.bpmn");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_bpmn_files() -> FileResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        let diagram = BpmnDiagram {
            id: "test".to_string(),
            name: None,
            documentation: None,
            processes: vec![],
            collaborations: vec![],
            data_stores: vec![],
            messages: vec![],
            signals: vec![],
            diagram_info: None,
        };

        // Create test files
        BpmnFileIo::save(dir_path.join("process1.bpmn"), &diagram)?;
        BpmnFileIo::save(dir_path.join("process2.bpmn"), &diagram)?;
        fs::write(dir_path.join("other.xml"), "<test />")?;

        let files = BpmnFileIo::list_in_directory(dir_path)?;
        assert_eq!(files.len(), 2);

        Ok(())
    }
}

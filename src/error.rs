use std::fmt;
use std::path::PathBuf;

use crate::format::ArchiveFormat;

#[derive(Debug)]
pub enum Error {
    FileNotFound(PathBuf),
    UnknownFormat(PathBuf),
    MissingTool {
        tool: &'static str,
        format: ArchiveFormat,
    },
    DestinationExists(PathBuf),
    ToolFailed {
        tool: &'static str,
        code: Option<i32>,
        stderr: String,
    },
    Io(std::io::Error),
    Usage(String),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Usage(_) => 1,
            Error::FileNotFound(_) => 1,
            Error::UnknownFormat(_) => 1,
            Error::DestinationExists(_) => 1,
            Error::MissingTool { .. } => 3,
            Error::ToolFailed { .. } => 4,
            Error::Io(_) => 1,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::FileNotFound(p) => write!(f, "{}: no such file", p.display()),
            Error::UnknownFormat(p) => {
                write!(f, "{}: unsupported or unrecognized format", p.display())
            }
            Error::MissingTool { tool, format } => {
                write!(
                    f,
                    "{tool} is not installed (needed for {ext} files)",
                    ext = format.extensions()[0]
                )
            }
            Error::DestinationExists(p) => {
                write!(f, "{} already exists (use -o to overwrite)", p.display())
            }
            Error::ToolFailed { tool, code, stderr } => {
                if let Some(c) = code {
                    write!(f, "{tool} exited with code {c}")
                } else {
                    write!(f, "{tool} was terminated by signal")
                }?;
                if !stderr.is_empty() {
                    write!(f, ": {stderr}")?;
                }
                Ok(())
            }
            Error::Io(e) => write!(f, "{e}"),
            Error::Usage(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

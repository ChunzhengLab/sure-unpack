use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::format::ArchiveFormat;

/// Search PATH for a tool binary. Returns the full path if found.
pub fn ensure(
    tool: &'static str,
    candidates: &[&str],
    format: ArchiveFormat,
) -> Result<PathBuf, Error> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in path_var.split(':') {
        for name in candidates {
            let candidate = Path::new(dir).join(name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    Err(Error::MissingTool { tool, format })
}

/// Return the display name and search candidates for a format's tool.
pub fn for_format(format: ArchiveFormat) -> (&'static str, &'static [&'static str]) {
    match format {
        ArchiveFormat::Tar
        | ArchiveFormat::TarGz
        | ArchiveFormat::TarBz2
        | ArchiveFormat::TarXz
        | ArchiveFormat::TarZst => ("tar", &["tar"]),
        ArchiveFormat::Zip => ("unzip", &["unzip"]),
        ArchiveFormat::SevenZ | ArchiveFormat::Rar | ArchiveFormat::Iso => ("7z", &["7z", "7zz"]),
        ArchiveFormat::Gz => ("gunzip", &["gunzip"]),
        ArchiveFormat::Bz2 => ("bunzip2", &["bunzip2"]),
        ArchiveFormat::Xz => ("xz", &["xz"]),
        ArchiveFormat::TarLz4 | ArchiveFormat::Lz4 => ("lz4", &["lz4"]),
        ArchiveFormat::Zst => ("zstd", &["zstd"]),
    }
}

/// Convenience: ensure the unpack tool for a given format.
pub fn ensure_for(format: ArchiveFormat) -> Result<PathBuf, Error> {
    let (name, candidates) = for_format(format);
    ensure(name, candidates, format)
}

/// Tool mapping for packing (different tools: zip instead of unzip, gzip instead of gunzip).
pub fn for_format_pack(format: ArchiveFormat) -> (&'static str, &'static [&'static str]) {
    match format {
        ArchiveFormat::Tar
        | ArchiveFormat::TarGz
        | ArchiveFormat::TarBz2
        | ArchiveFormat::TarXz
        | ArchiveFormat::TarZst => ("tar", &["tar"]),
        ArchiveFormat::Zip => ("zip", &["zip"]),
        ArchiveFormat::SevenZ => ("7z", &["7z", "7zz"]),
        ArchiveFormat::Gz => ("gzip", &["gzip"]),
        ArchiveFormat::Bz2 => ("bzip2", &["bzip2"]),
        ArchiveFormat::Xz => ("xz", &["xz"]),
        ArchiveFormat::TarLz4 | ArchiveFormat::Lz4 => ("lz4", &["lz4"]),
        ArchiveFormat::Zst => ("zstd", &["zstd"]),
        // rar/iso are read-only formats
        ArchiveFormat::Rar | ArchiveFormat::Iso => ("7z", &["7z", "7zz"]),
    }
}

use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::format::ArchiveFormat;
use crate::tool;

mod lz4;
mod seven_z;
mod single;
mod tar;
mod zip;

#[derive(Debug, Clone, Copy)]
pub enum Backend {
    Tar,
    Zip,
    SevenZ,
    Gunzip,
    Bunzip2,
    Xz,
    Lz4,
    Zstd,
}

impl Backend {
    pub fn from_format(fmt: ArchiveFormat) -> Self {
        match fmt {
            ArchiveFormat::Tar
            | ArchiveFormat::TarGz
            | ArchiveFormat::TarBz2
            | ArchiveFormat::TarXz
            | ArchiveFormat::TarZst => Backend::Tar,
            ArchiveFormat::Zip => Backend::Zip,
            ArchiveFormat::SevenZ | ArchiveFormat::Rar | ArchiveFormat::Iso => Backend::SevenZ,
            ArchiveFormat::Gz => Backend::Gunzip,
            ArchiveFormat::Bz2 => Backend::Bunzip2,
            ArchiveFormat::Xz => Backend::Xz,
            ArchiveFormat::TarLz4 | ArchiveFormat::Lz4 => Backend::Lz4,
            ArchiveFormat::Zst => Backend::Zstd,
        }
    }

    pub fn tool_name(&self) -> &'static str {
        match self {
            Backend::Tar => "tar",
            Backend::Zip => "unzip",
            Backend::SevenZ => "7z",
            Backend::Gunzip => "gunzip",
            Backend::Bunzip2 => "bunzip2",
            Backend::Xz => "xz",
            Backend::Lz4 => "lz4",
            Backend::Zstd => "zstd",
        }
    }

    pub fn display_name(&self, format: ArchiveFormat) -> &'static str {
        match (self, format) {
            (Backend::Lz4, ArchiveFormat::TarLz4) => "tar + lz4",
            _ => self.tool_name(),
        }
    }

    pub fn supports_strip_components(&self) -> bool {
        matches!(self, Backend::Tar)
    }

    pub fn ensure_tool(&self, format: ArchiveFormat) -> Result<PathBuf, Error> {
        match (self, format) {
            (Backend::Lz4, ArchiveFormat::TarLz4) => {
                tool::ensure("tar", &["tar"], format)?;
                tool::ensure("lz4", &["lz4"], format)
            }
            _ => tool::ensure_for(format),
        }
    }

    pub fn list(&self, archive: &Path, format: ArchiveFormat) -> Result<Vec<String>, Error> {
        match self {
            Backend::Gunzip | Backend::Bunzip2 | Backend::Xz | Backend::Zstd => {
                single::list(archive, format)
            }
            Backend::Lz4 => lz4::list(archive, format),
            _ => {
                let tool_path = self.ensure_tool(format)?;
                match self {
                    Backend::Tar => tar::list(&tool_path, archive, format),
                    Backend::Zip => zip::list(&tool_path, archive),
                    Backend::SevenZ => seven_z::list(&tool_path, archive),
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn extract(
        &self,
        archive: &Path,
        dest: &Path,
        format: ArchiveFormat,
        strip_components: u32,
        overwrite: bool,
        verbose: bool,
    ) -> Result<(), Error> {
        match self {
            Backend::Tar => {
                let tool_path = self.ensure_tool(format)?;
                tar::extract(&tool_path, archive, dest, format, strip_components, verbose)
            }
            Backend::Zip => {
                let tool_path = self.ensure_tool(format)?;
                zip::extract(&tool_path, archive, dest, overwrite, verbose)
            }
            Backend::SevenZ => {
                let tool_path = self.ensure_tool(format)?;
                seven_z::extract(&tool_path, archive, dest, overwrite, verbose)
            }
            Backend::Gunzip | Backend::Bunzip2 | Backend::Xz | Backend::Zstd => {
                let tool_path = self.ensure_tool(format)?;
                single::extract(&tool_path, self.tool_name(), archive, dest)
            }
            Backend::Lz4 => lz4::extract(archive, dest, format, strip_components, verbose),
        }
    }
}

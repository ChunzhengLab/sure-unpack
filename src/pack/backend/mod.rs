use std::path::Path;

use crate::error::Error;
use crate::format::ArchiveFormat;

mod lz4;
mod seven_z;
mod single;
mod tar;
mod zip;

pub fn pack(
    tool: &Path,
    tool_name: &'static str,
    source: &Path,
    output: &Path,
    format: ArchiveFormat,
    verbose: bool,
) -> Result<(), Error> {
    match format {
        ArchiveFormat::Tar
        | ArchiveFormat::TarGz
        | ArchiveFormat::TarBz2
        | ArchiveFormat::TarXz
        | ArchiveFormat::TarZst => tar::pack(tool, source, output, format, verbose),
        ArchiveFormat::TarLz4 | ArchiveFormat::Lz4 => {
            lz4::pack(tool, source, output, format, verbose)
        }
        ArchiveFormat::Zip => zip::pack(tool, source, output, verbose),
        ArchiveFormat::SevenZ => seven_z::pack(tool, source, output, verbose),
        ArchiveFormat::Gz | ArchiveFormat::Bz2 | ArchiveFormat::Xz | ArchiveFormat::Zst => {
            single::pack(tool, tool_name, source, output, verbose)
        }
        _ => Err(Error::Usage(format!(
            "packing {} is not supported",
            format.extensions()[0]
        ))),
    }
}

use std::path::Path;
use std::process::Command;

use crate::error::Error;
use crate::format::ArchiveFormat;

fn compression_flags(format: ArchiveFormat) -> &'static [&'static str] {
    match format {
        ArchiveFormat::TarGz => &["-z"],
        ArchiveFormat::TarBz2 => &["-j"],
        ArchiveFormat::TarXz => &["-J"],
        ArchiveFormat::TarZst => &["--zstd"],
        ArchiveFormat::Tar => &[],
        _ => &[],
    }
}

pub fn list(tool: &Path, archive: &Path, format: ArchiveFormat) -> Result<Vec<String>, Error> {
    let mut cmd = Command::new(tool);
    for flag in compression_flags(format) {
        cmd.arg(flag);
    }
    cmd.arg("-tf").arg(archive);

    let output = cmd.output()?;
    if !output.status.success() {
        return Err(Error::ToolFailed {
            tool: "tar",
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect())
}

/// Overwrite protection for tar is handled by the caller via pre-flight
/// member check, not by tar flags. This avoids relying on `-k` behavior
/// which varies between BSD tar and GNU tar.
pub fn extract(
    tool: &Path,
    archive: &Path,
    dest: &Path,
    format: ArchiveFormat,
    strip_components: u32,
    verbose: bool,
) -> Result<(), Error> {
    let mut cmd = Command::new(tool);
    for flag in compression_flags(format) {
        cmd.arg(flag);
    }
    cmd.arg("-xf").arg(archive).arg("-C").arg(dest);

    if strip_components > 0 {
        cmd.arg(format!("--strip-components={strip_components}"));
    }
    if verbose {
        cmd.arg("-v");
    }

    let output = cmd.output()?;
    if !output.status.success() {
        return Err(Error::ToolFailed {
            tool: "tar",
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into(),
        });
    }

    if verbose {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            eprint!("{stdout}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compression_flag_mapping() {
        assert_eq!(compression_flags(ArchiveFormat::TarGz), &["-z"]);
        assert_eq!(compression_flags(ArchiveFormat::TarBz2), &["-j"]);
        assert_eq!(compression_flags(ArchiveFormat::TarXz), &["-J"]);
        assert_eq!(compression_flags(ArchiveFormat::TarZst), &["--zstd"]);
        assert!(compression_flags(ArchiveFormat::Tar).is_empty());
    }
}

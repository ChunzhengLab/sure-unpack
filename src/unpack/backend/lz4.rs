use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::Error;
use crate::format::ArchiveFormat;
use crate::tool;

use super::single;

pub fn list(archive: &Path, format: ArchiveFormat) -> Result<Vec<String>, Error> {
    match format {
        ArchiveFormat::Lz4 => single::list(archive, format),
        ArchiveFormat::TarLz4 => list_tar_lz4(archive),
        _ => unreachable!(),
    }
}

pub fn extract(
    archive: &Path,
    dest: &Path,
    format: ArchiveFormat,
    verbose: bool,
) -> Result<(), Error> {
    match format {
        ArchiveFormat::Lz4 => {
            let tool = tool::ensure("lz4", &["lz4"], format)?;
            single::extract(&tool, "lz4", archive, dest)
        }
        ArchiveFormat::TarLz4 => extract_tar_lz4(archive, dest, verbose),
        _ => unreachable!(),
    }
}

fn list_tar_lz4(archive: &Path) -> Result<Vec<String>, Error> {
    let tar = tool::ensure("tar", &["tar"], ArchiveFormat::TarLz4)?;
    let lz4 = tool::ensure("lz4", &["lz4"], ArchiveFormat::TarLz4)?;

    let mut lz4_child = Command::new(lz4)
        .args(["-d", "-c", "-q"])
        .arg(archive)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let lz4_stdout = lz4_child.stdout.take().unwrap();
    let tar_out = Command::new(tar)
        .arg("-tf")
        .arg("-")
        .stdin(Stdio::from(lz4_stdout))
        .output()?;

    let lz4_out = lz4_child.wait_with_output()?;
    if !lz4_out.status.success() {
        return Err(Error::ToolFailed {
            tool: "lz4",
            code: lz4_out.status.code(),
            stderr: String::from_utf8_lossy(&lz4_out.stderr).into(),
        });
    }
    if !tar_out.status.success() {
        return Err(Error::ToolFailed {
            tool: "tar",
            code: tar_out.status.code(),
            stderr: String::from_utf8_lossy(&tar_out.stderr).into(),
        });
    }

    Ok(String::from_utf8_lossy(&tar_out.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect())
}

fn extract_tar_lz4(archive: &Path, dest: &Path, verbose: bool) -> Result<(), Error> {
    let tar = tool::ensure("tar", &["tar"], ArchiveFormat::TarLz4)?;
    let lz4 = tool::ensure("lz4", &["lz4"], ArchiveFormat::TarLz4)?;

    let mut lz4_child = Command::new(lz4)
        .args(["-d", "-c", "-q"])
        .arg(archive)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let lz4_stdout = lz4_child.stdout.take().unwrap();
    let mut tar_cmd = Command::new(tar);
    tar_cmd.arg("-xf").arg("-").arg("-C").arg(dest);
    if verbose {
        tar_cmd.arg("-v");
    }
    let tar_out = tar_cmd.stdin(Stdio::from(lz4_stdout)).output()?;

    let lz4_out = lz4_child.wait_with_output()?;
    if !lz4_out.status.success() {
        return Err(Error::ToolFailed {
            tool: "lz4",
            code: lz4_out.status.code(),
            stderr: String::from_utf8_lossy(&lz4_out.stderr).into(),
        });
    }
    if !tar_out.status.success() {
        return Err(Error::ToolFailed {
            tool: "tar",
            code: tar_out.status.code(),
            stderr: String::from_utf8_lossy(&tar_out.stderr).into(),
        });
    }

    if verbose {
        let stdout = String::from_utf8_lossy(&tar_out.stdout);
        if !stdout.is_empty() {
            eprint!("{stdout}");
        }
    }

    Ok(())
}

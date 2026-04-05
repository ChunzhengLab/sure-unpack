use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::Error;
use crate::format::ArchiveFormat;
use crate::tool;

use super::single;

pub fn pack(
    tool: &Path,
    source: &Path,
    output: &Path,
    format: ArchiveFormat,
    verbose: bool,
) -> Result<(), Error> {
    match format {
        ArchiveFormat::Lz4 => single::pack(tool, "lz4", source, output, verbose),
        ArchiveFormat::TarLz4 => pack_tar_lz4(tool, source, output, verbose),
        _ => unreachable!(),
    }
}

fn pack_tar_lz4(lz4: &Path, source: &Path, output: &Path, verbose: bool) -> Result<(), Error> {
    let tar = tool::ensure("tar", &["tar"], ArchiveFormat::TarLz4)?;
    let parent = source
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = source
        .file_name()
        .ok_or_else(|| Error::Usage("invalid source path".into()))?;

    let mut tar_child = Command::new(tar);
    tar_child.arg("-cf").arg("-");
    if verbose {
        tar_child.arg("-v");
    }
    tar_child.arg("-C").arg(parent).arg(name);
    let mut tar_child = tar_child
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let tar_stdout = tar_child.stdout.take().unwrap();
    let output_file = File::create(output)?;
    let mut lz4_cmd = Command::new(lz4);
    if verbose {
        lz4_cmd.args(["-v", "-c"]);
    } else {
        lz4_cmd.args(["-q", "-c"]);
    }
    let lz4_out = lz4_cmd
        .stdin(Stdio::from(tar_stdout))
        .stdout(Stdio::from(output_file))
        .stderr(Stdio::piped())
        .output()?;

    let tar_out = tar_child.wait_with_output()?;
    if !tar_out.status.success() {
        let _ = std::fs::remove_file(output);
        return Err(Error::ToolFailed {
            tool: "tar",
            code: tar_out.status.code(),
            stderr: String::from_utf8_lossy(&tar_out.stderr).into(),
        });
    }
    if !lz4_out.status.success() {
        let _ = std::fs::remove_file(output);
        return Err(Error::ToolFailed {
            tool: "lz4",
            code: lz4_out.status.code(),
            stderr: String::from_utf8_lossy(&lz4_out.stderr).into(),
        });
    }
    if verbose {
        let tar_stderr = String::from_utf8_lossy(&tar_out.stderr);
        if !tar_stderr.is_empty() {
            eprint!("{tar_stderr}");
        }
        let lz4_stderr = String::from_utf8_lossy(&lz4_out.stderr);
        if !lz4_stderr.is_empty() {
            eprint!("{lz4_stderr}");
        }
    }
    Ok(())
}

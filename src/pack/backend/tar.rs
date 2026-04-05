use std::path::Path;
use std::process::Command;

use crate::error::Error;
use crate::format::ArchiveFormat;

pub fn pack(
    tool: &Path,
    source: &Path,
    output: &Path,
    format: ArchiveFormat,
    verbose: bool,
) -> Result<(), Error> {
    let parent = source.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let name = source
        .file_name()
        .ok_or_else(|| Error::Usage("invalid source path".into()))?;

    let output_abs = if output.is_absolute() {
        output.to_path_buf()
    } else {
        std::env::current_dir()?.join(output)
    };

    let mut cmd = Command::new(tool);
    for flag in format.tar_compression_flags() {
        cmd.arg(flag);
    }
    cmd.arg("-cf").arg(&output_abs);
    if verbose {
        cmd.arg("-v");
    }
    cmd.arg("-C").arg(parent).arg(name);

    let out = cmd.output()?;
    if !out.status.success() {
        return Err(Error::ToolFailed {
            tool: "tar",
            code: out.status.code(),
            stderr: String::from_utf8_lossy(&out.stderr).into(),
        });
    }
    Ok(())
}

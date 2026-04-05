use std::path::Path;
use std::process::Command;

use crate::error::Error;

pub fn pack(tool: &Path, source: &Path, output: &Path, verbose: bool) -> Result<(), Error> {
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
    cmd.arg("a").arg(&output_abs).arg(name);
    if !verbose {
        cmd.arg("-bd");
    }
    cmd.current_dir(parent);

    let out = cmd.output()?;
    if !out.status.success() {
        return Err(Error::ToolFailed {
            tool: "7z",
            code: out.status.code(),
            stderr: String::from_utf8_lossy(&out.stderr).into(),
        });
    }
    Ok(())
}

use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::Error;

pub fn pack(
    tool: &Path,
    tool_name: &'static str,
    source: &Path,
    output: &Path,
    verbose: bool,
) -> Result<(), Error> {
    let args: &[&str] = match tool_name {
        "gzip" | "bzip2" | "xz" if verbose => &["-v", "-c"],
        "lz4" if verbose => &["-v", "-c"],
        "lz4" => &["-q", "-c"],
        "zstd" if verbose => &["-v", "-c", "--no-progress"],
        "zstd" => &["-c", "--no-progress"],
        _ => &["-c"],
    };

    let mut child = Command::new(tool)
        .args(args)
        .arg(source)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut stdout = child.stdout.take().unwrap();
    let mut file = File::create(output)?;
    std::io::copy(&mut stdout, &mut file)?;

    let status = child.wait()?;
    if !status.success() {
        let _ = std::fs::remove_file(output);
        return Err(Error::ToolFailed {
            tool: tool_name,
            code: status.code(),
            stderr: String::new(),
        });
    }
    Ok(())
}

use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::Error;
use crate::format::{self, ArchiveFormat};

/// For single-file compression, "listing" just returns the decompressed filename.
/// Pure logic — no external tool needed.
pub fn list(archive: &Path, format: ArchiveFormat) -> Result<Vec<String>, Error> {
    Ok(vec![format::archive_stem(archive, format)])
}

/// Extract single-file compression by streaming tool stdout to output file.
/// stderr is inherited so the user sees tool errors directly; this also
/// avoids a potential pipe deadlock from buffering both streams.
pub fn extract(
    tool: &Path,
    tool_name: &'static str,
    archive: &Path,
    dest: &Path,
) -> Result<(), Error> {
    let args: &[&str] = match tool_name {
        "gunzip" | "bunzip2" => &["-c"],
        "xz" => &["-dc"],
        "lz4" => &["-d", "-c", "-q"],
        "zstd" => &["-dc", "--no-progress"],
        _ => &["-dc"],
    };

    let mut child = Command::new(tool)
        .args(args)
        .arg(archive)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut stdout = child.stdout.take().unwrap();
    let mut file = File::create(dest)?;
    std::io::copy(&mut stdout, &mut file)?;

    let status = child.wait()?;
    if !status.success() {
        let _ = std::fs::remove_file(dest);
        return Err(Error::ToolFailed {
            tool: tool_name,
            code: status.code(),
            stderr: String::new(),
        });
    }

    Ok(())
}

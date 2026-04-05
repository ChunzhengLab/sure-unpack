use std::path::Path;
use std::process::Command;

use crate::error::Error;

pub fn list(tool: &Path, archive: &Path) -> Result<Vec<String>, Error> {
    let output = Command::new(tool).arg("l").arg(archive).output()?;
    if !output.status.success() {
        return Err(Error::ToolFailed {
            tool: "7z",
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into(),
        });
    }

    Ok(parse_7z_list(&String::from_utf8_lossy(&output.stdout)))
}

/// Parse `7z l` output to extract filenames.
///
/// The relevant section is between two "---" separator lines.
/// Each data line: `Date Time Attr Size Compressed Name`
fn parse_7z_list(stdout: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let mut in_table = false;
    let mut dash_count = 0;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("---") {
            dash_count += 1;
            if dash_count == 1 {
                in_table = true;
            } else {
                break;
            }
            continue;
        }
        if in_table && !trimmed.is_empty() {
            // Skip 5 fields: date, time, attr, size, compressed
            let mut rest = trimmed;
            for _ in 0..5 {
                rest = rest.trim_start();
                if let Some(pos) = rest.find(char::is_whitespace) {
                    rest = &rest[pos..];
                } else {
                    rest = "";
                    break;
                }
            }
            let name = rest.trim_start();
            if !name.is_empty() {
                entries.push(name.to_string());
            }
        }
    }
    entries
}

pub fn extract(
    tool: &Path,
    archive: &Path,
    dest: &Path,
    overwrite: bool,
    verbose: bool,
) -> Result<(), Error> {
    let mut cmd = Command::new(tool);
    cmd.arg("x").arg(archive);
    cmd.arg(format!("-o{}", dest.display())); // 7z uses -o without space
    // -aoa = overwrite all, -aos = skip existing
    cmd.arg(if overwrite { "-aoa" } else { "-aos" });
    if !verbose {
        cmd.arg("-bd");
    }
    cmd.arg("-y");

    let output = cmd.output()?;
    if !output.status.success() {
        return Err(Error::ToolFailed {
            tool: "7z",
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_7z_output() {
        let output = "\
7-Zip 23.01 (x64) : Copyright (c) 1999-2023 Igor Pavlov

Listing archive: test.7z

--
Path = test.7z
Type = 7z

   Date      Time    Attr         Size   Compressed  Name
------------------- ----- ------------ ------------  ------------------------
2024-01-01 12:00:00 D....            0            0  mydir
2024-01-01 12:00:00 .....          123          100  mydir/hello.txt
------------------- ----- ------------ ------------  ------------------------
                                   123          100  2 files";
        let entries = parse_7z_list(output);
        assert_eq!(entries, vec!["mydir", "mydir/hello.txt"]);
    }
}

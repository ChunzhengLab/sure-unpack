use std::path::Path;
use std::process::Command;

use crate::error::Error;

pub fn list(tool: &Path, archive: &Path) -> Result<Vec<String>, Error> {
    let output = Command::new(tool).arg("-l").arg(archive).output()?;
    if !output.status.success() {
        return Err(Error::ToolFailed {
            tool: "unzip",
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into(),
        });
    }

    Ok(parse_unzip_list(&String::from_utf8_lossy(&output.stdout)))
}

/// Parse `unzip -l` output to extract filenames.
///
/// Output format:
/// ```text
///   Archive:  foo.zip
///     Length      Date    Time    Name
///   ---------  ---------- -----   ----
///         123  2024-01-01 12:00   dir/file.txt
///           0  2024-01-01 12:00   dir/
///   ---------                     -------
///        123                     2 files
/// ```
fn parse_unzip_list(stdout: &str) -> Vec<String> {
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
            // Skip 3 whitespace-separated fields (length, date, time), take the rest as name
            let mut rest = trimmed;
            for _ in 0..3 {
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
    if !verbose {
        cmd.arg("-q");
    }
    cmd.arg(if overwrite { "-o" } else { "-n" });
    cmd.arg(archive).arg("-d").arg(dest);

    let output = cmd.output()?;
    // unzip returns 0 on success, 1 on warnings, 2+ on errors.
    if output.status.code().unwrap_or(2) >= 2 {
        return Err(Error::ToolFailed {
            tool: "unzip",
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
    fn parse_unzip_output() {
        let output = "\
  Archive:  test.zip
    Length      Date    Time    Name
  ---------  ---------- -----   ----
        123  2024-01-01 12:00   dir/file.txt
          0  2024-01-01 12:00   dir/
  ---------                     -------
       123                     2 files";
        let entries = parse_unzip_list(output);
        assert_eq!(entries, vec!["dir/file.txt", "dir/"]);
    }
}

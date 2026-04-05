use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sure_unpack() -> Command {
    Command::new(env!("CARGO_BIN_EXE_unpack"))
}

fn has_tool(name: &str) -> bool {
    // Search PATH manually instead of depending on `which`
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in path_var.split(':') {
        if Path::new(dir).join(name).is_file() {
            return true;
        }
    }
    false
}

fn temp_dir(name: &str) -> PathBuf {
    // Include PID to avoid collisions with parallel test runs or other processes
    let dir = std::env::temp_dir().join(format!(
        "sure-unpack-test-{}-{name}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn create_tar_gz(dir: &Path) -> PathBuf {
    let src_dir = dir.join("src");
    fs::create_dir_all(src_dir.join("mydir")).unwrap();
    fs::write(src_dir.join("mydir/hello.txt"), "hello world\n").unwrap();
    fs::write(src_dir.join("mydir/bye.txt"), "goodbye\n").unwrap();

    let archive = dir.join("test.tar.gz");
    let status = Command::new("tar")
        .args(["-czf"])
        .arg(&archive)
        .arg("-C")
        .arg(&src_dir)
        .arg("mydir")
        .status()
        .unwrap();
    assert!(status.success());
    archive
}

fn create_zip(dir: &Path) -> PathBuf {
    let src_dir = dir.join("src");
    fs::create_dir_all(src_dir.join("mydir")).unwrap();
    fs::write(src_dir.join("mydir/hello.txt"), "hello world\n").unwrap();

    let archive = dir.join("test.zip");
    let status = Command::new("zip")
        .args(["-r"])
        .arg(&archive)
        .arg("mydir")
        .current_dir(&src_dir)
        .status()
        .unwrap();
    assert!(status.success());
    archive
}

fn create_gz(dir: &Path) -> PathBuf {
    let src = dir.join("hello.txt");
    fs::write(&src, "hello world\n").unwrap();
    let status = Command::new("gzip")
        .arg("-k")
        .arg(&src)
        .status()
        .unwrap();
    assert!(status.success());
    dir.join("hello.txt.gz")
}

// ---- Tests ----

#[test]
fn list_tar_gz() {
    if !has_tool("tar") {
        return;
    }
    let dir = temp_dir("list-tar-gz");
    let archive = create_tar_gz(&dir);

    let output = sure_unpack().arg("list").arg(&archive).output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("mydir/hello.txt"));
    assert!(stdout.contains("mydir/bye.txt"));
}

#[test]
fn list_zip() {
    if !has_tool("unzip") || !has_tool("zip") {
        return;
    }
    let dir = temp_dir("list-zip");
    let archive = create_zip(&dir);

    let output = sure_unpack().arg("-l").arg(&archive).output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("mydir/hello.txt"));
}

#[test]
fn extract_tar_gz() {
    if !has_tool("tar") {
        return;
    }
    let dir = temp_dir("extract-tar-gz");
    let archive = create_tar_gz(&dir);
    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    let output = sure_unpack()
        .arg(&archive)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    // Should create work/test/ with mydir inside
    assert!(work.join("test/mydir/hello.txt").exists());
    assert!(work.join("test/mydir/bye.txt").exists());
}

#[test]
fn extract_zip() {
    if !has_tool("unzip") || !has_tool("zip") {
        return;
    }
    let dir = temp_dir("extract-zip");
    let archive = create_zip(&dir);
    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    let output = sure_unpack()
        .arg(&archive)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(work.join("test/mydir/hello.txt").exists());
}

#[test]
fn extract_gz_single() {
    if !has_tool("gunzip") {
        return;
    }
    let dir = temp_dir("extract-gz");
    let archive = create_gz(&dir);
    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    let output = sure_unpack()
        .arg(&archive)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let extracted = work.join("hello.txt");
    assert!(extracted.exists());
    assert_eq!(fs::read_to_string(&extracted).unwrap(), "hello world\n");
}

#[test]
fn refuse_overwrite() {
    if !has_tool("tar") {
        return;
    }
    let dir = temp_dir("refuse-overwrite");
    let archive = create_tar_gz(&dir);
    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    // First extract succeeds
    let output = sure_unpack()
        .arg(&archive)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(output.status.success());

    // Second extract should fail
    let output = sure_unpack()
        .arg(&archive)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"));
}

#[test]
fn overwrite_flag() {
    if !has_tool("tar") {
        return;
    }
    let dir = temp_dir("overwrite-flag");
    let archive = create_tar_gz(&dir);
    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    // First extract
    let output = sure_unpack()
        .arg(&archive)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(output.status.success());

    // Second extract with -o should succeed
    let output = sure_unpack()
        .arg("-o")
        .arg(&archive)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn extract_into_dir() {
    if !has_tool("tar") {
        return;
    }
    let dir = temp_dir("extract-into");
    let archive = create_tar_gz(&dir);
    let target = dir.join("custom-output");

    let output = sure_unpack()
        .arg("-C")
        .arg(&target)
        .arg(&archive)
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(target.join("test/mydir/hello.txt").exists());
}

#[test]
fn file_not_found() {
    let output = sure_unpack().arg("/nonexistent/file.tar.gz").output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no such file"));
}

#[test]
fn unknown_format() {
    let dir = temp_dir("unknown-format");
    let f = dir.join("readme.txt");
    fs::write(&f, "not an archive").unwrap();

    let output = sure_unpack().arg(&f).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unsupported"));
}

#[test]
fn help_flag() {
    let output = sure_unpack().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("unpack"));
    assert!(stdout.contains("USAGE"));
}

#[test]
fn here_flag_tar_gz() {
    if !has_tool("tar") {
        return;
    }
    let dir = temp_dir("here-tar-gz");
    let archive = create_tar_gz(&dir);
    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    // --here should extract directly into work/, not create a subdirectory
    let output = sure_unpack()
        .arg("--here")
        .arg(&archive)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    // mydir/ should be directly in work/, not in work/test/
    assert!(work.join("mydir/hello.txt").exists());
    assert!(!work.join("test").exists());
}

#[test]
fn here_flag_no_overwrite() {
    if !has_tool("tar") {
        return;
    }
    let dir = temp_dir("here-no-overwrite");
    let archive = create_tar_gz(&dir);
    let work = dir.join("work");
    fs::create_dir_all(work.join("mydir")).unwrap();
    fs::write(work.join("mydir/hello.txt"), "old content").unwrap();

    // --here without -o should refuse when member files already exist
    let output = sure_unpack()
        .arg("--here")
        .arg(&archive)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"));

    // Original content must be preserved
    let content = fs::read_to_string(work.join("mydir/hello.txt")).unwrap();
    assert_eq!(content, "old content");
}

#[test]
fn overwrite_zip() {
    if !has_tool("unzip") || !has_tool("zip") {
        return;
    }
    let dir = temp_dir("overwrite-zip");
    let archive = create_zip(&dir);
    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    // First extract
    let output = sure_unpack()
        .arg(&archive)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(output.status.success());

    // Modify a file in the extracted directory
    let hello = work.join("test/mydir/hello.txt");
    fs::write(&hello, "old content").unwrap();

    // Second extract with -o should overwrite
    let output = sure_unpack()
        .arg("-o")
        .arg(&archive)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    // Content should be restored to original
    let content = fs::read_to_string(&hello).unwrap();
    assert_eq!(content, "hello world\n");
}

#[test]
fn strip_components_rejected_for_zip() {
    if !has_tool("unzip") || !has_tool("zip") {
        return;
    }
    let dir = temp_dir("strip-zip");
    let archive = create_zip(&dir);

    let output = sure_unpack()
        .arg("--strip-components")
        .arg("1")
        .arg(&archive)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not supported"));
}

#[test]
fn single_file_list_without_tool() {
    // sure-unpack list file.txt.gz should work even if gunzip is not in PATH,
    // because listing a single-file format is pure string logic.
    let dir = temp_dir("single-list-no-tool");
    let f = dir.join("data.txt.gz");
    fs::write(&f, "fake gz content").unwrap();

    let output = sure_unpack()
        .arg("list")
        .arg(&f)
        .env("PATH", "") // empty PATH — no tools available
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "data.txt");
}

#[test]
fn strip_components_no_overwrite() {
    if !has_tool("tar") {
        return;
    }
    // Archive contains top/hello.txt. With --strip-components 1,
    // tar writes hello.txt directly. Pre-flight check must detect
    // the conflict at the stripped path, not the original.
    let dir = temp_dir("strip-no-overwrite");
    let src_dir = dir.join("src");
    fs::create_dir_all(src_dir.join("top")).unwrap();
    fs::write(src_dir.join("top/hello.txt"), "new\n").unwrap();

    let archive = dir.join("nested.tar.gz");
    let status = Command::new("tar")
        .args(["-czf"])
        .arg(&archive)
        .arg("-C")
        .arg(&src_dir)
        .arg("top")
        .status()
        .unwrap();
    assert!(status.success());

    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();
    fs::write(work.join("hello.txt"), "old\n").unwrap();

    // --strip-components 1 --here: should refuse because hello.txt exists
    let output = sure_unpack()
        .arg("--here")
        .arg("--strip-components")
        .arg("1")
        .arg(&archive)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"), "stderr: {stderr}");

    // Old file must be untouched
    let content = fs::read_to_string(work.join("hello.txt")).unwrap();
    assert_eq!(content, "old\n");
}

#[test]
fn dry_run_no_extract() {
    if !has_tool("tar") {
        return;
    }
    let dir = temp_dir("dry-run-no-extract");
    let archive = create_tar_gz(&dir);
    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    let output = sure_unpack()
        .arg("--dry-run")
        .arg(&archive)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend:"));
    assert!(stdout.contains("dest:"));
    assert!(stdout.contains("tool:      found"));
    assert!(stdout.contains("conflicts: none"));

    // Nothing should be extracted
    assert!(!work.join("test").exists());
}

#[test]
fn dry_run_with_conflict() {
    if !has_tool("tar") {
        return;
    }
    let dir = temp_dir("dry-run-conflict");
    let archive = create_tar_gz(&dir);
    let work = dir.join("work");
    fs::create_dir_all(work.join("mydir")).unwrap();
    fs::write(work.join("mydir/hello.txt"), "old").unwrap();

    // --dry-run --here: should detect conflict and exit non-zero
    let output = sure_unpack()
        .arg("--dry-run")
        .arg("--here")
        .arg(&archive)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("conflicts: 1"));
}

#[test]
fn dry_run_missing_tool_single() {
    let dir = temp_dir("dry-run-no-tool-single");
    let f = dir.join("data.txt.gz");
    fs::write(&f, "fake").unwrap();

    let output = sure_unpack()
        .arg("--dry-run")
        .arg(&f)
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tool:      NOT FOUND"));
}

#[test]
fn dry_run_missing_tool_multi() {
    if !has_tool("tar") {
        return;
    }
    let dir = temp_dir("dry-run-no-tool-multi");
    let archive = create_tar_gz(&dir);

    // Empty PATH: tar not found, dry-run should still print summary
    let output = sure_unpack()
        .arg("--dry-run")
        .arg(&archive)
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tool:      NOT FOUND"));
    assert!(stdout.contains("backend:   tar"));
}

#[test]
fn format_override_extract() {
    if !has_tool("tar") {
        return;
    }
    // Create a .tar.gz but name it .bin
    let dir = temp_dir("format-override");
    let archive = create_tar_gz(&dir);
    let renamed = dir.join("mystery.bin");
    fs::rename(&archive, &renamed).unwrap();

    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    // --format tar.gz explicitly overrides detection
    let output = sure_unpack()
        .arg("--format")
        .arg("tar.gz")
        .arg(&renamed)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    // stem is "mystery" (strip .bin since .tar.gz doesn't match)
    assert!(work.join("mystery/mydir/hello.txt").exists());
}

#[test]
fn format_override_list() {
    if !has_tool("tar") {
        return;
    }
    let dir = temp_dir("format-override-list");
    let archive = create_tar_gz(&dir);
    let renamed = dir.join("mystery.bin");
    fs::rename(&archive, &renamed).unwrap();

    // --format should work with list too
    let output = sure_unpack()
        .arg("list")
        .arg("--format")
        .arg("tar.gz")
        .arg(&renamed)
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("mydir/hello.txt"));
}

#[test]
fn sniff_detects_tar_gz() {
    if !has_tool("tar") || !has_tool("gunzip") {
        return;
    }
    // .tar.gz content named .bin — sniff + probe should detect tar.gz
    let dir = temp_dir("sniff-tar-gz");
    let archive = create_tar_gz(&dir);
    let renamed = dir.join("mystery.bin");
    fs::rename(&archive, &renamed).unwrap();

    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    // Should detect as tar.gz and extract correctly (no warning needed)
    let output = sure_unpack()
        .arg(&renamed)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(work.join("mystery/mydir/hello.txt").exists());
}

#[test]
fn sniff_fallback_zip() {
    if !has_tool("unzip") || !has_tool("zip") {
        return;
    }
    let dir = temp_dir("sniff-zip");
    let archive = create_zip(&dir);
    let renamed = dir.join("mystery.bin");
    fs::rename(&archive, &renamed).unwrap();

    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    // Sniff should detect .zip and extract correctly (no tarball ambiguity)
    let output = sure_unpack()
        .arg(&renamed)
        .current_dir(&work)
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(work.join("mystery/mydir/hello.txt").exists());
}

#[test]
fn sniff_subcommand() {
    if !has_tool("tar") || !has_tool("gunzip") || !has_tool("zip") {
        return;
    }
    let dir = temp_dir("sniff-subcommand");
    let tar_gz = create_tar_gz(&dir);
    let zip = create_zip(&dir);

    let output = sure_unpack()
        .arg("sniff")
        .arg(&tar_gz)
        .arg(&zip)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(".tar.gz"), "stdout: {stdout}");
    assert!(stdout.contains(".zip"), "stdout: {stdout}");
}

#[test]
fn sniff_subcommand_unknown() {
    let dir = temp_dir("sniff-unknown");
    let f = dir.join("readme.txt");
    fs::write(&f, "just text").unwrap();

    let output = sure_unpack()
        .arg("sniff")
        .arg(&f)
        .output()
        .unwrap();
    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("unknown"));
}

#[test]
fn sniff_subcommand_renamed() {
    if !has_tool("tar") || !has_tool("gunzip") {
        return;
    }
    let dir = temp_dir("sniff-renamed");
    let archive = create_tar_gz(&dir);
    let renamed = dir.join("mystery.bin");
    fs::rename(&archive, &renamed).unwrap();

    let output = sure_unpack()
        .arg("sniff")
        .arg(&renamed)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(".tar.gz"), "stdout: {stdout}");
}

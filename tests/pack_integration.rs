use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn pack() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pack"))
}

fn unpack() -> Command {
    Command::new(env!("CARGO_BIN_EXE_unpack"))
}

fn has_tool(name: &str) -> bool {
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in path_var.split(':') {
        if Path::new(dir).join(name).is_file() {
            return true;
        }
    }
    false
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pack-test-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn create_source_dir(dir: &Path) -> PathBuf {
    let src = dir.join("mydir");
    fs::create_dir_all(src.join("sub")).unwrap();
    fs::write(src.join("hello.txt"), "hello\n").unwrap();
    fs::write(src.join("sub/deep.txt"), "deep\n").unwrap();
    src
}

fn create_source_file(dir: &Path) -> PathBuf {
    let f = dir.join("data.txt");
    fs::write(&f, "some data\n").unwrap();
    f
}

// --- Basic packing ---

#[test]
fn pack_default_zip() {
    if !has_tool("zip") {
        return;
    }
    let dir = temp_dir("default-zip");
    let src = create_source_dir(&dir);

    let output = pack().arg(&src).current_dir(&dir).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.join("mydir.zip").exists());
}

#[test]
fn pack_tar_gz() {
    if !has_tool("tar") {
        return;
    }
    let dir = temp_dir("tar-gz");
    let src = create_source_dir(&dir);
    let out = dir.join("out.tar.gz");

    let output = pack().arg(&src).arg(&out).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out.exists());
}

#[test]
fn pack_single_gz() {
    if !has_tool("gzip") {
        return;
    }
    let dir = temp_dir("single-gz");
    let src = create_source_file(&dir);
    let out = dir.join("data.txt.gz");

    let output = pack().arg(&src).arg(&out).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out.exists());
}

#[test]
fn pack_single_lz4() {
    if !has_tool("lz4") {
        return;
    }
    let dir = temp_dir("single-lz4");
    let src = create_source_file(&dir);
    let out = dir.join("data.txt.lz4");

    let output = pack().arg(&src).arg(&out).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out.exists());
}

#[test]
fn pack_tar_lz4() {
    if !has_tool("tar") || !has_tool("lz4") {
        return;
    }
    let dir = temp_dir("tar-lz4");
    let src = create_source_dir(&dir);
    let out = dir.join("out.tar.lz4");

    let output = pack().arg(&src).arg(&out).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out.exists());
}

// --- Validation ---

#[test]
fn pack_dir_gz_refused() {
    let dir = temp_dir("dir-gz-refused");
    let src = create_source_dir(&dir);

    let output = pack().arg(&src).arg(dir.join("out.gz")).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not supported"));
    assert!(stderr.contains(".tar.gz"));
}

#[test]
fn pack_invalid_format() {
    let dir = temp_dir("invalid-format");
    let src = create_source_dir(&dir);

    // --format nonsense should fail even if output extension is valid
    let output = pack()
        .arg("-f")
        .arg("nonsense")
        .arg(&src)
        .arg(dir.join("out.zip"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown format"), "stderr: {stderr}");
}

#[test]
fn pack_rar_not_supported() {
    let dir = temp_dir("rar-not-supported");
    let src = create_source_dir(&dir);

    let output = pack()
        .arg("-f")
        .arg("rar")
        .arg(&src)
        .arg(dir.join("out.rar"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not supported"), "stderr: {stderr}");
}

#[test]
fn pack_iso_not_supported() {
    let dir = temp_dir("iso-not-supported");
    let src = create_source_dir(&dir);

    let output = pack().arg(&src).arg(dir.join("out.iso")).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not supported"), "stderr: {stderr}");
}

#[test]
fn pack_format_conflict() {
    let dir = temp_dir("format-conflict");
    let src = create_source_dir(&dir);

    let output = pack()
        .arg("-f")
        .arg("zip")
        .arg(&src)
        .arg(dir.join("out.tar.gz"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("conflicts"));
}

#[test]
fn pack_refuse_overwrite() {
    if !has_tool("zip") {
        return;
    }
    let dir = temp_dir("refuse-overwrite");
    let src = create_source_dir(&dir);
    let out = dir.join("out.zip");

    // First pack
    let output = pack().arg(&src).arg(&out).output().unwrap();
    assert!(output.status.success());

    // Second pack should refuse
    let output = pack().arg(&src).arg(&out).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"));
}

#[test]
fn pack_overwrite_flag() {
    if !has_tool("zip") {
        return;
    }
    let dir = temp_dir("overwrite-flag");
    let src = create_source_dir(&dir);
    let out = dir.join("out.zip");

    pack().arg(&src).arg(&out).output().unwrap();

    let output = pack().arg("-o").arg(&src).arg(&out).output().unwrap();
    assert!(output.status.success());
}

#[test]
fn pack_dry_run() {
    if !has_tool("zip") {
        return;
    }
    let dir = temp_dir("dry-run");
    let src = create_source_dir(&dir);

    let output = pack()
        .arg("--dry-run")
        .arg(&src)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("format:    .zip"));
    assert!(stdout.contains("backend:   zip"));

    // No file should be created
    assert!(!dir.join("mydir.zip").exists());
}

#[test]
fn pack_verbose_tar_gz() {
    if !has_tool("tar") {
        return;
    }
    let dir = temp_dir("verbose-tar-gz");
    let src = create_source_dir(&dir);
    let out = dir.join("out.tar.gz");

    let output = pack().arg("-v").arg(&src).arg(&out).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("mydir"), "output: {combined}");
}

#[test]
fn pack_verbose_zip() {
    if !has_tool("zip") {
        return;
    }
    let dir = temp_dir("verbose-zip");
    let src = create_source_dir(&dir);
    let out = dir.join("out.zip");

    let output = pack().arg("-v").arg(&src).arg(&out).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("mydir"), "output: {combined}");
}

#[test]
fn pack_verbose_7z() {
    if !has_tool("7z") && !has_tool("7zz") {
        return;
    }
    let dir = temp_dir("verbose-7z");
    let src = create_source_dir(&dir);
    let out = dir.join("out.7z");

    let output = pack().arg("-v").arg(&src).arg(&out).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("Everything is Ok") || combined.contains("Scanning the drive"));
}

#[test]
fn pack_verbose_tar_lz4() {
    if !has_tool("tar") || !has_tool("lz4") {
        return;
    }
    let dir = temp_dir("verbose-tar-lz4");
    let src = create_source_dir(&dir);
    let out = dir.join("out.tar.lz4");

    let output = pack().arg("-v").arg(&src).arg(&out).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("mydir"), "output: {combined}");
}

#[test]
fn pack_source_not_found() {
    let output = pack().arg("/nonexistent/dir").output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no such file"));
}

#[test]
fn pack_help() {
    let output = pack().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pack"));
    assert!(stdout.contains("USAGE"));
}

// --- Round-trip tests ---

#[test]
fn roundtrip_zip() {
    if !has_tool("zip") || !has_tool("unzip") {
        return;
    }
    let dir = temp_dir("roundtrip-zip");
    let src = create_source_dir(&dir);
    let archive = dir.join("test.zip");
    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    // Pack
    let output = pack().arg(&src).arg(&archive).output().unwrap();
    assert!(
        output.status.success(),
        "pack: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Unpack
    let output = unpack().arg(&archive).current_dir(&work).output().unwrap();
    assert!(
        output.status.success(),
        "unpack: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify
    assert!(work.join("test/mydir/hello.txt").exists());
    assert_eq!(
        fs::read_to_string(work.join("test/mydir/hello.txt")).unwrap(),
        "hello\n"
    );
    assert!(work.join("test/mydir/sub/deep.txt").exists());
}

#[test]
fn roundtrip_tar_gz() {
    if !has_tool("tar") {
        return;
    }
    let dir = temp_dir("roundtrip-tar-gz");
    let src = create_source_dir(&dir);
    let archive = dir.join("test.tar.gz");
    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    let output = pack().arg(&src).arg(&archive).output().unwrap();
    assert!(
        output.status.success(),
        "pack: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = unpack().arg(&archive).current_dir(&work).output().unwrap();
    assert!(
        output.status.success(),
        "unpack: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(work.join("test/mydir/hello.txt").exists());
    assert_eq!(
        fs::read_to_string(work.join("test/mydir/hello.txt")).unwrap(),
        "hello\n"
    );
}

#[test]
fn roundtrip_gz() {
    if !has_tool("gzip") || !has_tool("gunzip") {
        return;
    }
    let dir = temp_dir("roundtrip-gz");
    let src = create_source_file(&dir);
    let archive = dir.join("data.txt.gz");
    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    let output = pack().arg(&src).arg(&archive).output().unwrap();
    assert!(
        output.status.success(),
        "pack: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = unpack().arg(&archive).current_dir(&work).output().unwrap();
    assert!(
        output.status.success(),
        "unpack: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        fs::read_to_string(work.join("data.txt")).unwrap(),
        "some data\n"
    );
}

#[test]
fn roundtrip_lz4() {
    if !has_tool("lz4") {
        return;
    }
    let dir = temp_dir("roundtrip-lz4");
    let src = create_source_file(&dir);
    let archive = dir.join("data.txt.lz4");
    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    let output = pack().arg(&src).arg(&archive).output().unwrap();
    assert!(
        output.status.success(),
        "pack: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = unpack().arg(&archive).current_dir(&work).output().unwrap();
    assert!(
        output.status.success(),
        "unpack: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        fs::read_to_string(work.join("data.txt")).unwrap(),
        "some data\n"
    );
}

#[test]
fn roundtrip_tar_lz4() {
    if !has_tool("tar") || !has_tool("lz4") {
        return;
    }
    let dir = temp_dir("roundtrip-tar-lz4");
    let src = create_source_dir(&dir);
    let archive = dir.join("test.tar.lz4");
    let work = dir.join("work");
    fs::create_dir_all(&work).unwrap();

    let output = pack().arg(&src).arg(&archive).output().unwrap();
    assert!(
        output.status.success(),
        "pack: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = unpack().arg(&archive).current_dir(&work).output().unwrap();
    assert!(
        output.status.success(),
        "unpack: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(work.join("test/mydir/hello.txt").exists());
    assert_eq!(
        fs::read_to_string(work.join("test/mydir/hello.txt")).unwrap(),
        "hello\n"
    );
    assert!(work.join("test/mydir/sub/deep.txt").exists());
}

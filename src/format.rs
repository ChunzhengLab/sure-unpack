use std::path::Path;

use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    TarZst,
    Zip,
    SevenZ,
    Rar,
    Iso,
    Gz,
    Bz2,
    Xz,
    Zst,
}

impl ArchiveFormat {
    /// All formats, ordered so compound extensions (.tar.gz) come before simple ones (.gz).
    const ALL: &[ArchiveFormat] = &[
        ArchiveFormat::TarGz,
        ArchiveFormat::TarBz2,
        ArchiveFormat::TarXz,
        ArchiveFormat::TarZst,
        ArchiveFormat::Tar,
        ArchiveFormat::Zip,
        ArchiveFormat::SevenZ,
        ArchiveFormat::Rar,
        ArchiveFormat::Iso,
        ArchiveFormat::Gz,
        ArchiveFormat::Bz2,
        ArchiveFormat::Xz,
        ArchiveFormat::Zst,
    ];

    /// The canonical extension(s) for this format. Single source of truth.
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            ArchiveFormat::Tar => &[".tar"],
            ArchiveFormat::TarGz => &[".tar.gz", ".tgz"],
            ArchiveFormat::TarBz2 => &[".tar.bz2", ".tbz2"],
            ArchiveFormat::TarXz => &[".tar.xz", ".txz"],
            ArchiveFormat::TarZst => &[".tar.zst"],
            ArchiveFormat::Zip => &[".zip"],
            ArchiveFormat::SevenZ => &[".7z"],
            ArchiveFormat::Rar => &[".rar"],
            ArchiveFormat::Iso => &[".iso"],
            ArchiveFormat::Gz => &[".gz"],
            ArchiveFormat::Bz2 => &[".bz2"],
            ArchiveFormat::Xz => &[".xz"],
            ArchiveFormat::Zst => &[".zst"],
        }
    }

    /// Tar compression flags for this format. Single source of truth
    /// shared by both pack and unpack tar backends.
    pub fn tar_compression_flags(&self) -> &'static [&'static str] {
        match self {
            ArchiveFormat::TarGz => &["-z"],
            ArchiveFormat::TarBz2 => &["-j"],
            ArchiveFormat::TarXz => &["-J"],
            ArchiveFormat::TarZst => &["--zstd"],
            ArchiveFormat::Tar => &[],
            _ => &[],
        }
    }

    /// Whether this format is a multi-file container (vs single-file compression).
    pub fn is_multi_file(&self) -> bool {
        matches!(
            self,
            ArchiveFormat::Tar
                | ArchiveFormat::TarGz
                | ArchiveFormat::TarBz2
                | ArchiveFormat::TarXz
                | ArchiveFormat::TarZst
                | ArchiveFormat::Zip
                | ArchiveFormat::SevenZ
                | ArchiveFormat::Rar
                | ArchiveFormat::Iso
        )
    }
}

/// Detect archive format from filename extension (case-insensitive).
/// Compound extensions (.tar.gz) are checked before simple ones (.gz)
/// because `ALL` is ordered that way.
pub fn detect(path: &Path) -> Result<ArchiveFormat, Error> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::UnknownFormat(path.to_path_buf()))?;

    let lower = name.to_ascii_lowercase();

    for &fmt in ArchiveFormat::ALL {
        for ext in fmt.extensions() {
            if lower.ends_with(ext) {
                return Ok(fmt);
            }
        }
    }

    Err(Error::UnknownFormat(path.to_path_buf()))
}

/// Parse a format name from --format flag (e.g. "tar.gz", "zip", "7z").
pub fn from_name(name: &str) -> Option<ArchiveFormat> {
    let lower = name.to_ascii_lowercase();
    // Try matching against extensions (with or without leading dot)
    let with_dot = if lower.starts_with('.') {
        lower.clone()
    } else {
        format!(".{lower}")
    };
    for &fmt in ArchiveFormat::ALL {
        for ext in fmt.extensions() {
            if *ext == with_dot {
                return Some(fmt);
            }
        }
    }
    None
}

/// Sniff the outer archive format from file header magic bytes.
/// Pure file I/O — no external tool calls, no subprocess spawning.
/// For gz/bz2/xz/zst, returns the outer layer only (Gz, not TarGz).
/// Use `probe_tar_inside()` separately to check for tar within compression.
pub fn sniff_outer(path: &Path) -> Option<ArchiveFormat> {
    let mut buf = [0u8; 262];
    let n = {
        use std::io::Read;
        let mut f = std::fs::File::open(path).ok()?;
        f.read(&mut buf).ok()?
    };
    if n < 2 {
        return None;
    }

    if n >= 7 && buf[..7] == *b"Rar!\x1a\x07\x01" {
        return Some(ArchiveFormat::Rar);
    }
    if n >= 6 && buf[..6] == *b"Rar!\x1a\x07" {
        return Some(ArchiveFormat::Rar);
    }
    if n >= 6 && buf[..6] == [0xFD, b'7', b'z', b'X', b'Z', 0x00] {
        return Some(ArchiveFormat::Xz);
    }
    if n >= 6 && buf[..6] == [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C] {
        return Some(ArchiveFormat::SevenZ);
    }
    if n >= 4 && buf[..4] == *b"PK\x03\x04" {
        return Some(ArchiveFormat::Zip);
    }
    if n >= 4 && buf[..4] == [0x28, 0xB5, 0x2F, 0xFD] {
        return Some(ArchiveFormat::Zst);
    }
    if n >= 3 && buf[..3] == *b"BZh" {
        return Some(ArchiveFormat::Bz2);
    }
    if n >= 2 && buf[..2] == [0x1F, 0x8B] {
        return Some(ArchiveFormat::Gz);
    }
    if n >= 262 && &buf[257..262] == b"ustar" {
        return Some(ArchiveFormat::Tar);
    }
    if let Ok(true) = check_iso(path) {
        return Some(ArchiveFormat::Iso);
    }
    None
}

/// Decompress the first 262 bytes with the given tool and check for tar magic.
/// Returns false if the tool is not available or decompression fails.
pub fn probe_tar_inside(path: &Path, tool: &str, args: &[&str]) -> bool {
    use std::io::Read;
    use std::process::{Command, Stdio};

    let child = Command::new(tool)
        .args(args)
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(_) => return false,
    };

    let mut buf = [0u8; 262];
    let mut filled = 0;
    if let Some(stdout) = child.stdout.as_mut() {
        while filled < buf.len() {
            match stdout.read(&mut buf[filled..]) {
                Ok(0) => break,   // EOF
                Ok(n) => filled += n,
                Err(_) => break,
            }
        }
    }

    let _ = child.kill();
    let _ = child.wait();

    filled >= 262 && &buf[257..262] == b"ustar"
}

fn check_iso(path: &Path) -> std::io::Result<bool> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(path)?;
    f.seek(SeekFrom::Start(32769))?;
    let mut magic = [0u8; 5];
    f.read_exact(&mut magic)?;
    Ok(&magic == b"CD001")
}

/// Derive the "stem" used as default output directory/filename.
///
/// - `foo.tar.gz` → `foo`
/// - `archive.zip` → `archive`
/// - `file.txt.gz` → `file.txt` (single-file decompression)
pub fn archive_stem(path: &Path, format: ArchiveFormat) -> String {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("output");

    let lower = name.to_ascii_lowercase();
    for ext in format.extensions() {
        if lower.ends_with(ext) {
            return name[..name.len() - ext.len()].to_string();
        }
    }
    // Extension didn't match (e.g. --format gz on mystery.bin).
    // For single-file formats, returning the original name would make
    // dest == source. Append ".out" to avoid self-targeting.
    if format.is_multi_file() {
        name.to_string()
    } else {
        format!("{name}.out")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detect_tar_variants() {
        let cases = [
            ("foo.tar", ArchiveFormat::Tar),
            ("foo.tar.gz", ArchiveFormat::TarGz),
            ("foo.tgz", ArchiveFormat::TarGz),
            ("foo.tar.bz2", ArchiveFormat::TarBz2),
            ("foo.tbz2", ArchiveFormat::TarBz2),
            ("foo.tar.xz", ArchiveFormat::TarXz),
            ("foo.txz", ArchiveFormat::TarXz),
            ("foo.tar.zst", ArchiveFormat::TarZst),
        ];
        for (name, expected) in cases {
            let p = PathBuf::from(name);
            assert_eq!(detect(&p).unwrap(), expected, "failed for {name}");
        }
    }

    #[test]
    fn detect_other_formats() {
        let cases = [
            ("archive.zip", ArchiveFormat::Zip),
            ("data.7z", ArchiveFormat::SevenZ),
            ("archive.rar", ArchiveFormat::Rar),
            ("disc.iso", ArchiveFormat::Iso),
            ("file.gz", ArchiveFormat::Gz),
            ("file.bz2", ArchiveFormat::Bz2),
            ("file.xz", ArchiveFormat::Xz),
            ("file.zst", ArchiveFormat::Zst),
        ];
        for (name, expected) in cases {
            let p = PathBuf::from(name);
            assert_eq!(detect(&p).unwrap(), expected, "failed for {name}");
        }
    }

    #[test]
    fn detect_case_insensitive() {
        let cases = [
            "FOO.TAR.GZ",
            "Foo.Tar.Bz2",
            "ARCHIVE.ZIP",
            "Data.7Z",
            "file.TGZ",
        ];
        for name in cases {
            let p = PathBuf::from(name);
            assert!(detect(&p).is_ok(), "failed for {name}");
        }
    }

    #[test]
    fn detect_unknown() {
        let p = PathBuf::from("readme.txt");
        assert!(detect(&p).is_err());
        let p = PathBuf::from("noext");
        assert!(detect(&p).is_err());
    }

    #[test]
    fn stem_multi_file() {
        let cases = [
            ("foo.tar.gz", ArchiveFormat::TarGz, "foo"),
            ("foo.tgz", ArchiveFormat::TarGz, "foo"),
            ("bar.tar.bz2", ArchiveFormat::TarBz2, "bar"),
            ("bar.tbz2", ArchiveFormat::TarBz2, "bar"),
            ("baz.tar.xz", ArchiveFormat::TarXz, "baz"),
            ("baz.txz", ArchiveFormat::TarXz, "baz"),
            ("qux.tar.zst", ArchiveFormat::TarZst, "qux"),
            ("data.tar", ArchiveFormat::Tar, "data"),
            ("archive.zip", ArchiveFormat::Zip, "archive"),
            ("stuff.7z", ArchiveFormat::SevenZ, "stuff"),
        ];
        for (name, fmt, expected) in cases {
            let p = PathBuf::from(name);
            assert_eq!(archive_stem(&p, fmt), expected, "failed for {name}");
        }
    }

    #[test]
    fn stem_single_file() {
        let cases = [
            ("file.txt.gz", ArchiveFormat::Gz, "file.txt"),
            ("data.csv.bz2", ArchiveFormat::Bz2, "data.csv"),
            ("log.txt.xz", ArchiveFormat::Xz, "log.txt"),
            ("dump.sql.zst", ArchiveFormat::Zst, "dump.sql"),
        ];
        for (name, fmt, expected) in cases {
            let p = PathBuf::from(name);
            assert_eq!(archive_stem(&p, fmt), expected, "failed for {name}");
        }
    }

    #[test]
    fn stem_mismatched_extension_single_file() {
        // When extension doesn't match format, single-file must not return
        // the original name (that would make dest == source).
        let p = PathBuf::from("mystery.bin");
        assert_eq!(archive_stem(&p, ArchiveFormat::Gz), "mystery.bin.out");
        assert_eq!(archive_stem(&p, ArchiveFormat::Bz2), "mystery.bin.out");
        assert_eq!(archive_stem(&p, ArchiveFormat::Xz), "mystery.bin.out");
        assert_eq!(archive_stem(&p, ArchiveFormat::Zst), "mystery.bin.out");
    }

    #[test]
    fn stem_mismatched_extension_multi_file() {
        // Multi-file formats can safely use the original name as a directory.
        let p = PathBuf::from("mystery.bin");
        assert_eq!(archive_stem(&p, ArchiveFormat::Zip), "mystery.bin");
        assert_eq!(archive_stem(&p, ArchiveFormat::TarGz), "mystery.bin");
    }

    /// If a new variant is added to ArchiveFormat but not to ALL,
    /// this exhaustive match forces a compile error.
    #[test]
    fn all_covers_every_variant() {
        for &fmt in ArchiveFormat::ALL {
            match fmt {
                ArchiveFormat::Tar
                | ArchiveFormat::TarGz
                | ArchiveFormat::TarBz2
                | ArchiveFormat::TarXz
                | ArchiveFormat::TarZst
                | ArchiveFormat::Zip
                | ArchiveFormat::SevenZ
                | ArchiveFormat::Rar
                | ArchiveFormat::Iso
                | ArchiveFormat::Gz
                | ArchiveFormat::Bz2
                | ArchiveFormat::Xz
                | ArchiveFormat::Zst => {}
            }
        }
        assert_eq!(ArchiveFormat::ALL.len(), 13);
    }

    #[test]
    fn is_multi_file_check() {
        assert!(ArchiveFormat::Tar.is_multi_file());
        assert!(ArchiveFormat::TarGz.is_multi_file());
        assert!(ArchiveFormat::Zip.is_multi_file());
        assert!(ArchiveFormat::SevenZ.is_multi_file());
        assert!(!ArchiveFormat::Gz.is_multi_file());
        assert!(!ArchiveFormat::Bz2.is_multi_file());
        assert!(!ArchiveFormat::Xz.is_multi_file());
        assert!(!ArchiveFormat::Zst.is_multi_file());
    }
}

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
    name.to_string()
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

mod backend;
mod cli;
mod error;
mod format;
mod safety;

use std::path::Path;

use backend::Backend;
use cli::Command;
use error::Error;
use format::ArchiveFormat;

fn run() -> Result<(), Error> {
    let cmd = match cli::parse(std::env::args())? {
        Some(cmd) => cmd,
        None => return Ok(()),
    };

    match cmd {
        Command::List(opts) => do_list(opts),
        Command::Extract(opts) => do_extract(opts),
        Command::Sniff(paths) => do_sniff(&paths),
    }
}

fn do_list(opts: cli::ListOpts) -> Result<(), Error> {
    let archive = &opts.archive;
    if !archive.exists() {
        return Err(Error::FileNotFound(archive.to_path_buf()));
    }

    let fmt = resolve_format(archive, &opts.format_override)?;
    let backend = Backend::from_format(fmt);
    let entries = backend.list(archive, fmt)?;

    let warnings = safety::check_entries(&entries);
    safety::print_warnings(&warnings);

    for entry in &entries {
        println!("{entry}");
    }
    eprintln!("{} entries", entries.len());

    Ok(())
}

fn do_extract(opts: cli::ExtractOpts) -> Result<(), Error> {
    let archive = &opts.archive;
    if !archive.exists() {
        return Err(Error::FileNotFound(archive.to_path_buf()));
    }

    let fmt = resolve_format(archive, &opts.format_override)?;
    let backend = Backend::from_format(fmt);

    if opts.strip_components > 0 && !backend.supports_strip_components() {
        return Err(Error::Usage(format!(
            "--strip-components is not supported for {} files",
            fmt.extensions()[0]
        )));
    }

    let tool_ok = backend.ensure_tool(fmt).is_ok();
    if !opts.dry_run && !tool_ok {
        backend.ensure_tool(fmt)?;
    }

    let dest = resolve_dest(&opts, archive, fmt);

    let mut conflicts = Vec::new();
    if fmt.is_multi_file() && tool_ok {
        let entries = backend.list(archive, fmt)?;
        let warnings = safety::check_entries(&entries);
        safety::print_warnings(&warnings);

        if !opts.overwrite && dest.exists() {
            conflicts = find_member_conflicts(&entries, &dest, opts.strip_components);
            if !opts.dry_run
                && let Some(first) = conflicts.first()
            {
                return Err(Error::DestinationExists(first.clone()));
            }
        }
    }

    let is_auto_subdir =
        fmt.is_multi_file() && opts.dest.is_none() && opts.into.is_none() && !opts.here;
    if !opts.overwrite && is_auto_subdir && dest.exists() {
        if opts.dry_run {
            conflicts.push(dest.clone());
        } else {
            return Err(Error::DestinationExists(dest));
        }
    }

    if !fmt.is_multi_file() && !opts.overwrite && dest.exists() {
        if opts.dry_run {
            conflicts.push(dest.clone());
        } else {
            return Err(Error::DestinationExists(dest));
        }
    }

    if opts.dry_run {
        println!("archive:   {}", archive.display());
        println!("format:    {}", fmt.extensions()[0]);
        println!("backend:   {}", backend.tool_name());
        println!("tool:      {}", if tool_ok { "found" } else { "NOT FOUND" });
        println!("dest:      {}", dest.display());
        if opts.strip_components > 0 {
            println!("strip:     {} components", opts.strip_components);
        }
        if conflicts.is_empty() {
            println!("conflicts: none");
        } else {
            println!("conflicts: {}", conflicts.len());
            for c in &conflicts {
                println!("  {}", c.display());
            }
        }
        if !tool_ok || !conflicts.is_empty() {
            std::process::exit(1);
        }
        return Ok(());
    }

    if fmt.is_multi_file() {
        std::fs::create_dir_all(&dest)?;
    } else if let Some(parent) = dest.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }

    backend.extract(archive, &dest, fmt, opts.strip_components, opts.overwrite, opts.verbose)?;

    eprintln!("extracted to {}", dest.display());
    Ok(())
}

fn do_sniff(paths: &[std::path::PathBuf]) -> Result<(), Error> {
    let mut any_fail = false;
    for path in paths {
        let name = path.display();
        if !path.exists() {
            println!("{name}: no such file");
            any_fail = true;
        } else if let Some(fmt) = sniff_full(path) {
            println!("{name}: {}", fmt.extensions()[0]);
        } else {
            println!("{name}: unknown");
            any_fail = true;
        }
    }
    if any_fail {
        std::process::exit(1);
    }
    Ok(())
}

// --- Format resolution ---

/// The decision chain:
/// 1. --format → use it
/// 2. sniff_outer (15us) → zip/7z/rar/tar/iso → use it
/// 3. sniff_outer → gz/bz2/xz/zst:
///    a. extension is matching .tar.* → use tar variant (skip probe)
///    b. else → probe_tar_inside (1.5ms)
/// 4. sniff_outer → None → fall back to extension
/// 5. nothing → error
fn resolve_format(
    archive: &Path,
    format_override: &Option<String>,
) -> Result<ArchiveFormat, Error> {
    if let Some(name) = format_override {
        return format::from_name(name)
            .ok_or_else(|| Error::Usage(format!("unknown format: {name}")));
    }

    let sniffed = format::sniff_outer(archive);
    let ext = format::detect(archive).ok();

    match sniffed {
        // Definitive formats — sniff is ground truth
        Some(
            fmt @ (ArchiveFormat::Zip
            | ArchiveFormat::SevenZ
            | ArchiveFormat::Rar
            | ArchiveFormat::Tar
            | ArchiveFormat::Iso),
        ) => Ok(fmt),

        // Stream compression — might be tar inside
        Some(outer @ (ArchiveFormat::Gz | ArchiveFormat::Bz2 | ArchiveFormat::Xz | ArchiveFormat::Zst)) => {
            // If extension already says tar.*, trust it (skip 1.5ms probe)
            if let Some(ext_fmt) = ext
                && is_tar_variant_of(ext_fmt, outer)
            {
                return Ok(ext_fmt);
            }
            // Otherwise probe to check for tar inside
            Ok(upgrade_to_tar(archive, outer))
        }

        // sniff can't tell (e.g. old tar without ustar, or truly unknown)
        _ => ext.ok_or_else(|| Error::UnknownFormat(archive.to_path_buf())),
    }
}

/// Full sniff for the `sniff` subcommand: sniff_outer + probe, no extension.
fn sniff_full(path: &Path) -> Option<ArchiveFormat> {
    let outer = format::sniff_outer(path)?;
    match outer {
        ArchiveFormat::Gz | ArchiveFormat::Bz2 | ArchiveFormat::Xz | ArchiveFormat::Zst => {
            Some(upgrade_to_tar(path, outer))
        }
        _ => Some(outer),
    }
}

/// Check if ext_fmt is the tar variant of the given outer compression.
fn is_tar_variant_of(ext: ArchiveFormat, outer: ArchiveFormat) -> bool {
    matches!(
        (ext, outer),
        (ArchiveFormat::TarGz, ArchiveFormat::Gz)
            | (ArchiveFormat::TarBz2, ArchiveFormat::Bz2)
            | (ArchiveFormat::TarXz, ArchiveFormat::Xz)
            | (ArchiveFormat::TarZst, ArchiveFormat::Zst)
    )
}

/// Try to upgrade a stream compression format to its tar variant via probe.
fn upgrade_to_tar(path: &Path, outer: ArchiveFormat) -> ArchiveFormat {
    let (tool, args): (&str, &[&str]) = match outer {
        ArchiveFormat::Gz => ("gunzip", &["-c"]),
        ArchiveFormat::Bz2 => ("bunzip2", &["-c"]),
        ArchiveFormat::Xz => ("xz", &["-dc"]),
        ArchiveFormat::Zst => ("zstd", &["-dc", "--no-progress"]),
        _ => return outer,
    };

    if format::probe_tar_inside(path, tool, args) {
        match outer {
            ArchiveFormat::Gz => ArchiveFormat::TarGz,
            ArchiveFormat::Bz2 => ArchiveFormat::TarBz2,
            ArchiveFormat::Xz => ArchiveFormat::TarXz,
            ArchiveFormat::Zst => ArchiveFormat::TarZst,
            _ => outer,
        }
    } else {
        outer
    }
}

// --- Helpers ---

fn find_member_conflicts(
    entries: &[String],
    dest: &Path,
    strip_components: u32,
) -> Vec<std::path::PathBuf> {
    let strip = strip_components as usize;
    let mut conflicts = Vec::new();
    for entry in entries {
        let stripped = if strip > 0 {
            let components: Vec<&str> = entry.split('/').collect();
            if components.len() <= strip {
                continue;
            }
            components[strip..].join("/")
        } else {
            entry.to_string()
        };
        if stripped.is_empty() {
            continue;
        }
        let target = dest.join(&stripped);
        if target.is_file() {
            conflicts.push(target);
        }
    }
    conflicts
}

fn resolve_dest(
    opts: &cli::ExtractOpts,
    archive: &Path,
    fmt: ArchiveFormat,
) -> std::path::PathBuf {
    if let Some(ref d) = opts.dest {
        return d.clone();
    }
    if let Some(ref d) = opts.into {
        return d.join(format::archive_stem(archive, fmt));
    }
    if opts.here && fmt.is_multi_file() {
        return std::path::PathBuf::from(".");
    }
    std::path::PathBuf::from(format::archive_stem(archive, fmt))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("unpack: {e}");
        std::process::exit(e.exit_code());
    }
}

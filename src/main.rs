mod backend;
mod cli;
mod error;
mod format;
mod safety;

use std::path::Path;

use backend::Backend;
use cli::Command;
use error::Error;

fn run() -> Result<(), Error> {
    let cmd = match cli::parse(std::env::args())? {
        Some(cmd) => cmd,
        None => return Ok(()), // --help or --version was printed
    };

    match cmd {
        Command::List(opts) => do_list(&opts.archive),
        Command::Extract(opts) => do_extract(opts),
    }
}

fn do_list(archive: &Path) -> Result<(), Error> {
    if !archive.exists() {
        return Err(Error::FileNotFound(archive.to_path_buf()));
    }

    let fmt = format::detect(archive)?;
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

    let fmt = format::detect(archive)?;
    let backend = Backend::from_format(fmt);

    if opts.strip_components > 0 && !backend.supports_strip_components() {
        return Err(Error::Usage(format!(
            "--strip-components is not supported for {} files",
            fmt.extensions()[0]
        )));
    }

    let dest = resolve_dest(&opts, archive, fmt);

    // For multi-file archives: list for safety scan + pre-flight overwrite check
    if fmt.is_multi_file() {
        let entries = backend.list(archive, fmt)?;
        let warnings = safety::check_entries(&entries);
        safety::print_warnings(&warnings);

        // Pre-flight: if dest exists and !overwrite, check for member file conflicts.
        // This replaces fragile backend-level flags (tar -k) with an explicit check.
        if !opts.overwrite
            && dest.exists()
            && let Some(conflict) = find_member_conflict(&entries, &dest, opts.strip_components)
        {
            return Err(Error::DestinationExists(conflict));
        }
    }

    // For auto-created subdirectories: refuse if the directory already exists
    let is_auto_subdir = opts.dest.is_none() && opts.into.is_none() && !opts.here;
    if !opts.overwrite && is_auto_subdir && dest.exists() {
        return Err(Error::DestinationExists(dest));
    }

    // Single-file: refuse if output file already exists
    if !fmt.is_multi_file() && !opts.overwrite && dest.exists() {
        return Err(Error::DestinationExists(dest));
    }

    if fmt.is_multi_file() {
        std::fs::create_dir_all(&dest)?;
    } else if let Some(parent) = dest.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }

    backend.extract(
        archive,
        &dest,
        fmt,
        opts.strip_components,
        opts.overwrite,
        opts.verbose,
    )?;

    eprintln!("extracted to {}", dest.display());
    Ok(())
}

/// Check if any archive member would overwrite an existing file under dest.
/// Only flags files, not directories — a directory existing at the target
/// is expected and not a conflict.
fn find_member_conflict(
    entries: &[String],
    dest: &Path,
    strip_components: u32,
) -> Option<std::path::PathBuf> {
    let strip = strip_components as usize;
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
            return Some(target);
        }
    }
    None
}

fn resolve_dest(
    opts: &cli::ExtractOpts,
    archive: &Path,
    fmt: format::ArchiveFormat,
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
        eprintln!("sure-unpack: {e}");
        std::process::exit(e.exit_code());
    }
}

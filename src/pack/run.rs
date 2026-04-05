use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::format::{self, ArchiveFormat};
use crate::tool;

use super::backend;
use super::cli::{self, Command};

pub fn run() -> Result<(), Error> {
    let cmd = match cli::parse(std::env::args())? {
        Some(cmd) => cmd,
        None => return Ok(()),
    };

    match cmd {
        Command::Pack(opts) => do_pack(opts),
    }
}

fn do_pack(opts: cli::PackOpts) -> Result<(), Error> {
    let source = &opts.source;
    if !source.exists() {
        return Err(Error::FileNotFound(source.to_path_buf()));
    }

    let fmt = resolve_output_format(&opts)?;

    // Directory + stream compression → error
    if source.is_dir() && !fmt.is_multi_file() {
        return Err(Error::Usage(format!(
            "directories are not supported for {} output; use {}",
            fmt.extensions()[0],
            suggest_tar_variant(fmt),
        )));
    }

    let output = resolve_output_path(&opts, source, fmt);

    // Check output exists
    if !opts.overwrite && output.exists() {
        return Err(Error::DestinationExists(output));
    }

    let (tool_name, candidates) = tool::for_format_pack(fmt);
    let tool_result = tool::ensure(tool_name, candidates, fmt);

    if opts.dry_run {
        let tool_ok = tool_result.is_ok();
        println!("source:    {}", source.display());
        println!("output:    {}", output.display());
        println!("format:    {}", fmt.extensions()[0]);
        println!("backend:   {tool_name}");
        println!("tool:      {}", if tool_ok { "found" } else { "NOT FOUND" });
        println!("overwrite: {}", if opts.overwrite { "yes" } else { "no" });
        if !tool_ok {
            return Err(tool_result.unwrap_err());
        }
        return Ok(());
    }

    let tool_path = tool_result?;
    backend::pack(&tool_path, tool_name, source, &output, fmt, opts.verbose)?;

    eprintln!("packed to {}", output.display());
    Ok(())
}

/// Formats that are read-only (cannot be created by pack).
fn is_pack_supported(fmt: ArchiveFormat) -> bool {
    !matches!(fmt, ArchiveFormat::Rar | ArchiveFormat::Iso)
}

/// Resolve output format from --format and/or output extension.
fn resolve_output_format(opts: &cli::PackOpts) -> Result<ArchiveFormat, Error> {
    // Validate --format first: reject unknown names immediately
    let from_flag = match &opts.format_override {
        Some(name) => {
            let fmt = format::from_name(name)
                .ok_or_else(|| Error::Usage(format!("unknown format: {name}")))?;
            if !is_pack_supported(fmt) {
                return Err(Error::Usage(format!(
                    "packing {} is not supported",
                    fmt.extensions()[0]
                )));
            }
            Some(fmt)
        }
        None => None,
    };

    let from_ext = opts
        .output
        .as_ref()
        .and_then(|p| format::detect(p).ok());

    match (from_flag, from_ext) {
        (Some(flag_fmt), Some(ext_fmt)) if flag_fmt != ext_fmt => {
            Err(Error::Usage(format!(
                "--format {} conflicts with output extension {}",
                flag_fmt.extensions()[0],
                ext_fmt.extensions()[0],
            )))
        }
        (Some(fmt), _) => Ok(fmt),
        (_, Some(fmt)) => {
            if !is_pack_supported(fmt) {
                return Err(Error::Usage(format!(
                    "packing {} is not supported",
                    fmt.extensions()[0]
                )));
            }
            Ok(fmt)
        }
        (None, None) => Ok(ArchiveFormat::Zip),
    }
}

fn resolve_output_path(
    opts: &cli::PackOpts,
    source: &Path,
    fmt: ArchiveFormat,
) -> PathBuf {
    if let Some(ref out) = opts.output {
        return out.clone();
    }
    // Derive from source name + format extension
    let stem = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("output");
    let ext = fmt.extensions()[0]; // e.g. ".zip", ".tar.gz"
    PathBuf::from(format!("{stem}{ext}"))
}

fn suggest_tar_variant(fmt: ArchiveFormat) -> &'static str {
    match fmt {
        ArchiveFormat::Gz => ".tar.gz",
        ArchiveFormat::Bz2 => ".tar.bz2",
        ArchiveFormat::Xz => ".tar.xz",
        ArchiveFormat::Zst => ".tar.zst",
        _ => ".tar.gz",
    }
}

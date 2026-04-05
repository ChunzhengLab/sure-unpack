use std::path::PathBuf;

use crate::error::Error;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const USAGE: &str = "\
pack - create archives from files and directories

USAGE:
    pack [OPTIONS] <SRC> [OUT]

OPTIONS:
    -f, --format <FMT>   Output format (e.g. zip, tar.gz, 7z)
        --dry-run        Show what would happen without packing
    -o, --overwrite      Allow overwriting existing output file
    -v, --verbose        Show detailed output
        --help           Show this help
        --version        Show version";

#[derive(Debug)]
pub enum Command {
    Pack(PackOpts),
}

#[derive(Debug)]
pub struct PackOpts {
    pub source: PathBuf,
    pub output: Option<PathBuf>,
    pub format_override: Option<String>,
    pub dry_run: bool,
    pub overwrite: bool,
    pub verbose: bool,
}

pub fn parse<I>(args: I) -> Result<Option<Command>, Error>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let _program = args.next();

    let mut format_override: Option<String> = None;
    let mut dry_run = false;
    let mut overwrite = false;
    let mut verbose = false;
    let mut positionals: Vec<String> = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("{USAGE}");
                return Ok(None);
            }
            "--version" | "-V" => {
                println!("pack {VERSION}");
                return Ok(None);
            }
            "-f" | "--format" => {
                let val = args
                    .next()
                    .ok_or_else(|| Error::Usage("--format requires a format name".into()))?;
                format_override = Some(val);
            }
            "--dry-run" => {
                dry_run = true;
            }
            "-o" | "--overwrite" => {
                overwrite = true;
            }
            "-v" | "--verbose" => {
                verbose = true;
            }
            s if s.starts_with('-') => {
                return Err(Error::Usage(format!("unknown option: {s}")));
            }
            _ => {
                positionals.push(arg);
            }
        }
    }

    if positionals.is_empty() {
        return Err(Error::Usage("missing source argument".into()));
    }

    let source = PathBuf::from(&positionals[0]);
    let output = positionals.get(1).map(PathBuf::from);

    Ok(Some(Command::Pack(PackOpts {
        source,
        output,
        format_override,
        dry_run,
        overwrite,
        verbose,
    })))
}

use std::path::PathBuf;

use crate::error::Error;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const USAGE: &str = "\
unpack - one command to unpack them all

USAGE:
    unpack [OPTIONS] <ARCHIVE> [DEST]
    unpack list <ARCHIVE>

COMMANDS:
    list            Preview archive contents without extracting
    (default)       Extract the archive

OPTIONS:
    -C, --into <DIR>         Extract into specified directory
        --here               Extract into current directory (no subdirectory)
    -o, --overwrite          Allow overwriting existing files
        --strip-components N Strip N leading path components (tar only)
    -v, --verbose            Show detailed output
    -l, --list               Same as 'unpack list'
        --help               Show this help
        --version            Show version";

#[derive(Debug)]
pub enum Command {
    Extract(ExtractOpts),
    List(ListOpts),
}

#[derive(Debug)]
pub struct ExtractOpts {
    pub archive: PathBuf,
    pub dest: Option<PathBuf>,
    pub into: Option<PathBuf>,
    pub here: bool,
    pub overwrite: bool,
    pub strip_components: u32,
    pub verbose: bool,
}

#[derive(Debug)]
pub struct ListOpts {
    pub archive: PathBuf,
}

/// Parse CLI arguments. Returns Ok(None) if --help or --version was printed.
pub fn parse<I>(args: I) -> Result<Option<Command>, Error>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let _program = args.next(); // skip argv[0]

    let mut is_list = false;
    let mut into: Option<PathBuf> = None;
    let mut here = false;
    let mut overwrite = false;
    let mut strip_components: u32 = 0;
    let mut verbose = false;
    let mut positionals: Vec<String> = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("{USAGE}");
                return Ok(None);
            }
            "--version" | "-V" => {
                println!("unpack {VERSION}");
                return Ok(None);
            }
            "list" if positionals.is_empty() && !is_list => {
                is_list = true;
            }
            "-l" | "--list" => {
                is_list = true;
            }
            "-C" | "--into" => {
                let val = args.next().ok_or_else(|| {
                    Error::Usage(format!("{arg} requires a directory argument"))
                })?;
                into = Some(PathBuf::from(val));
            }
            "--here" => {
                here = true;
            }
            "-o" | "--overwrite" => {
                overwrite = true;
            }
            "--strip-components" => {
                let val = args.next().ok_or_else(|| {
                    Error::Usage("--strip-components requires a number".into())
                })?;
                strip_components = val.parse::<u32>().map_err(|_| {
                    Error::Usage(format!("--strip-components: invalid number '{val}'"))
                })?;
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
        return Err(Error::Usage("missing archive file argument".into()));
    }

    if is_list {
        return Ok(Some(Command::List(ListOpts {
            archive: PathBuf::from(&positionals[0]),
        })));
    }

    let archive = PathBuf::from(&positionals[0]);
    let dest = positionals.get(1).map(PathBuf::from);

    Ok(Some(Command::Extract(ExtractOpts {
        archive,
        dest,
        into,
        here,
        overwrite,
        strip_components,
        verbose,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn bare_extract() {
        let cmd = parse(args("unpack foo.tar.gz")).unwrap().unwrap();
        match cmd {
            Command::Extract(opts) => {
                assert_eq!(opts.archive, PathBuf::from("foo.tar.gz"));
                assert!(opts.dest.is_none());
                assert!(opts.into.is_none());
                assert!(!opts.here);
                assert!(!opts.overwrite);
                assert_eq!(opts.strip_components, 0);
                assert!(!opts.verbose);
            }
            _ => panic!("expected Extract"),
        }
    }

    #[test]
    fn extract_with_dest() {
        let cmd = parse(args("unpack foo.tar.gz /tmp/out")).unwrap().unwrap();
        match cmd {
            Command::Extract(opts) => {
                assert_eq!(opts.archive, PathBuf::from("foo.tar.gz"));
                assert_eq!(opts.dest.unwrap(), PathBuf::from("/tmp/out"));
            }
            _ => panic!("expected Extract"),
        }
    }

    #[test]
    fn extract_with_options() {
        let cmd = parse(args("unpack -o -v --here --strip-components 2 foo.zip"))
            .unwrap()
            .unwrap();
        match cmd {
            Command::Extract(opts) => {
                assert!(opts.overwrite);
                assert!(opts.verbose);
                assert!(opts.here);
                assert_eq!(opts.strip_components, 2);
                assert_eq!(opts.archive, PathBuf::from("foo.zip"));
            }
            _ => panic!("expected Extract"),
        }
    }

    #[test]
    fn extract_with_into() {
        let cmd = parse(args("unpack -C /tmp foo.tar.gz")).unwrap().unwrap();
        match cmd {
            Command::Extract(opts) => {
                assert_eq!(opts.into.unwrap(), PathBuf::from("/tmp"));
                assert_eq!(opts.archive, PathBuf::from("foo.tar.gz"));
            }
            _ => panic!("expected Extract"),
        }
    }

    #[test]
    fn list_subcommand() {
        let cmd = parse(args("unpack list foo.tar.gz")).unwrap().unwrap();
        match cmd {
            Command::List(opts) => {
                assert_eq!(opts.archive, PathBuf::from("foo.tar.gz"));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn list_flag() {
        let cmd = parse(args("unpack -l foo.zip")).unwrap().unwrap();
        match cmd {
            Command::List(opts) => {
                assert_eq!(opts.archive, PathBuf::from("foo.zip"));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn list_long_flag() {
        let cmd = parse(args("unpack --list archive.7z")).unwrap().unwrap();
        match cmd {
            Command::List(opts) => {
                assert_eq!(opts.archive, PathBuf::from("archive.7z"));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn help_returns_none() {
        assert!(parse(args("unpack --help")).unwrap().is_none());
        assert!(parse(args("unpack -h")).unwrap().is_none());
    }

    #[test]
    fn version_returns_none() {
        assert!(parse(args("unpack --version")).unwrap().is_none());
        assert!(parse(args("unpack -V")).unwrap().is_none());
    }

    #[test]
    fn missing_archive() {
        assert!(parse(args("unpack")).is_err());
    }

    #[test]
    fn unknown_option() {
        assert!(parse(args("unpack --foo bar.tar")).is_err());
    }

    #[test]
    fn into_missing_value() {
        assert!(parse(args("unpack -C")).is_err());
    }

    #[test]
    fn strip_components_invalid() {
        assert!(parse(args("unpack --strip-components abc foo.tar")).is_err());
    }
}

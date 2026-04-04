# sure-unpack

[![build](https://github.com/ChunzhengLab/sure-unpack/actions/workflows/rust.yml/badge.svg)](https://github.com/ChunzhengLab/sure-unpack/actions/workflows/rust.yml) [![version](https://img.shields.io/github/v/tag/ChunzhengLab/sure-unpack?label=version)](https://github.com/ChunzhengLab/sure-unpack/releases) [![license](https://img.shields.io/github/license/ChunzhengLab/sure-unpack)](LICENSE)

English | [中文](README.zh-cn.md)

`unpack` is a command-line tool written in Rust. It identifies the archive format from file headers and calls the right system tool to extract it.

No need to remember whether to use `tar`, `unzip`, or `7z`, or what `-xzf`, `-xJf`, `-d`, `-o` means for which tool. Just `unpack <file>` and it handles the rest.

## Install

```sh
brew tap ChunzhengLab/tap
brew install unpack
```

See [homebrew-tap](https://github.com/ChunzhengLab/homebrew-tap) for details.

## Usage

```
unpack [OPTIONS] <ARCHIVE> [DEST]
unpack sniff <FILE>...
unpack list <ARCHIVE>
```

The simplest usage is `unpack <file>`. Format is auto-detected from the file header first, then extension as fallback.

## Examples

```sh
unpack project.tar.gz              # → ./project/
unpack archive.zip -C /tmp         # → /tmp/archive/
unpack mystery.bin                 # auto-detect from file header
unpack sniff mystery.bin           # what format is this file?
unpack list project.tar.gz         # preview contents without extracting
unpack -l project.tar.gz           # same as above
unpack project.tar.gz --here       # extract into current directory
unpack -o project.tar.gz           # allow overwriting existing files
unpack --dry-run project.tar.gz    # show what would happen, don't extract
unpack --strip-components 1 a.tgz  # strip top-level directory (tar only)
unpack --format tar.gz mystery.bin # override all detection
```

## Options

```
-C, --into <DIR>         Extract into specified directory
    --here               Extract into current directory (no subdirectory)
-o, --overwrite          Allow overwriting existing files
    --format <FMT>       Override format detection (e.g. tar.gz, zip, 7z)
    --strip-components N Strip N leading path components (tar only)
    --dry-run            Show what would happen without extracting
-v, --verbose            Show detailed output
-l, --list               Same as 'unpack list'
    --help               Show this help
    --version            Show version
```

## Supported Formats

| Format | Backend | Notes |
|--------|---------|-------|
| `.tar` `.tar.gz` `.tgz` `.tar.bz2` `.tbz2` `.tar.xz` `.txz` `.tar.zst` | `tar` | Supports `--strip-components` |
| `.zip` | `unzip` | |
| `.7z` | `7z` / `7zz` | |
| `.rar` | `7z` / `7zz` | |
| `.iso` | `7z` / `7zz` | |
| `.gz` | `gunzip` | Single-file decompression |
| `.bz2` | `bunzip2` | Single-file decompression |
| `.xz` | `xz` | Single-file decompression |
| `.zst` | `zstd` | Single-file decompression |

`unpack` does not parse archive formats itself. It delegates to system tools already installed on your machine. If a required tool is missing, it tells you exactly what to install.

## Defaults

- **Auto subdirectory**: multi-file archives extract to `./<archive-stem>/`, not the current directory. Use `--here` to extract in place.
- **No overwrite**: refuses to extract when the target already exists. Use `-o` to override.
- **Path safety warnings**: entries containing `..` or absolute paths trigger a warning on stderr.
- **Missing tool reporting**: clear error message naming the missing tool and the format it handles.
- **Header sniffing**: `unpack` reads the file header to identify the format, even when the extension is wrong or missing. For compressed streams (`.gz`, `.bz2`, etc.), it probes inside to detect tarballs. Use `unpack sniff <file>` to see what the header says.

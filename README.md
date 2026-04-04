# sure-unpack

[![build](https://github.com/ChunzhengLab/sure-unpack/actions/workflows/rust.yml/badge.svg)](https://github.com/ChunzhengLab/sure-unpack/actions/workflows/rust.yml) [![version](https://img.shields.io/github/v/tag/ChunzhengLab/sure-unpack?label=version)](https://github.com/ChunzhengLab/sure-unpack/releases) [![license](https://img.shields.io/github/license/ChunzhengLab/sure-unpack)](LICENSE)

English | [中文](README.zh-cn.md)

`sure-unpack` is a command-line tool written in Rust. It detects the archive format automatically and calls the right system tool to extract it.

No more remembering `tar -xzf` vs `unzip -d` vs `7z x -o`. Just `sure-unpack <file>`.

## Install

```sh
brew tap ChunzhengLab/tap
brew install sure-unpack
```

See [homebrew-tap](https://github.com/ChunzhengLab/homebrew-tap) for details.

## Usage

```
sure-unpack [OPTIONS] <ARCHIVE> [DEST]
sure-unpack list <ARCHIVE>
```

The simplest usage is `sure-unpack <file>`. Format is detected from the file extension, and an output directory is created automatically.

## Examples

```sh
sure-unpack project.tar.gz              # → ./project/
sure-unpack archive.zip -C /tmp         # → /tmp/archive/
sure-unpack project.tar.gz --here       # extract into current directory
sure-unpack list project.tar.gz         # preview contents without extracting
sure-unpack -l project.tar.gz           # same as above
sure-unpack -o project.tar.gz           # allow overwriting existing files
sure-unpack --strip-components 1 a.tgz  # strip top-level directory (tar only)
```

## Options

```
-C, --into <DIR>         Extract into specified directory
    --here               Extract into current directory (no subdirectory)
-o, --overwrite          Allow overwriting existing files
    --strip-components N Strip N leading path components (tar only)
-v, --verbose            Show detailed output
-l, --list               Same as 'sure-unpack list'
    --help               Show this help
    --version            Show version
```

## Supported Formats

| Format | Backend | Notes |
|--------|---------|-------|
| `.tar` `.tar.gz` `.tgz` `.tar.bz2` `.tbz2` `.tar.xz` `.txz` `.tar.zst` | `tar` | Supports `--strip-components` |
| `.zip` | `unzip` | |
| `.7z` | `7z` / `7zz` | |
| `.gz` | `gunzip` | Single-file decompression |
| `.bz2` | `bunzip2` | Single-file decompression |
| `.xz` | `xz` | Single-file decompression |
| `.zst` | `zstd` | Single-file decompression |

`sure-unpack` does not parse archive formats itself. It delegates to system tools already installed on your machine. If a required tool is missing, it tells you exactly what to install.

## Defaults

- **Auto subdirectory**: multi-file archives extract to `./<archive-stem>/`, not the current directory. Use `--here` to extract in place.
- **No overwrite**: refuses to extract when the target already exists. Use `-o` to override.
- **Path safety warnings**: entries containing `..` or absolute paths trigger a warning on stderr.
- **Missing tool reporting**: clear error message naming the missing tool and the format it handles.


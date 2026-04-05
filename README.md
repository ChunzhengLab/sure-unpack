# sure-unpack

[![build](https://github.com/ChunzhengLab/sure-unpack/actions/workflows/rust.yml/badge.svg)](https://github.com/ChunzhengLab/sure-unpack/actions/workflows/rust.yml) [![version](https://img.shields.io/github/v/tag/ChunzhengLab/sure-unpack?label=version)](https://github.com/ChunzhengLab/sure-unpack/releases) [![license](https://img.shields.io/github/license/ChunzhengLab/sure-unpack)](LICENSE)

English | [中文](README.zh-cn.md)

Two commands for packing and unpacking archives. `unpack` identifies the format from file headers and extracts it. `pack` creates archives in the format you specify.

No need to remember whether to use `tar`, `unzip`, or `7z`, or what `-xzf`, `-xJf`, `-czf`, `-d`, `-o` means for which tool. Just `unpack <file>` and `pack <dir>`.

## Install

```sh
brew tap ChunzhengLab/tap
brew install unpack
```

See [homebrew-tap](https://github.com/ChunzhengLab/homebrew-tap) for details.

## unpack

```
unpack [OPTIONS] <ARCHIVE> [DEST]
unpack sniff <FILE>...
unpack list <ARCHIVE>
```

### Examples

```sh
unpack project.tar.gz              # → ./project/
unpack archive.zip -C /tmp         # → /tmp/archive/
unpack mystery.bin                 # auto-detect from file header
unpack sniff mystery.bin           # what format is this file?
unpack list project.tar.gz         # preview contents without extracting
unpack project.tar.gz --here       # extract into current directory
unpack --dry-run project.tar.gz    # show what would happen
unpack --format tar.gz mystery.bin # override all detection
```

### Options

```
-C, --into <DIR>         Extract into specified directory
    --here               Extract into current directory (no subdirectory)
-o, --overwrite          Allow overwriting existing files
    --format <FMT>       Override format detection (e.g. tar.gz, zip, 7z)
    --dry-run            Show what would happen without extracting
-v, --verbose            Show detailed output
-l, --list               Same as 'unpack list'
```

## pack

```
pack [OPTIONS] <SRC> [OUT]
```

### Examples

```sh
pack mydir/                        # → mydir.zip (default format)
pack mydir/ output.tar.gz          # format from extension
pack mydir/ output.7z              # 7z archive
pack file.txt output.gz            # single-file compression
pack mydir/ output.tar.lz4         # tar + lz4
pack -f tar.gz mydir/              # explicit format
pack --dry-run mydir/              # show what would happen
```

### Options

```
-f, --format <FMT>   Output format (e.g. zip, tar.gz, 7z)
    --dry-run        Show what would happen without packing
-o, --overwrite      Allow overwriting existing output file
-v, --verbose        Show detailed output
```

### Rules

- Default format is `.zip` when no output path is given
- `--format` and output extension must agree, or an error is raised
- Directories cannot be packed into single-file formats (`.gz`, `.bz2`, `.xz`, `.lz4`, `.zst`) — use the matching `.tar.*` variant instead

## Supported Formats

| Format | unpack | pack | Backend |
|--------|--------|------|---------|
| `.tar` `.tar.gz` `.tgz` `.tar.bz2` `.tbz2` `.tar.xz` `.txz` `.tar.lz4` `.tar.zst` | yes | yes | `tar` / `lz4` |
| `.zip` | yes | yes | `unzip` / `zip` |
| `.7z` | yes | yes | `7z` / `7zz` |
| `.rar` | yes | — | `7z` / `7zz` |
| `.iso` | yes | — | `7z` / `7zz` |
| `.gz` | yes | yes | `gunzip` / `gzip` |
| `.bz2` | yes | yes | `bunzip2` / `bzip2` |
| `.xz` | yes | yes | `xz` |
| `.lz4` | yes | yes | `lz4` |
| `.zst` | yes | yes | `zstd` |

## Defaults

- **Auto subdirectory**: `unpack` extracts multi-file archives to `./<archive-stem>/`, not the current directory.
- **No overwrite**: both commands refuse to clobber existing files. Use `-o` to override.
- **Header sniffing**: `unpack` reads the file header to identify the format, even when the extension is wrong or missing.
- **Missing tool reporting**: clear error message naming the missing tool and the format it handles.

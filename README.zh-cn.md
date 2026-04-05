# sure-unpack

[![build](https://github.com/ChunzhengLab/sure-unpack/actions/workflows/rust.yml/badge.svg)](https://github.com/ChunzhengLab/sure-unpack/actions/workflows/rust.yml) [![version](https://img.shields.io/github/v/tag/ChunzhengLab/sure-unpack?label=version)](https://github.com/ChunzhengLab/sure-unpack/releases) [![license](https://img.shields.io/github/license/ChunzhengLab/sure-unpack)](LICENSE)

[English](README.md) | 中文

解压和打包的双命令工具。`unpack` 通过文件头识别格式并解压，`pack` 按指定格式创建归档。

不用再纠结该调 `tar`、`unzip` 还是 `7z`，也不用记 `-xzf`、`-xJf`、`-czf`、`-d`、`-o` 分别对应哪个工具。`unpack <file>` 和 `pack <dir>` 自动处理一切。

## 安装

```sh
brew tap ChunzhengLab/tap
brew install unpack
```

详见 [homebrew-tap](https://github.com/ChunzhengLab/homebrew-tap)。

## unpack

```
unpack [选项] <归档文件> [目标目录]
unpack sniff <文件>...
unpack list <归档文件>
```

### 示例

```sh
unpack project.tar.gz              # → ./project/
unpack archive.zip -C /tmp         # → /tmp/archive/
unpack mystery.bin                 # 自动识别文件头
unpack sniff mystery.bin           # 这个文件是什么格式？
unpack list project.tar.gz         # 预览内容，不解压
unpack project.tar.gz --here       # 解压到当前目录
unpack --dry-run project.tar.gz    # 预演
unpack --format tar.gz mystery.bin # 覆盖所有检测
```

### 选项

```
-C, --into <DIR>         解压到指定目录
    --here               解压到当前目录，不自动建子目录
-o, --overwrite          允许覆盖已有文件
    --format <FMT>       手动指定格式（如 tar.gz、zip、7z）
    --dry-run            预演，不实际解压
-v, --verbose            显示详细输出
-l, --list               等同于 unpack list
```

## pack

```
pack [选项] <源路径> [输出文件]
```

### 示例

```sh
pack mydir/                        # → mydir.zip（默认格式）
pack mydir/ output.tar.gz          # 从扩展名推断格式
pack mydir/ output.7z              # 7z 归档
pack file.txt output.gz            # 单文件压缩
pack mydir/ output.tar.lz4         # tar + lz4
pack -f tar.gz mydir/              # 显式指定格式
pack --dry-run mydir/              # 预演
```

### 选项

```
-f, --format <FMT>   输出格式（如 zip、tar.gz、7z）
    --dry-run        预演，不实际打包
-o, --overwrite      允许覆盖已有输出文件
-v, --verbose        显示详细输出
```

### 规则

- 没有指定输出文件时，默认格式为 `.zip`
- `--format` 和输出扩展名不一致时直接报错，不猜测
- 目录不能压缩为单文件格式（`.gz`、`.bz2`、`.xz`、`.lz4`、`.zst`），请使用对应的 `.tar.*` 格式

## 支持的格式

| 格式 | unpack | pack | 后端工具 |
|------|--------|------|---------|
| `.tar` `.tar.gz` `.tgz` `.tar.bz2` `.tbz2` `.tar.xz` `.txz` `.tar.lz4` `.tar.zst` | 支持 | 支持 | `tar` / `lz4` |
| `.zip` | 支持 | 支持 | `unzip` / `zip` |
| `.7z` | 支持 | 支持 | `7z` / `7zz` |
| `.rar` | 支持 | — | `7z` / `7zz` |
| `.iso` | 支持 | — | `7z` / `7zz` |
| `.gz` | 支持 | 支持 | `gunzip` / `gzip` |
| `.bz2` | 支持 | 支持 | `bunzip2` / `bzip2` |
| `.xz` | 支持 | 支持 | `xz` |
| `.lz4` | 支持 | 支持 | `lz4` |
| `.zst` | 支持 | 支持 | `zstd` |

## 默认行为

- **自动建立子目录**：`unpack` 将多文件归档解压到 `./<归档名>/`，而非直接散落在当前目录。
- **拒绝覆盖**：两个命令默认都拒绝覆盖已有文件，使用 `-o` 可强制覆盖。
- **文件头探测**：`unpack` 通过读取文件头识别格式，即使扩展名错误或缺失也能正确处理。
- **缺少工具时明确报错**：提示缺少哪个工具以及它对应哪种格式。

# sure-unpack

[![build](https://github.com/ChunzhengLab/sure-unpack/actions/workflows/rust.yml/badge.svg)](https://github.com/ChunzhengLab/sure-unpack/actions/workflows/rust.yml) [![version](https://img.shields.io/github/v/tag/ChunzhengLab/sure-unpack?label=version)](https://github.com/ChunzhengLab/sure-unpack/releases) [![license](https://img.shields.io/github/license/ChunzhengLab/sure-unpack)](LICENSE)

[English](README.md) | 中文

`sure-unpack` 是一个用 Rust 编写的命令行工具。它自动识别归档格式，调用系统中已安装的解压工具完成解包。

不用再记 `tar -xzf`、`unzip -d`、`7z x -o` 之间的差异，一个 `sure-unpack <file>` 就够了。

## 安装

```sh
brew tap ChunzhengLab/tap
brew install sure-unpack
```

详见 [homebrew-tap](https://github.com/ChunzhengLab/homebrew-tap)。

## 用法

```
sure-unpack [选项] <归档文件> [目标目录]
sure-unpack list <归档文件>
```

最简单的用法是 `sure-unpack <file>`，格式从扩展名自动识别，输出目录自动创建。

## 示例

```sh
sure-unpack project.tar.gz              # → ./project/
sure-unpack archive.zip -C /tmp         # → /tmp/archive/
sure-unpack project.tar.gz --here       # 解压到当前目录，不建子目录
sure-unpack list project.tar.gz         # 预览内容，不解压
sure-unpack -l project.tar.gz           # 同上
sure-unpack -o project.tar.gz           # 允许覆盖已有文件
sure-unpack --strip-components 1 a.tgz  # 去掉顶层目录（仅限 tar）
```

## 选项

```
-C, --into <DIR>         解压到指定目录
    --here               解压到当前目录，不自动建子目录
-o, --overwrite          允许覆盖已有文件
    --strip-components N 去掉前 N 层路径（仅限 tar）
-v, --verbose            显示详细输出
-l, --list               等同于 sure-unpack list
    --help               显示帮助
    --version            显示版本
```

## 支持的格式

| 格式 | 后端工具 | 说明 |
|------|---------|------|
| `.tar` `.tar.gz` `.tgz` `.tar.bz2` `.tbz2` `.tar.xz` `.txz` `.tar.zst` | `tar` | 支持 `--strip-components` |
| `.zip` | `unzip` | |
| `.7z` | `7z` / `7zz` | |
| `.gz` | `gunzip` | 单文件解压 |
| `.bz2` | `bunzip2` | 单文件解压 |
| `.xz` | `xz` | 单文件解压 |
| `.zst` | `zstd` | 单文件解压 |

`sure-unpack` 本身不解析压缩格式，而是调用系统中已安装的工具。如果缺少所需工具，会明确提示需要安装什么。

## 默认行为

- **自动建立子目录**：多文件归档解压到 `./<归档名>/`，而非直接散落在当前目录。使用 `--here` 可解压到当前目录。
- **拒绝覆盖**：目标已存在时拒绝解压，避免意外覆盖。使用 `-o` 可强制覆盖。
- **路径安全警告**：归档中包含 `..` 或绝对路径的条目会在 stderr 输出警告。
- **缺少工具时明确报错**：提示缺少哪个工具以及它对应哪种格式。


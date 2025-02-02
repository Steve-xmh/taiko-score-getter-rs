# Taiko Score Getter

这是一个用于获取太鼓达人成绩的工具。使用本工具可以方便到获取到您的太鼓达人的成绩信息。

目前已测试的平台有 Windows 和 macOS 平台，Linux 尚未测试（大概率不支持）。

## 和[原版](https://github.com/donnote/taiko_score_getter_cn)的差别

本程序使用 Rust 语言编写，在保持基本功能可用的情况下：

- 有一个比较好看的简易 GUI（目前仅支持 Windows 和 macOS，其他平台开发中）
- 程序体积减小（19MB 降低到 4MB 左右）
- 证书可自动安装
    - Windows 无感安装，无需任何操作
    - macOS 用户虽然可以自动安装证书，但是仍然需要用户手动信任方可使用
- 自动还原代理配置

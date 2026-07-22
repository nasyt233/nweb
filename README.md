# nweb

[简体中文](README.md)
[English](README_en.md)

一个用 Rust 编写的极简文件浏览器 —— 无需配置，开箱即用。

## 特点

- 📁 单页浏览：点击文件夹即可展开，无需跳转页面。
- 🎨 可自定义外观：通过 `nweb.yml` 修改标题、介绍、背景图、透明度与模糊度。
- 📋 日志记录：每次请求（路径 + 状态码）自动记录到 `nweb.log`，方便排查问题。
- 🔒 隐藏敏感文件：`nweb.yml`、`nweb.log`、`README*` 在列表不可见，直接访问返回 404。
- 💾 文件下载：点击任意文件即可下载，所有 MIME 类型均支持。
- ⚡ 快速响应：仅扫描当前目录，深层文件夹按需加载，性能优异。

## 快速开始

### 1. 安装
从 [Releases](https://github.com/nasyt233/nweb/releases) 下载对应平台的 `nweb` 二进制文件，或自行编译：

通用一键安装脚本
```bash
bash -c "$(curl -L https://raw.gitcode.com/nasyt/nweb/raw/master/install.sh)"
```

从源码开始构建
```bash
git clone https://github.com/nasyt233/nweb.git
cd nweb
cargo build --release
```

### 2. 运行

```bash
./nweb <目录> <端口>
```

示例（将当前目录作为根目录，端口 8080）：

```bash
./nweb . 8080
```

启动后，打开浏览器访问 http://127.0.0.1:8080 即可浏览文件。

### 3. 自定义配置（可选）

在服务根目录下创建 nweb.yml 文件（首次启动会自动生成），内容如下：

```yaml
title: "我的文件站"                # 网页标题
description: "文件浏览器"           # 网页介绍
background_api: "https://example.com/bg.jpg"  # 背景图URL
opacity: 0.3                      # 背景透明度（0~1）
blur: "5px"                       # 毛玻璃模糊程度
```

修改后重启服务即可生效。

## 高级用法

获取目录树 JSON

外部程序可通过 /api/路径 获取任意目录的 JSON 结构，例如：

```bash
curl http://127.0.0.1:8080/api/src
```

返回：

```json
[
  {"name": "main.rs", "is_dir": false, "size": 1024, "path": "src/main.rs"},
  {"name": "lib", "is_dir": true, "size": 0, "path": "src/lib"}
]
```

## 日志

所有请求（含 API 和文件下载）会记录到 nweb.log，同时输出到终端，格式如下：

```
[2025-01-01 12:00:00] / 200
[2025-01-01 12:00:01] /api/src 200
[2025-01-01 12:00:02] /README.md 404
```

## 许可证

MIT License

##交流群

欢迎加入 NAS油条技术交流群 610699712

# nweb

[简体中文](README.md)
[English](README.en.md)

一个用 Rust 编写的极简文件浏览器 —— 无需配置，开箱即用。

## 特点
- 📁 单页浏览：点击文件夹即可展开，无需跳转页面
- 🌐 完美支持中文路径和文件名
- 🎨 可自定义外观：标题、描述、背景图、透明度、模糊度
- 🌙 内置夜间模式，保护眼睛
- 📋 访问日志：记录请求路径、状态码、访问者 IP
- 🔐 管理后台：在线修改配置，查看运行状态和日志，文件管理，运行命令等
- 🗂️ 后台文件管理：搜索、排序、批量删除、重命名、下载、在线编辑文本文件
- 🛡️ 安全加固：敏感文件保护、管理端同源校验、命令执行超时
- 🔒 隐藏敏感文件：nweb.yml、nweb.log 在列表不可见
- 💾 文件下载：支持所有 MIME 类型
- ⚡ 快速响应：仅扫描当前目录，按需加载
- 🔎 目录内即时搜索与文件类型筛选
- ↕️ 按名称、大小、修改时间排序，并支持路径面包屑导航

## 快速开始
### 安装

通用一键安装脚本
```bash
bash -c "$(curl -L https://raw.gitcode.com/nasyt/nweb/raw/master/install.sh)"
```

从 [Releases](https://github.com/nasyt233/nweb/releases) 下载二进制文件，或自行编译：
```bash
git clone https://github.com/nasyt233/nweb.git
cd nweb
cargo build --release
```

### 运行
基础启动命令
```bash
./nweb <目录> <端口>
```

示例（当前目录，8080端口）
```bash
./nweb . 8080
```

访问地址：http://127.0.0.1:8080 即可浏览文件
管理后台地址：http://127.0.0.1:8080/@admin
默认账号密码：`nweb` / `nweb`

## 配置文件
程序首次运行后自动生成 `nweb.yml`，修改配置后即时生效，无需重启服务
```yaml
title: "我的文件站"
description: "文件浏览器"
background_api: "https://www.loliapi.com/acg/"
opacity: 0.3
blur: "5px"
show_hidden: false
index_file: "index.html"
admin_user: "nweb"
admin_pass: "nweb"
default_sort: "name"          # 可选值 name / size / time
show_file_size: true
show_admin_btn: true
clear_log_on_start: true
home_dir: ""                  # 临时主页目录，留空使用启动目录
enable_exec: true
```

## 高级用法
### API 接口
获取目录树 JSON 数据
```bash
curl http://127.0.0.1:8080/api/tree/
curl http://127.0.0.1:8080/api/src
```

接口返回示例
```json
[
  {"name": "main.rs", "is_dir": false, "size": 1024, "path": "src/main.rs"},
  {"name": "lib", "is_dir": true, "size": 0, "path": "src/lib"}
]
```

### 命令行参数
| 参数 | 说明 | 默认值 |
| ---- | ---- | ---- |
| `<目录> <端口>` | 旧版启动传参方式 | - |
| -d, --dir | 指定服务根目录 | `.` |
| -p, --port | 指定监听端口 | `8080` |
| -H, --host | 绑定主机IP | `0.0.0.0` |
| -h, --help | 查看帮助信息 | - |
| -v, --version | 查看程序版本 | - |

## 日志
日志输出格式示例
```
[22:18:10] ✓ 200 192.168.1.100 /
[22:18:10] ✓ 200 192.168.1.100 /api/tree/
[22:19:00] ✓ 200 192.168.1.100 /api/tree/target
```

## 更新日志
### v0.9.0
- 新增 主页面目录内搜索、类型筛选、排序和面包屑导航
- 新增 后台文件编辑、重命名、批量删除、下载与上传覆盖开关
- 优化 后台文件管理界面与移动端布局
- 修复 `default_sort` 配置未生效
- 修复 URL 编码可绕过敏感文件保护
- 修复配置值注入页面样式的问题
- 增加 管理端请求同源校验与命令执行超时

## 许可证
MIT License

## 交流群
QQ群：610699712
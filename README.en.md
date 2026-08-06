# nweb

[简体中文](README.md)
[English](README.en.md)

A minimal file browser written in Rust — zero configuration, ready to run out of the box.

## Features
- 📁 Single-page navigation: Expand folders inline without page reloads
- 🌐 Full native support for Chinese paths and filenames
- 🎨 Fully customizable appearance: site title, description, background image, transparency, blur effect
- 🌙 Built-in dark mode for eye protection
- 📋 Access logging: record request path, status code, visitor IP address
- 🔐 Admin dashboard: edit config online, view runtime status and access logs
- 🔒 Sensitive file hiding: `nweb.yml` and `nweb.log` are hidden from file lists
- 💾 Universal file download: support for all MIME types
- ⚡ Lightning fast response: only scan current directory, load content on demand

## Quick Start
### Installation
One-click installation script (Linux/macOS):
```bash
bash -c "$(curl -L https://raw.gitcode.com/nasyt/nweb/raw/master/install.sh)"
```

Download prebuilt binaries from [Releases](https://github.com/nasyt233/nweb/releases), or build from source manually:
```bash
git clone https://github.com/nasyt233/nweb.git
cd nweb
cargo build --release
```

### Run
Basic legacy startup syntax:
```bash
./nweb <directory> <port>
```

Example (serve current directory on port 8080):
```bash
./nweb . 8080
```

- File browser: http://127.0.0.1:8080
- Admin panel: http://127.0.0.1:8080/@admin
- Default credentials: `nweb` / `nweb`

## Configuration File
`nweb.yml` will be auto-generated on first launch. Changes take effect instantly without restarting the service.
```yaml
title: "My File Station"
description: "File Browser"
background_api: "https://www.loliapi.com/acg/"
opacity: 0.3
blur: "5px"
show_hidden: false
index_file: "index.html"
admin_user: "nweb"
admin_pass: "nweb"
default_sort: "name"          # Available options: name / size / time
show_file_size: true
show_admin_btn: true
clear_log_on_start: true
home_dir: ""                  # Custom root directory; leave empty to use launch directory
```

## Advanced Usage
### API Endpoints
Fetch directory tree as JSON:
```bash
curl http://127.0.0.1:8080/api/tree/
curl http://127.0.0.1:8080/api/src
```

Sample JSON response:
```json
[
  {"name": "main.rs", "is_dir": false, "size": 1024, "path": "src/main.rs"},
  {"name": "lib", "is_dir": true, "size": 0, "path": "src/lib"}
]
```

### CLI Arguments
| Argument | Description | Default Value |
| ---- | ---- | ---- |
| `<directory> <port>` | Legacy positional startup arguments | - |
| -d, --dir | Set root serving directory | `.` |
| -p, --port | Set listening port | `8080` |
| -H, --host | Bind listening IP address | `0.0.0.0` |
| -h, --help | Print help information | - |
| -v, --version | Print program version | - |

## Log Format
Sample log output:
```
[22:18:10] ✓ 200 192.168.1.100 /
[22:18:10] ✓ 200 192.168.1.100 /api/tree/
[22:19:00] ✓ 200 192.168.1.100 /api/tree/target
```

## License
MIT License

## Community Group
QQ Group: 610699712

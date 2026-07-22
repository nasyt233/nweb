# nweb

[简体中文](README.md)
[English](README_en.md)

A minimalist file browser written in Rust - no configuration required, ready to use out of the box.


## Features

- 📁 single-page browsing: Click on the folder to expand without jumping to the page.
- 🎨 customizable appearance: modify title, introduction, background image, transparency, and blur via 'nweb.yml'.
- 📋 log recording: Each request (path + status code) is automatically recorded to 'nweb.log' for easy troubleshooting.
- 🔒 hide sensitive files: 'nweb.yml', 'nweb.log', 'README*' are not visible in the list, and direct access returns 404.
- 💾 file download: Click on any file to download. All MIME types are supported.
- ⚡ Quick response: only scans the current directory, deep folders are loaded on demand, and performance is excellent.

Get started quickly

1. fix
From [Releases] (https://github.com/nasyt233/nweb/releases) to download the corresponding platform ` nweb ` binary file, or to compile:

Universal One-Click Installation Script
```bash
bash -c "$(curl -L https://raw.gitcode.com/nasyt/nweb/raw/master/install.sh)"
```

Build from the source code
```bash
git clone https://github.com/nasyt233/nweb.git
cd nweb
cargo build --release
` ` `

2. Run

```bash
./nweb < Directory > < Port >
` ` `

Example (taking the current directory as the root directory, port 8080) :

```bash
./nweb . 8080
` ` `

Once launched, open a browser and visit http://127.0.0.1:8080 to view the file.

3. Custom Configuration (Optional)

Create the nweb.yml file in the service root directory (automatically generated upon the first startup), with the following content:

```yaml
title: "My File Station" # Web Page Title
description: "File Browser" # Webpage Introduction
Background_api: "https://example.com/bg.jpg" # background URL
opacity: 0.3 # Background transparency (0-1)
blur: "5px" # frosted glass blur degree
` ` `

The modification will take effect after restarting the service.

Advanced usage

Get the directory tree JSON

External programs can obtain the JSON structure of any directory through the /api/ path, for example:

```bash
The curl http://127.0.0.1:8080/api/src
` ` `

Return

```json
[
{"name": "main.rs", "is_dir": false, "size": 1024, "path": "src/main.rs"},
{"name": "lib", "is_dir": true, "size": 0, "path": "src/lib"}
]
` ` `

"Log"

All requests (including API and file downloads) will be recorded in nweb.log and simultaneously output to the terminal in the following format:

` ` `
[2025-01-01 12:00:00] / 200
[2025-01-01 12:00:01] /api/src 200
[2025-01-01 12:00:02] /README.md 404
` ` `

"License"

MIT License

"Communication group.

Welcome to join the NAS Youtiao Technology Exchange Group at 610699712
#!/bin/bash
myuname=$(uname | tr 'A-Z' 'a-z');arch=$(uname -m)
if [[ -n $PREFIX ]]; then arch=aarch64;else [[ $arch == "x86_64" ]] && arch="amd64"; [[ $arch == "aarch64" ]] && arch="arm64";fi
[[ -n $PREFIX ]] && bin=$PREFIX/bin || bin=/usr/bin
echo "正在获取最新版本信息";tag_name=$(curl -s 'https://api.github.com/repos/nasyt233/nweb/releases/latest' | grep -m1 '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
curl -s https://myip.ipip.net | grep -oE "中国|China" >/dev/null 2>&1 && speed=https://gh-proxy.org/
dow_url="${speed}https://github.com/nasyt233/nweb/releases/download/$tag_name/$tag_name-$myuname-$arch"
echo "正在下载文件";curl --progress-bar -o nweb -L $dow_url || echo "文件下载失败"
chmod +x nweb;mv nweb $bin
echo "nweb安装完成";echo "输入nweb查看帮助"
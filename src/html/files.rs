//! 后台文件管理模块（仅在管理员认证通过后注入到管理页面）

pub fn generate_files_html() -> String {
    let template = r####"
<section class="card" id="fmCard">
    <h2>📁 文件管理 <span class="fm-ver">v__VERSION__</span></h2>
    <div class="fm-bar">
        <button class="btn fm-btn-ghost" onclick="fmUp()">⬆ 上级目录</button>
        <span class="fm-path" id="fmPath">/</span>
        <button class="btn fm-btn-ghost" onclick="fmLoad(fmPath)">🔄 刷新</button>
    </div>
    <div class="fm-bar">
        <input type="text" id="fmNewDir" placeholder="新文件夹名称">
        <button class="btn" onclick="fmMkdir()">📁 新建文件夹</button>
    </div>
    <div class="fm-bar">
        <input type="file" id="fmUploadInput" multiple>
        <button class="btn" onclick="fmUpload()">⬆ 上传到当前目录</button>
        <span class="fm-status" id="fmStatus"></span>
    </div>
    <div class="fm-list" id="fmList"><div class="fm-empty">加载中...</div></div>
    <style>
        #fmCard .fm-bar { display:flex; gap:10px; align-items:center; flex-wrap:wrap; margin:0 0 10px; }
        #fmCard .fm-bar input[type="text"] { flex:1; min-width:150px; }
        #fmCard .fm-bar input[type="file"] { flex:1; padding:8px; }
        .fm-btn-ghost { background:#eef2ff !important; color:#4338ca !important; }
        .fm-path { color:#4f46e5; font-weight:600; word-break:break-all; font-family:Consolas,monospace; font-size:13px; }
        .fm-ver { font-size:12px; color:#9ca3af; font-weight:400; }
        .fm-list { border:1px solid rgba(102,126,234,0.18); border-radius:12px; overflow:hidden; background:rgba(255,255,255,0.72); }
        .fm-row { display:flex; align-items:center; gap:10px; padding:9px 12px; border-bottom:1px solid rgba(102,126,234,0.10); font-size:14px; }
        .fm-row:last-child { border-bottom:none; }
        .fm-row:hover { background:rgba(102,126,234,0.07); }
        .fm-name { flex:1; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; color:#374151; text-decoration:none; cursor:pointer; }
        a.fm-name:hover { color:#4f46e5; text-decoration:underline; }
        .fm-meta { color:#9ca3af; font-size:12px; white-space:nowrap; }
        .fm-del { background:none; border:none; color:#ef4444; cursor:pointer; font-size:13px; padding:0; }
        .fm-del:hover { text-decoration:underline; }
        .fm-status { color:#4f46e5; font-size:13px; }
        .fm-empty { padding:16px; color:#9ca3af; text-align:center; }
    </style>
    <script>
        let fmPath = "";
        function fmH(s){
            return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
                .replace(/"/g, "&quot;").replace(new RegExp(String.fromCharCode(39), "g"), "&#39;");
        }
        function fmEnc(p){ return String(p).split("/").map(encodeURIComponent).join("/"); }
        function fmParent(p){ var i = p.lastIndexOf("/"); return i < 0 ? "" : p.substring(0, i); }
        function fmSize(n){
            if(n < 1024) return n + " B";
            var u = ["KB", "MB", "GB", "TB"], i = -1;
            do { n = n / 1024; i++; } while(n >= 1024 && i < u.length - 1);
            return n.toFixed(1) + " " + u[i];
        }
        function fmTime(t){
            if(!t) return "-";
            var d = new Date(t * 1000), p = function(v){ return (v < 10 ? "0" : "") + v; };
            return d.getFullYear() + "-" + p(d.getMonth() + 1) + "-" + p(d.getDate()) + " " + p(d.getHours()) + ":" + p(d.getMinutes());
        }
        function fmMsg(t){ document.getElementById("fmStatus").textContent = t || ""; }

        async function fmLoad(path){
            fmPath = path || "";
            document.getElementById("fmPath").textContent = "/" + fmPath;
            var list = document.getElementById("fmList");
            list.innerHTML = "<div class=\"fm-empty\">加载中...</div>";
            try {
                var res = await fetch("/@admin/files?path=" + encodeURIComponent(fmPath));
                if(!res.ok) throw new Error("HTTP " + res.status);
                var data = await res.json();
                if(data.error){ list.innerHTML = "<div class=\"fm-empty\">❌ " + fmH(data.error) + "</div>"; return; }
                var items = data.entries || [];
                if(!items.length){ list.innerHTML = "<div class=\"fm-empty\">空目录</div>"; return; }
                list.innerHTML = items.map(function(it){
                    var icon = it.is_dir ? "📁" : (it.sensitive ? "🔒" : "📄");
                    var name;
                    if(it.is_dir){
                        name = "<a class=\"fm-name\" data-dir=\"" + fmH(it.path) + "\">" + fmH(it.name) + "/</a>";
                    } else if(it.sensitive){
                        name = "<span class=\"fm-name\">" + fmH(it.name) + "</span>";
                    } else {
                        name = "<a class=\"fm-name\" href=\"/" + fmEnc(it.path) + "\" target=\"_blank\">" + fmH(it.name) + "</a>";
                    }
                    var meta = (it.is_dir ? "文件夹" : fmSize(it.size)) + " · " + fmTime(it.modified);
                    var del = it.sensitive
                        ? "<span class=\"fm-meta\">🔒 受保护</span>"
                        : "<button class=\"fm-del\" data-del=\"" + fmH(it.path) + "\" data-isdir=\"" + (it.is_dir ? "1" : "0") + "\">删除</button>";
                    return "<div class=\"fm-row\"><span>" + icon + "</span>" + name + "<span class=\"fm-meta\">" + meta + "</span>" + del + "</div>";
                }).join("");
                Array.prototype.forEach.call(list.querySelectorAll("[data-dir]"), function(el){
                    el.addEventListener("click", function(){ fmLoad(el.getAttribute("data-dir")); });
                });
                Array.prototype.forEach.call(list.querySelectorAll("[data-del]"), function(el){
                    el.addEventListener("click", function(){ fmDelete(el.getAttribute("data-del"), el.getAttribute("data-isdir") === "1"); });
                });
            } catch(e) {
                list.innerHTML = "<div class=\"fm-empty\">❌ 加载失败: " + fmH(e.message) + "</div>";
            }
        }

        async function fmDelete(path, isDir){
            var tip = isDir
                ? "确认删除文件夹 " + path + " 及其中的全部内容？此操作不可恢复"
                : "确认删除文件 " + path + " ？";
            if(!confirm(tip)) return;
            try {
                var res = await fetch("/@admin/files/delete?path=" + encodeURIComponent(path), { method: "POST" });
                var data = await res.json();
                if(data.error){ alert("❌ " + data.error); return; }
                fmMsg("✅ 已删除 " + path);
                fmLoad(fmPath);
            } catch(e) { alert("❌ 删除失败: " + e.message); }
        }

        async function fmMkdir(){
            var input = document.getElementById("fmNewDir");
            var name = input.value.trim();
            if(!name){ fmMsg("请输入文件夹名称"); return; }
            try {
                var res = await fetch("/@admin/files/mkdir?path=" + encodeURIComponent(fmPath) + "&name=" + encodeURIComponent(name), { method: "POST" });
                var data = await res.json();
                if(data.error){ alert("❌ " + data.error); return; }
                input.value = "";
                fmMsg("✅ 已创建 " + name);
                fmLoad(fmPath);
            } catch(e) { alert("❌ 创建失败: " + e.message); }
        }

        async function fmUpload(){
            var input = document.getElementById("fmUploadInput");
            var files = Array.prototype.slice.call(input.files || []);
            if(!files.length){ fmMsg("请先选择文件"); return; }
            for(var i = 0; i < files.length; i++){
                var f = files[i];
                fmMsg("上传中 (" + (i + 1) + "/" + files.length + "): " + f.name);
                try {
                    var res = await fetch("/@admin/upload?dir=" + encodeURIComponent(fmPath) + "&name=" + encodeURIComponent(f.name) + "&overwrite=1", { method: "POST", body: f });
                    var data = await res.json().catch(function(){ return { error: "HTTP " + res.status }; });
                    if(!res.ok || data.error){ fmMsg("❌ " + f.name + ": " + (data.error || ("HTTP " + res.status))); continue; }
                } catch(e) { fmMsg("❌ " + f.name + ": " + e.message); continue; }
            }
            fmMsg("✅ 上传完成");
            input.value = "";
            fmLoad(fmPath);
        }

        function fmUp(){ if(!fmPath) return; fmLoad(fmParent(fmPath)); }

        document.addEventListener("DOMContentLoaded", function(){ fmLoad(""); });
    </script>
</section>
"####;
    template.replace("__VERSION__", env!("CARGO_PKG_VERSION"))
}

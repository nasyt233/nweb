//! 后台文件管理模块（仅在管理员认证通过后注入到管理页面）

pub fn generate_files_html() -> String {
    let template = r####"
<section class="card" id="fmCard">
    <h2>📁 文件管理 <span class="fm-ver">v__VERSION__</span></h2>
    <div class="fm-toolbar">
        <button class="btn fm-btn-ghost" type="button" onclick="fmUp()" title="上级目录">⬆ 上级</button>
        <span class="fm-path" id="fmPath">/</span>
        <button class="btn fm-btn-ghost" type="button" onclick="fmLoad(fmPath)" title="刷新目录">↻ 刷新</button>
    </div>
    <div class="fm-toolbar">
        <input type="search" id="fmSearch" placeholder="搜索当前目录" aria-label="搜索文件">
        <select id="fmSort" aria-label="排序方式">
            <option value="name">名称</option><option value="size">大小</option><option value="modified">修改时间</option>
        </select>
        <select id="fmFilter" aria-label="文件类型">
            <option value="all">全部类型</option><option value="folder">文件夹</option><option value="file">文件</option>
        </select>
    </div>
    <div class="fm-toolbar">
        <input type="text" id="fmNewDir" placeholder="新文件夹名称">
        <button class="btn" type="button" onclick="fmMkdir()">＋ 新建文件夹</button>
        <input type="file" id="fmUploadInput" multiple>
        <label class="fm-overwrite"><input type="checkbox" id="fmOverwrite"> 覆盖同名文件</label>
        <button class="btn" type="button" onclick="fmUpload()">↑ 上传文件</button>
    </div>
    <div class="fm-actions">
        <label><input type="checkbox" id="fmSelectAll"> 全选</label>
        <span id="fmSelectionCount">已选 0 项</span>
        <button class="btn fm-btn-danger" type="button" id="fmDeleteSelected" disabled>删除所选</button>
        <span class="fm-status" id="fmStatus" role="status"></span>
    </div>
    <div class="fm-list" id="fmList"><div class="fm-empty">加载中...</div></div>
    <div class="fm-modal" id="fmEditor" hidden>
        <div class="fm-dialog" role="dialog" aria-modal="true" aria-labelledby="fmEditorTitle">
            <div class="fm-dialog-head"><strong id="fmEditorTitle">文本编辑器</strong><button type="button" class="fm-icon-btn" onclick="fmCloseEditor()" title="关闭">✕</button></div>
            <textarea id="fmEditorText" spellcheck="false"></textarea>
            <div class="fm-dialog-foot"><span id="fmEditorStatus"></span><button type="button" class="btn fm-btn-ghost" onclick="fmCloseEditor()">取消</button><button type="button" class="btn" onclick="fmSaveEditor()">保存</button></div>
        </div>
    </div>
    <style>
        #fmCard .fm-toolbar { display:flex; gap:9px; align-items:center; flex-wrap:wrap; margin:0 0 10px; }
        #fmCard .fm-toolbar input[type="text"], #fmCard .fm-toolbar input[type="search"] { flex:1; min-width:150px; }
        #fmCard .fm-toolbar input[type="file"] { flex:1; min-width:180px; padding:6px; }
        #fmCard .fm-toolbar select { width:auto; min-width:120px; }
        #fmCard .fm-toolbar input, #fmCard .fm-toolbar select { border:1px solid #d1d5db; border-radius:8px; background:#fff; color:#273244; }
        .fm-btn-ghost { background:#eef2ff !important; color:#4338ca !important; }
        .fm-btn-danger { background:#fff1f2 !important; color:#be123c !important; }
        .fm-btn-danger:disabled { opacity:.45; cursor:not-allowed; }
        .fm-path { color:#4f46e5; font-weight:600; overflow-wrap:anywhere; font-family:monospace; font-size:13px; flex:1; }
        .fm-ver { font-size:12px; color:#9ca3af; font-weight:400; }
        .fm-actions { display:flex; align-items:center; gap:12px; flex-wrap:wrap; margin:4px 0 10px; font-size:13px; color:#64748b; }
        .fm-overwrite { display:flex; align-items:center; gap:5px; color:#64748b; font-size:13px; white-space:nowrap; }
        .fm-overwrite input { width:auto; }
        .fm-actions label { display:flex; align-items:center; gap:6px; margin:0; }
        .fm-list { border:1px solid rgba(102,126,234,.18); border-radius:8px; overflow:auto; max-height:60vh; background:rgba(255,255,255,.78); }
        .fm-row { display:grid; grid-template-columns:20px 24px minmax(130px,1fr) auto auto; align-items:center; gap:10px; padding:9px 12px; border-bottom:1px solid rgba(102,126,234,.10); font-size:14px; }
        .fm-row:last-child { border-bottom:0; }
        .fm-row:hover { background:rgba(102,126,234,.07); }
        .fm-check { width:16px; height:16px; }
        .fm-name { min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; color:#374151; text-decoration:none; cursor:pointer; }
        a.fm-name:hover { color:#4f46e5; text-decoration:underline; }
        .fm-meta { color:#9ca3af; font-size:12px; white-space:nowrap; }
        .fm-buttons { display:flex; align-items:center; gap:8px; }
        .fm-action { border:0; background:none; color:#4f46e5; cursor:pointer; font:inherit; font-size:13px; padding:3px; }
        .fm-action.danger { color:#e11d48; }
        .fm-status { color:#4f46e5; font-size:13px; }
        .fm-empty { padding:18px; color:#9ca3af; text-align:center; }
        .fm-modal { position:fixed; inset:0; z-index:100000; display:grid; place-items:center; padding:16px; background:rgba(15,23,42,.62); }
        .fm-modal[hidden] { display:none; }
        .fm-dialog { width:min(900px,100%); max-height:90vh; display:flex; flex-direction:column; gap:12px; padding:16px; border-radius:10px; background:#fff; box-shadow:0 20px 60px rgba(0,0,0,.3); }
        .fm-dialog-head,.fm-dialog-foot { display:flex; align-items:center; justify-content:space-between; gap:10px; }
        .fm-dialog-head { color:#1f2937; }
        .fm-dialog-foot { justify-content:flex-end; color:#64748b; font-size:13px; }
        #fmEditorText { width:100%; min-height:45vh; resize:vertical; padding:12px; border:1px solid #cbd5e1; border-radius:8px; font:13px/1.5 ui-monospace,monospace; tab-size:4; }
        .fm-icon-btn { border:0; background:none; cursor:pointer; font-size:18px; color:#64748b; }
        @media(max-width:640px) { .fm-row { grid-template-columns:18px 20px minmax(80px,1fr) auto; gap:7px; } .fm-meta { display:none; } .fm-buttons { gap:5px; } .fm-action { font-size:12px; } .fm-list { max-height:55vh; } }
    </style>
    <script>
        let fmPath = "", fmEntries = [], fmEditingPath = "";
        function fmH(s){ return String(s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/\"/g,"&quot;").replace(/'/g,"&#39;"); }
        function fmEnc(p){ return String(p).split("/").map(encodeURIComponent).join("/"); }
        function fmParent(p){ var i=p.lastIndexOf("/"); return i<0?"":p.substring(0,i); }
        function fmSize(n){ if(n<1024)return n+" B"; var u=["KB","MB","GB","TB"],i=-1; do{n/=1024;i++;}while(n>=1024&&i<u.length-1); return n.toFixed(1)+" "+u[i]; }
        function fmTime(t){ if(!t)return "-"; var d=new Date(t*1000),p=function(v){return(v<10?"0":"")+v;}; return d.getFullYear()+"-"+p(d.getMonth()+1)+"-"+p(d.getDate())+" "+p(d.getHours())+":"+p(d.getMinutes()); }
        function fmMsg(t){ document.getElementById("fmStatus").textContent=t||""; }
        function fmType(it){ var e=(it.name.split(".").pop()||"").toLowerCase(); if(it.is_dir)return "folder"; if(["jpg","jpeg","png","gif","webp","svg","bmp"].includes(e))return "图片"; if(["mp4","mkv","mov","webm"].includes(e))return "视频"; if(["mp3","wav","flac","ogg"].includes(e))return "音频"; if(["pdf","txt","md","doc","docx","json","yaml","yml","toml","rs","js","css","html","py"].includes(e))return "文档"; return "文件"; }
        function fmRender(){
            var list=document.getElementById("fmList"), q=document.getElementById("fmSearch").value.trim().toLocaleLowerCase(), filter=document.getElementById("fmFilter").value, sort=document.getElementById("fmSort").value;
            var items=fmEntries.filter(function(it){return it.name.toLocaleLowerCase().includes(q)&&(filter==="all"||(filter==="folder"&&it.is_dir)||(filter==="file"&&!it.is_dir));});
            items.sort(function(a,b){if(a.is_dir!==b.is_dir)return a.is_dir?-1:1;if(sort==="size")return b.size-a.size||a.name.localeCompare(b.name);if(sort==="modified")return b.modified-a.modified||a.name.localeCompare(b.name);return a.name.localeCompare(b.name,undefined,{sensitivity:"base",numeric:true});});
            if(!items.length){list.innerHTML='<div class="fm-empty">'+(q||filter!=="all"?"没有匹配项":"空目录")+'</div>';fmUpdateSelection();return;}
            list.innerHTML=items.map(function(it){
                var icon=it.is_dir?"📁":(it.sensitive?"🔒":"📄"), name=it.is_dir?'<button type="button" class="fm-action fm-name" data-dir="'+fmH(it.path)+'">'+fmH(it.name)+"/</button>":'<a class="fm-name" href="/'+fmEnc(it.path)+'" target="_blank" rel="noopener">'+fmH(it.name)+"</a>";
                var meta=(it.is_dir?"文件夹 · "+it.name:" "+fmSize(it.size)+" · "+fmTime(it.modified));
                var actions=it.sensitive?'<span class="fm-meta">受保护</span>':'<div class="fm-buttons">'+(!it.is_dir?'<button class="fm-action" data-download="'+fmH(it.path)+'">下载</button>':'')+'<button class="fm-action" data-rename="'+fmH(it.path)+'">重命名</button>'+(!it.is_dir&&fmEditable(it.name)?'<button class="fm-action" data-edit="'+fmH(it.path)+'">编辑</button>':'')+'<button class="fm-action danger" data-del="'+fmH(it.path)+'" data-isdir="'+(it.is_dir?"1":"0")+'">删除</button></div>';
                return '<div class="fm-row" data-path="'+fmH(it.path)+'"><input class="fm-check" type="checkbox" data-select="'+fmH(it.path)+'" aria-label="选择 '+fmH(it.name)+'"><span>'+icon+'</span>'+name+'<span class="fm-meta">'+meta+'</span>'+actions+'</div>';
            }).join("");
            list.querySelectorAll("[data-dir]").forEach(function(el){el.onclick=function(){fmLoad(el.dataset.dir);};});
            list.querySelectorAll("[data-del]").forEach(function(el){el.onclick=function(){fmDelete(el.dataset.del,el.dataset.isdir==="1");};});
            list.querySelectorAll("[data-rename]").forEach(function(el){el.onclick=function(){fmRename(el.dataset.rename);};});
            list.querySelectorAll("[data-edit]").forEach(function(el){el.onclick=function(){fmOpenEditor(el.dataset.edit);};});
            list.querySelectorAll("[data-download]").forEach(function(el){el.onclick=function(){window.location.href="/"+fmEnc(el.dataset.download);};});
            list.querySelectorAll("[data-select]").forEach(function(el){el.onchange=fmUpdateSelection;});
            fmUpdateSelection();
        }
        function fmEditable(name){return /\.(txt|md|markdown|json|ya?ml|toml|html?|css|js|ts|rs|py|sh|xml|env)$/i.test(name);}
        function fmUpdateSelection(){var n=document.querySelectorAll("#fmList [data-select]:checked").length;document.getElementById("fmSelectionCount").textContent="已选 "+n+" 项";document.getElementById("fmDeleteSelected").disabled=n===0;}
        async function fmLoad(path){
            fmPath=path||"";document.getElementById("fmPath").textContent="/"+fmPath;var list=document.getElementById("fmList");list.innerHTML='<div class="fm-empty">加载中...</div>';
            try{var res=await fetch("/@admin/files?path="+encodeURIComponent(fmPath));if(!res.ok)throw new Error("HTTP "+res.status);var data=await res.json();if(data.error)throw new Error(data.error);fmEntries=data.entries||[];document.getElementById("fmSelectAll").checked=false;fmRender();}
            catch(e){list.innerHTML='<div class="fm-empty">❌ 加载失败：'+fmH(e.message)+'</div>';}
        }
        async function fmPost(url){var res=await fetch(url,{method:"POST"});var data=await res.json();if(!res.ok||data.error)throw new Error(data.error||("HTTP "+res.status));return data;}
        async function fmDelete(path,isDir){var tip=isDir?"删除文件夹及其中全部内容？此操作不可恢复：\n"+path:"删除文件？\n"+path;if(!confirm(tip))return;try{await fmPost("/@admin/files/delete?path="+encodeURIComponent(path));fmMsg("已删除："+path);await fmLoad(fmPath);}catch(e){alert("删除失败："+e.message);}}
        async function fmMkdir(){var input=document.getElementById("fmNewDir"),name=input.value.trim();if(!name){fmMsg("请输入文件夹名称");return;}try{await fmPost("/@admin/files/mkdir?path="+encodeURIComponent(fmPath)+"&name="+encodeURIComponent(name));input.value="";fmMsg("已创建："+name);await fmLoad(fmPath);}catch(e){alert("创建失败："+e.message);}}
        async function fmRename(path){var old=path.split("/").pop(),name=prompt("输入新的名称",old);if(name===null||!name.trim()||name.trim()===old)return;try{await fmPost("/@admin/files/rename?path="+encodeURIComponent(path)+"&name="+encodeURIComponent(name.trim()));fmMsg("已重命名："+name.trim());await fmLoad(fmPath);}catch(e){alert("重命名失败："+e.message);}}
        async function fmUpload(){var input=document.getElementById("fmUploadInput"),files=Array.prototype.slice.call(input.files||[]);if(!files.length){fmMsg("请先选择文件");return;}var failed=0;for(var i=0;i<files.length;i++){var f=files[i];fmMsg("上传中 "+(i+1)+"/"+files.length+"："+f.name);try{var overwrite=document.getElementById("fmOverwrite").checked;var res=await fetch("/@admin/upload?dir="+encodeURIComponent(fmPath)+"&name="+encodeURIComponent(f.name)+"&overwrite="+(overwrite?"1":"0"),{method:"POST",body:f});var data=await res.json().catch(function(){return{error:"HTTP "+res.status};});if(!res.ok||data.error){failed++;fmMsg("上传失败："+f.name+"："+(data.error||res.status));}}catch(e){failed++;fmMsg("上传失败："+f.name+"："+e.message);}}input.value="";await fmLoad(fmPath);fmMsg(failed?"上传结束，失败 "+failed+" 个":"上传完成，共 "+files.length+" 个");}
        async function fmOpenEditor(path){fmEditingPath=path;document.getElementById("fmEditorTitle").textContent="编辑："+path;document.getElementById("fmEditorText").value="";document.getElementById("fmEditorStatus").textContent="读取中…";document.getElementById("fmEditor").hidden=false;try{var res=await fetch("/@admin/files/read?path="+encodeURIComponent(path));var data=await res.json();if(!res.ok||data.error)throw new Error(data.error||("HTTP "+res.status));document.getElementById("fmEditorText").value=data.content;document.getElementById("fmEditorStatus").textContent="UTF-8 文本 · 最多 2 MiB";}catch(e){document.getElementById("fmEditorStatus").textContent=e.message;}}
        async function fmSaveEditor(){var text=document.getElementById("fmEditorText").value;document.getElementById("fmEditorStatus").textContent="保存中…";try{var res=await fetch("/@admin/files/write?path="+encodeURIComponent(fmEditingPath),{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({content:text})});var data=await res.json();if(!res.ok||data.error)throw new Error(data.error||("HTTP "+res.status));document.getElementById("fmEditorStatus").textContent="已保存";await fmLoad(fmPath);}catch(e){document.getElementById("fmEditorStatus").textContent="保存失败："+e.message;}}
        function fmCloseEditor(){document.getElementById("fmEditor").hidden=true;}
        function fmUp(){if(fmPath)fmLoad(fmParent(fmPath));}
        document.getElementById("fmSearch").addEventListener("input",fmRender);document.getElementById("fmSort").addEventListener("change",fmRender);document.getElementById("fmFilter").addEventListener("change",fmRender);
        document.getElementById("fmSelectAll").addEventListener("change",function(){document.querySelectorAll("#fmList [data-select]").forEach(function(el){el.checked=this.checked;},this);fmUpdateSelection();});
        document.getElementById("fmDeleteSelected").addEventListener("click",async function(){var selected=Array.prototype.slice.call(document.querySelectorAll("#fmList [data-select]:checked")).map(function(el){return el.dataset.select;});if(!selected.length||!confirm("确认删除选中的 "+selected.length+" 项？文件夹内所有内容也会删除。"))return;var failed=0;for(var i=0;i<selected.length;i++){try{await fmPost("/@admin/files/delete?path="+encodeURIComponent(selected[i]));}catch(e){failed++;}}await fmLoad(fmPath);fmMsg(failed?"批量删除完成，失败 "+failed+" 项":"已删除 "+selected.length+" 项");});
        document.getElementById("fmNewDir").addEventListener("keydown",function(e){if(e.key==="Enter"){e.preventDefault();fmMkdir();}});
        document.addEventListener("keydown",function(e){if(e.key==="Escape")fmCloseEditor();});
        document.addEventListener("DOMContentLoaded",function(){fmLoad("");});
    </script>
</section>
"####;
    template.replace("__VERSION__", env!("CARGO_PKG_VERSION"))
}

use crate::config::Config;
use crate::html::home::html_escape;

/// 管理页面 HTML
pub fn generate_admin_html(config: &Config) -> String {
    let title = html_escape(&config.title);
    let bg_api = &config.background_api;

    let name_selected = if config.default_sort == "name" { "selected" } else { "" };
    let size_selected = if config.default_sort == "size" { "selected" } else { "" };
    let time_selected = if config.default_sort == "time" { "selected" } else { "" };
    let show_size_checked = if config.show_file_size { "checked" } else { "" };
    let show_admin_checked = if config.show_admin_btn { "checked" } else { "" };
    let clear_log_checked = if config.clear_log_on_start { "checked" } else { "" };

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>管理后台 - {}</title>
    <style>
        * {{ margin:0; padding:0; box-sizing:border-box; }}
        body {{
            font-family:'Segoe UI',Arial,sans-serif;
            background: url('{}') no-repeat center center fixed;
            background-size: cover;
            padding:20px;
            min-height:100vh;
        }}
        .container {{
            max-width:960px;
            margin:0 auto;
            background: rgba(255,255,255,0.85);
            border-radius:20px;
            padding:30px;
            backdrop-filter: blur(10px);
            box-shadow:0 20px 60px rgba(0,0,0,0.3);
            border:1px solid rgba(255,255,255,0.3);
        }}
        h1, h2 {{ color:#333; }}
        label {{ display:block; margin:10px 0 5px; font-weight:bold; color:#555; }}
        input, textarea, select {{ 
            width:100%; 
            padding:10px; 
            border:1px solid #ddd; 
            border-radius:8px; 
            background:rgba(255,255,255,0.8);
            transition: border-color 0.3s;
        }}
        input:focus, textarea:focus, select:focus {{
            border-color:#667eea;
            outline:none;
            box-shadow:0 0 0 3px rgba(102,126,234,0.1);
        }}
        .form-group {{ margin-bottom:15px; }}
        .btn {{ 
            padding:10px 24px; 
            border:none; 
            border-radius:25px; 
            background:linear-gradient(135deg,#667eea,#764ba2); 
            color:white; 
            cursor:pointer;
            font-size:14px;
            font-weight:500;
            transition:all 0.3s;
        }}
        .btn:hover {{ 
            transform:translateY(-2px);
            box-shadow:0 4px 15px rgba(102,126,234,0.4);
        }}
        .btn-secondary {{
            background:rgba(102,126,234,0.15);
            color:#667eea;
            border:1px solid rgba(102,126,234,0.2);
        }}
        .btn-secondary:hover {{
            background:rgba(102,126,234,0.25);
        }}
        .status-grid {{ 
            display:grid; 
            grid-template-columns:repeat(auto-fit, minmax(200px, 1fr)); 
            gap:15px; 
            margin:15px 0; 
        }}
        .status-item {{ 
            background:rgba(255,255,255,0.6); 
            padding:12px 16px; 
            border-radius:12px;
            border:1px solid rgba(102,126,234,0.1);
        }}
        .status-item .label {{ font-size:12px; color:#999; display:block; }}
        .status-item .value {{ font-size:20px; font-weight:600; color:#333; }}
        .exec-area {{
            margin: 15px 0;
        }}
        .exec-output {{
            background: #1e1e1e;
            color: #d4d4d4;
            padding: 10px;
            border-radius: 5px;
            font-family: monospace;
            font-size: 13px;
            max-height: 400px;
            overflow: auto;
            white-space: pre-wrap;
            margin-top: 10px;
        }}
        .logs {{ 
            background:rgba(0,0,0,0.05); 
            padding:15px; 
            border-radius:10px; 
            max-height:300px; 
            overflow-y:auto; 
            white-space:pre-wrap; 
            font-family:'Courier New',monospace; 
            font-size:12px; 
            line-height:1.6;
            color:#333;
        }}
        .nav-links {{ margin-bottom:20px; padding-bottom:15px; border-bottom:1px solid rgba(102,126,234,0.2); }}
        .nav-links a {{ color:#667eea; margin-right:20px; text-decoration:none; font-weight:500; }}
        .nav-links a:hover {{ text-decoration:underline; }}
        .checkbox-group {{ display:flex; align-items:center; gap:10px; }}
        .checkbox-group input[type="checkbox"] {{ width:auto; }}
        .checkbox-group label {{ margin:0; font-weight:400; }}
        .save-result {{ margin-top:15px; padding:10px; border-radius:8px; }}
        .save-result.success {{ background:rgba(16,185,129,0.1); color:#10b981; border:1px solid rgba(16,185,129,0.2); }}
        .save-result.error {{ background:rgba(239,68,68,0.1); color:#ef4444; border:1px solid rgba(239,68,68,0.2); }}
        hr {{ border:none; border-top:1px solid rgba(102,126,234,0.2); margin:25px 0; }}
        @media (max-width:768px) {{ .container {{ padding:20px; }} .status-grid {{ grid-template-columns:1fr 1fr; }} }}
        @media (max-width:480px) {{ .status-grid {{ grid-template-columns:1fr; }} }}
    </style>
</head>
<body>
    <div class="container">
        <div class="nav-links">
            <a href="/">← 返回主界面</a>
            <a href="/@admin" style="font-weight:bold;">⚙️ 管理后台</a>
        </div>
        <h1>⚙️ 管理后台</h1>

        <h2>📊 运行状态</h2>
        <div class="status-grid" id="statusGrid">
            <div class="status-item">
                <span class="label">进程 PID</span>
                <span class="value" id="pid">-</span>
            </div>
            <div class="status-item">
                <span class="label">内存使用</span>
                <span class="value" id="memory">- MB</span>
            </div>
            <div class="status-item">
                <span class="label">CPU 占用</span>
                <span class="value" id="cpu">- %</span>
            </div>
            <div class="status-item">
                <span class="label">运行时间</span>
                <span class="value" id="uptime">-</span>
            </div>
        </div>

        <hr>

        <h2>📝 编辑配置</h2>
        <form id="configForm">
            <div class="form-group">
                <label>标题</label>
                <input type="text" name="title" value="{}">
            </div>
            <div class="form-group">
                <label>描述</label>
                <input type="text" name="description" value="{}">
            </div>
            <div class="form-group">
                <label>背景图片 API</label>
                <input type="text" name="background_api" value="{}">
            </div>
            <div class="form-group">
                <label>透明度 (0.0 ~ 1.0)</label>
                <input type="number" step="0.01" min="0" max="1" name="opacity" value="{}">
            </div>
            <div class="form-group">
                <label>模糊程度 (例如 5px)</label>
                <input type="text" name="blur" value="{}">
            </div>
            <div class="form-group checkbox-group">
                <input type="checkbox" name="show_hidden" {} value="true">
                <label>显示隐藏文件（以 . 开头的文件）</label>
            </div>
            <div class="form-group">
                <label>首页文件名</label>
                <input type="text" name="index_file" value="{}">
                <small style="color:#999;">访问根目录时默认加载的文件</small>
            </div>
            <div class="form-group">
                <label>默认排序方式</label>
                <select name="default_sort">
                    <option value="name" {}>按名称</option>
                    <option value="size" {}>按大小</option>
                    <option value="time" {}>按时间</option>
                </select>
            </div>
            <div class="form-group checkbox-group">
                <input type="checkbox" name="show_file_size" {} value="true">
                <label>显示文件大小</label>
            </div>
            <div class="form-group checkbox-group">
                <input type="checkbox" name="show_admin_btn" {} value="true">
                <label>主页显示管理按钮</label>
            </div>
            <div class="form-group checkbox-group">
                <input type="checkbox" name="clear_log_on_start" {} value="true">
                <label>启动时自动清理日志</label>
            </div>
            <div class="form-group">
                <label>临时主页目录（留空则使用启动目录）</label>
                <input type="text" name="home_dir" value="{}" placeholder="例如: /storage/emulated/0/Downloads">
                <small style="color:#999;">仅修改网页显示，不改变实际文件路径，不显示在目录列表中</small>
            </div>
            <div class="form-group">
                <label>管理员用户名</label>
                <input type="text" name="admin_user" value="{}">
            </div>
            <div class="form-group">
                <label>管理员密码</label>
                <input type="password" name="admin_pass" value="{}">
            </div>
            <button type="submit" class="btn">💾 保存配置</button>
            <button type="button" class="btn btn-secondary" onclick="loadConfig()">🔄 重新加载</button>
        </form>
        <div id="saveResult"></div>

        <hr>

        <h2>📋 运行日志（最近 100 行）</h2>
        <div class="logs" id="logs">加载中...</div>
        <p style="margin-top:10px;">
            <button onclick="loadLogs()" class="btn btn-secondary">🔄 刷新日志</button>
            <span style="color:#999;font-size:13px;margin-left:15px;">日志文件: nweb.log</span>
        </p>

        <hr>

        <h2>💻 命令执行 (仅管理员)</h2>
        <div class="exec-area">
            <div class="form-group" style="display:flex; gap:10px;">
                <input type="text" id="cmdInput" placeholder="输入要执行的命令，例如: ls -la" style="flex:1;">
                <button id="execBtn" class="btn">▶ 执行</button>
                <button id="clearBtn" class="btn btn-secondary">清空输出</button>
            </div>
            <div id="execResult" class="exec-output">等待执行...</div>
        </div>
    </div>

    <script>
        // ========== 管理后台 JS ==========
        let term = null;
        let socket = null;
        let isConnected = false;

        // ========== 加载状态 ==========
        async function loadStatus() {{
            try {{
                const res = await fetch('/@admin/status');
                if (!res.ok) throw new Error('HTTP ' + res.status);
                const data = await res.json();
                document.getElementById('pid').textContent = data.pid || '-';
                document.getElementById('memory').textContent = (data.memory / 1024).toFixed(0) + ' MB' || '- MB';
                document.getElementById('cpu').textContent = data.cpu.toFixed(1) + ' %' || '- %';
                const uptime = data.uptime || 0;
                const hours = Math.floor(uptime / 3600);
                const minutes = Math.floor((uptime % 3600) / 60);
                const seconds = uptime % 60;
                document.getElementById('uptime').textContent = 
                    hours + 'h ' + minutes + 'm ' + seconds + 's';
            }} catch (e) {{
                console.error('加载状态失败:', e);
            }}
        }}

        // ========== 加载日志 ==========
        async function loadLogs() {{
            try {{
                const res = await fetch('/@admin/logs');
                if (!res.ok) throw new Error('HTTP ' + res.status);
                const data = await res.json();
                document.getElementById('logs').textContent = data.logs || '暂无日志';
            }} catch (e) {{
                document.getElementById('logs').textContent = '加载日志失败: ' + e.message;
                console.error('加载日志失败:', e);
            }}
        }}

        // ========== 加载配置 ==========
        async function loadConfig() {{
            try {{
                const res = await fetch('/@admin/config');
                if (!res.ok) throw new Error('HTTP ' + res.status);
                const config = await res.json();
                document.querySelector('[name="title"]').value = config.title || '';
                document.querySelector('[name="description"]').value = config.description || '';
                document.querySelector('[name="background_api"]').value = config.background_api || '';
                document.querySelector('[name="opacity"]').value = config.opacity || 0.3;
                document.querySelector('[name="blur"]').value = config.blur || '5px';
                document.querySelector('[name="show_hidden"]').checked = config.show_hidden || false;
                document.querySelector('[name="index_file"]').value = config.index_file || 'index.html';
                document.querySelector('[name="default_sort"]').value = config.default_sort || 'name';
                document.querySelector('[name="show_file_size"]').checked = config.show_file_size !== undefined ? config.show_file_size : true;
                document.querySelector('[name="show_admin_btn"]').checked = config.show_admin_btn !== undefined ? config.show_admin_btn : true;
                document.querySelector('[name="clear_log_on_start"]').checked = config.clear_log_on_start !== undefined ? config.clear_log_on_start : true;
                document.querySelector('[name="home_dir"]').value = config.home_dir || '';
                document.querySelector('[name="admin_user"]').value = config.admin_user || 'nweb';
                document.querySelector('[name="admin_pass"]').value = config.admin_pass || 'nweb';
                showResult('✅ 配置已重新加载', 'success');
            }} catch (e) {{
                showResult('❌ 加载配置失败: ' + e.message, 'error');
                console.error('加载配置失败:', e);
            }}
        }}

        // ========== 显示结果消息 ==========
        function showResult(message, type) {{
            const div = document.getElementById('saveResult');
            div.textContent = message;
            div.className = 'save-result ' + type;
            setTimeout(() => {{
                div.className = 'save-result';
                div.textContent = '';
            }}, 5000);
        }}

        // ========== 提交配置 ==========
        document.getElementById('configForm').addEventListener('submit', async (e) => {{
            e.preventDefault();
            const form = e.target;
            const formData = new FormData(form);
            const config = {{
                title: formData.get('title') || '',
                description: formData.get('description') || '',
                background_api: formData.get('background_api') || '',
                opacity: parseFloat(formData.get('opacity')) || 0.3,
                blur: formData.get('blur') || '5px',
                show_hidden: formData.has('show_hidden'),
                index_file: formData.get('index_file') || 'index.html',
                default_sort: formData.get('default_sort') || 'name',
                show_file_size: formData.has('show_file_size'),
                show_admin_btn: formData.has('show_admin_btn'),
                clear_log_on_start: formData.has('clear_log_on_start'),
                home_dir: formData.get('home_dir') || '',
                admin_user: formData.get('admin_user') || 'nweb',
                admin_pass: formData.get('admin_pass') || 'nweb',
            }};
            try {{
                const res = await fetch('/@admin/config', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(config),
                }});
                const result = await res.text();
                if (res.ok) {{
                    showResult('✅ ' + result, 'success');
                }} else {{
                    showResult('❌ ' + result, 'error');
                }}
            }} catch (e) {{
                showResult('❌ 请求失败: ' + e.message, 'error');
            }}
        }});

        // ========== 命令执行 ==========
        document.getElementById('execBtn').addEventListener('click', async function() {{
            const input = document.getElementById('cmdInput');
            const output = document.getElementById('execResult');
            const cmd = input.value.trim();
            if (!cmd) {{
                output.textContent = '请输入命令';
                return;
            }}
            output.textContent = '⏳ 执行中...';
            try {{
                const res = await fetch('/@admin/exec', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ cmd: cmd }}),
                }});
                const data = await res.json();
                let resultText = `退出码: ${{data.exit_code}}\n\n`;
                if (data.stdout) resultText += `[stdout]\n${{data.stdout}}\n`;
                if (data.stderr) resultText += `[stderr]\n${{data.stderr}}\n`;
                output.textContent = resultText || '(无输出)';
            }} catch (e) {{
                output.textContent = '❌ 请求失败: ' + e.message;
            }}
        }});

        document.getElementById('clearBtn').addEventListener('click', function() {{
            document.getElementById('execResult').textContent = '等待执行...';
            document.getElementById('cmdInput').value = '';
        }});

        // ========== 页面加载时初始化 ==========
        document.addEventListener('DOMContentLoaded', function() {{
            loadStatus();
            loadLogs();
            loadConfig();
            setInterval(loadStatus, 10000);
            setInterval(loadLogs, 30000);
        }});
    </script>
</body>
</html>"#,
        // 参数对应
        title, bg_api,
        config.title, config.description, config.background_api,
        config.opacity, config.blur,
        if config.show_hidden { "checked" } else { "" },
        config.index_file,
        name_selected, size_selected, time_selected,
        show_size_checked,
        show_admin_checked,
        clear_log_checked,
        config.home_dir,
        config.admin_user,
        config.admin_pass
    )
}
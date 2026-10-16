use crate::config::Config;

/// HTML 转义函数
pub fn html_escape(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#x27;")
}

/// 生成主页面 HTML
pub fn generate_index_html(config: &Config) -> String {
    let title = html_escape(&config.title);
    let description = html_escape(&config.description);
    let bg_api = &config.background_api;
    let opacity = config.opacity;
    let blur = &config.blur;
    let admin_btn_style = if config.show_admin_btn {
        "display:inline-block;"
    } else {
        "display:none;"
    };
    let show_file_size_js = if config.show_file_size { "true" } else { "false" };

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        /* ========== 全局重置 ========== */
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        html {{
            margin: 0;
            padding: 0;
            min-height: 100vh;
            height: 100%;
        }}
        body {{
            margin: 0;
            padding: 0;
            min-height: 100vh;
            height: 100%;
            font-family:'Segoe UI',Arial,sans-serif;
            background: url('{}') no-repeat center center fixed;
            background-size: cover;
            transition: background-color 0.3s, color 0.3s;
        }}
        .container {{
            max-width:960px;
            margin:30px auto;
            padding:30px;
            background: rgba(255, 255, 255, {});
            border-radius:20px;
            box-shadow:0 20px 60px rgba(0,0,0,0.3);
            backdrop-filter: blur({});
            -webkit-backdrop-filter: blur({});
            border:1px solid rgba(255,255,255,0.3);
            transition: background 0.3s, backdrop-filter 0.3s;
        }}
        .header {{
            display:flex;
            justify-content:space-between;
            align-items:center;
            flex-wrap:wrap;
            gap:15px;
            margin-bottom:20px;
            padding-bottom:15px;
            border-bottom:2px solid rgba(102,126,234,0.2);
        }}
        .header-left {{
            display:flex;
            align-items:center;
            gap:12px;
        }}
        .header-left h1 {{
            font-size:28px;
            font-weight:700;
            background:linear-gradient(135deg,#667eea,#764ba2);
            -webkit-background-clip:text;
            -webkit-text-fill-color:transparent;
            transition: background 0.3s;
        }}
        .btn {{
            padding:8px 18px;
            border:none;
            border-radius:25px;
            font-size:14px;
            font-weight:500;
            cursor:pointer;
            transition:all 0.3s;
            background:rgba(102,126,234,0.15);
            color:#667eea;
            border:1px solid rgba(102,126,234,0.2);
        }}
        .btn:hover {{
            transform:translateY(-2px);
            box-shadow:0 4px 12px rgba(102,126,234,0.3);
        }}
        .btn-primary {{
            background:linear-gradient(135deg,#667eea,#764ba2);
            color:white;
            border:none;
        }}
        .btn-primary:hover {{
            box-shadow:0 4px 15px rgba(102,126,234,0.4);
        }}
        .theme-btn {{
            background: rgba(255, 255, 255, 0.3);
            border: 1px solid rgba(255, 255, 255, 0.5);
            border-radius: 30px;
            padding: 8px 16px;
            cursor: pointer;
            font-size: 14px;
            backdrop-filter: blur(5px);
            transition: all 0.3s;
            color: #333;
        }}
        .theme-btn:hover {{
            background: rgba(255, 255, 255, 0.5);
            transform: scale(1.05);
        }}
        .project-info {{
            background: rgba(255, 255, 255, 0.5);
            border-radius:12px;
            padding:15px 20px;
            margin-bottom:15px;
            backdrop-filter: blur(5px);
            border: 1px solid rgba(255, 255, 255, 0.3);
            transition: background 0.3s, border-color 0.3s;
        }}
        .project-info p {{ margin:0; color:#555; line-height:1.6; transition: color 0.3s; }}
        .stats {{
            display:flex;
            gap:15px;
            flex-wrap:wrap;
            margin-bottom:20px;
        }}
        .stats-item {{
            display:flex;
            align-items:center;
            gap:6px;
            font-size:14px;
            color:#666;
            background:rgba(255,255,255,0.6);
            padding:4px 14px;
            border-radius:20px;
            border:1px solid rgba(255,255,255,0.3);
            transition: background 0.3s, color 0.3s, border-color 0.3s;
        }}
        .stats-item .num {{ font-weight:700; color:#667eea; transition: color 0.3s; }}
        .file-tree {{ margin-top:10px; }}
        .tree-root {{ list-style:none; padding-left:0; }}
        .tree-root li {{ margin:4px 0; list-style:none; }}
        .entry {{
            display:flex;
            align-items:center;
            justify-content:space-between;
            padding:10px 14px;
            border-radius:10px;
            transition:all 0.3s;
            text-decoration:none;
            color:#333;
            background:rgba(255,255,255,0.5);
            border:1px solid transparent;
            cursor:pointer;
        }}
        .entry:hover {{
            background:rgba(102,126,234,0.08);
            border-color:rgba(102,126,234,0.15);
            transform:translateX(4px);
        }}
        .folder-entry {{
            background:rgba(102,126,234,0.06);
            font-weight:600;
            border-left:4px solid #667eea;
        }}
        .file-entry {{
            background:rgba(255,255,255,0.3);
        }}
        .entry-left {{
            display:flex;
            align-items:center;
            gap:10px;
            flex:1;
            min-width:0;
        }}
        .entry-left .icon {{ font-size:20px; flex-shrink:0; }}
        .entry-left .name {{ word-break:break-word; font-size:15px; }}
        .file-info {{ color:#999; font-size:13px; flex-shrink:0; margin-left:10px; transition: color 0.3s; }}
        .children {{ padding-left:30px; }}
        .toggle-icon {{ transition:transform 0.3s; display:inline-block; margin-right:6px; color:#667eea; }}
        .toggle-icon.open {{ transform:rotate(90deg); }}
        .footer {{
            margin-top:30px;
            padding-top:20px;
            border-top:1px solid rgba(102,126,234,0.1);
            text-align:center;
            font-size:13px;
            color:#999;
            transition: border-color 0.3s, color 0.3s;
        }}
        .footer span {{ color:#667eea; transition: color 0.3s; }}
        .loading {{ text-align:center; padding:20px; color:#666; }}
        @keyframes spin {{ 100% {{ transform:rotate(360deg); }} }}
        .spinner {{ display:inline-block; width:20px; height:20px; border:2px solid rgba(102,126,234,0.2); border-top:2px solid #667eea; border-radius:50%; animation:spin 0.8s linear infinite; }}

        .back-to-top {{
            position: fixed;
            bottom: 30px;
            right: 30px;
            width: 50px;
            height: 50px;
            background: linear-gradient(135deg, #667eea, #764ba2);
            color: white;
            border: none;
            border-radius: 50%;
            font-size: 24px;
            cursor: pointer;
            box-shadow: 0 4px 15px rgba(0,0,0,0.3);
            transition: all 0.3s;
            display: none;
            z-index: 999;
        }}
        .back-to-top:hover {{
            transform: scale(1.1);
            box-shadow: 0 6px 20px rgba(102,126,234,0.4);
        }}
        .back-to-top.show {{ display: block; }}

        /* ========== 图片预览器 ========== */
        #imageViewerOverlay {{
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.92);
            z-index: 99999;
            display: flex;
            flex-direction: column;
            user-select: none;
        }}
        .viewer-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 12px 20px;
            background: rgba(0, 0, 0, 0.5);
            flex-shrink: 0;
            z-index: 2;
        }}
        .viewer-counter {{
            color: #fff;
            font-size: 14px;
            font-weight: 500;
        }}
        .viewer-controls {{
            display: flex;
            gap: 8px;
            align-items: center;
        }}
        .viewer-btn {{
            width: 36px;
            height: 36px;
            border: none;
            border-radius: 50%;
            background: rgba(255, 255, 255, 0.15);
            color: #fff;
            font-size: 16px;
            cursor: pointer;
            transition: all 0.3s;
            display: flex;
            align-items: center;
            justify-content: center;
        }}
        .viewer-btn:hover {{
            background: rgba(255, 255, 255, 0.3);
            transform: scale(1.05);
        }}
        .viewer-close {{
            background: rgba(255, 70, 70, 0.3);
            font-size: 18px;
        }}
        .viewer-close:hover {{
            background: rgba(255, 70, 70, 0.6);
        }}
        .viewer-body {{
            flex: 1;
            display: flex;
            align-items: center;
            justify-content: center;
            position: relative;
            overflow: hidden;
            cursor: zoom-out;
        }}
        .viewer-nav {{
            position: absolute;
            top: 50%;
            transform: translateY(-50%);
            width: 50px;
            height: 80px;
            border: none;
            background: rgba(0, 0, 0, 0.4);
            color: #fff;
            font-size: 32px;
            cursor: pointer;
            transition: all 0.3s;
            z-index: 3;
            border-radius: 4px;
        }}
        .viewer-nav:hover {{
            background: rgba(255, 255, 255, 0.2);
        }}
        .viewer-prev {{
            left: 20px;
        }}
        .viewer-next {{
            right: 20px;
        }}
        .viewer-image-wrapper {{
            display: flex;
            align-items: center;
            justify-content: center;
            width: 100%;
            height: 100%;
            padding: 20px;
        }}
        #viewerImage {{
            max-width: 100%;
            max-height: 100%;
            object-fit: contain;
            transition: transform 0.2s ease;
            box-shadow: 0 4px 30px rgba(0, 0, 0, 0.3);
        }}
        .viewer-footer {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 10px 20px;
            background: rgba(0, 0, 0, 0.4);
            flex-shrink: 0;
            z-index: 2;
        }}
        .viewer-footer span {{
            color: rgba(255, 255, 255, 0.8);
            font-size: 13px;
            font-family: monospace;
        }}
        body.night-mode #imageViewerOverlay {{
            background: rgba(10, 10, 10, 0.95);
        }}
        @media (max-width: 768px) {{
            .viewer-nav {{
                width: 36px;
                height: 60px;
                font-size: 24px;
            }}
            .viewer-prev {{ left: 8px; }}
            .viewer-next {{ right: 8px; }}
            .viewer-btn {{
                width: 32px;
                height: 32px;
                font-size: 14px;
            }}
            .viewer-header {{
                padding: 8px 12px;
            }}
            .viewer-counter {{
                font-size: 12px;
            }}
            .viewer-footer span {{
                font-size: 11px;
            }}
        }}

        /* ========== 夜间模式 ========== */
        html.night-mode,
        body.night-mode {{
            background: #1a1a1a;
            background-size: cover;
            background-position: center;
            min-height: 100vh;
            height: 100%;
            margin: 0;
            padding: 0;
        }}
        body.night-mode .container {{
            background: rgba(30, 30, 30, {});
            backdrop-filter: blur({}) saturate(180%);
            border-color: rgba(255, 255, 255, 0.1);
            color: #eee;
        }}
        body.night-mode .project-info {{
            background: rgba(40, 40, 40, 0.6);
            border-color: rgba(255, 255, 255, 0.1);
        }}
        body.night-mode .project-info p {{
            color: #eee;
        }}
        body.night-mode .stats-item {{
            background: rgba(50, 50, 50, 0.6);
            border-color: rgba(255, 255, 255, 0.1);
            color: #ccc;
        }}
        body.night-mode .stats-item .num {{
            color: #7ab0e0;
        }}
        body.night-mode .header-left h1 {{
            background: linear-gradient(135deg, #7ab0e0, #a78bfa);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }}
        body.night-mode .theme-btn {{
            background: rgba(50, 50, 50, 0.5);
            border-color: rgba(255, 255, 255, 0.2);
            color: #eee;
        }}
        body.night-mode .theme-btn:hover {{
            background: rgba(80, 80, 80, 0.6);
        }}
        body.night-mode .btn {{
            background: rgba(122, 176, 224, 0.2);
            color: #7ab0e0;
            border-color: rgba(122, 176, 224, 0.3);
        }}
        body.night-mode .btn-primary {{
            background: linear-gradient(135deg, #5a9acf, #8b6fe8);
            color: white;
        }}
        body.night-mode .tree-root .folder-entry {{
            background: rgba(40, 60, 80, 0.8);
            border-left-color: #5a9acf;
            color: #ddd;
        }}
        body.night-mode .tree-root .file-entry {{
            background: rgba(50, 50, 50, 0.8);
            color: #ccc;
        }}
        body.night-mode .tree-root .entry:hover {{
            background: rgba(122, 176, 224, 0.15);
            border-color: rgba(122, 176, 224, 0.3);
        }}
        body.night-mode .file-info {{
            color: #aaa;
        }}
        body.night-mode .footer {{
            border-top-color: rgba(255, 255, 255, 0.1);
            color: #888;
        }}
        body.night-mode .footer span {{
            color: #7ab0e0;
        }}
        body.night-mode .entry {{
            color: #ddd;
            background: rgba(40, 40, 40, 0.6);
        }}
        body.night-mode .entry:hover {{
            background: rgba(122, 176, 224, 0.15);
        }}
        body.night-mode .project-info h2 {{
            color: #eee;
        }}
        body.night-mode .loading {{
            color: #999;
        }}
        body.night-mode .back-to-top {{
            background: linear-gradient(135deg, #5a9acf, #8b6fe8);
        }}

        @media (max-width:768px) {{
            .container {{ margin:15px; padding:20px; }}
            .header-left h1 {{ font-size:22px; }}
            .stats {{ gap:8px; }}
            .stats-item {{ font-size:12px; padding:3px 10px; }}
            .entry {{ padding:8px 12px; font-size:14px; }}
            .entry-left .icon {{ font-size:17px; }}
            .file-info {{ font-size:11px; }}
            .children {{ padding-left:20px; }}
            .back-to-top {{ bottom:20px; right:20px; width:45px; height:45px; font-size:20px; }}
        }}
        @media (max-width:480px) {{
            .container {{ margin:10px; padding:15px; }}
            .header {{ flex-direction:column; align-items:stretch; }}
            .header-right {{ justify-content:flex-start; flex-wrap:wrap; }}
            .btn {{ padding:6px 14px; font-size:13px; }}
            .children {{ padding-left:15px; }}
            .back-to-top {{ bottom:15px; right:15px; width:40px; height:40px; font-size:18px; }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="header-left">
                <h1>📁 {}</h1>
            </div>
            <div class="header-right">
                <button class="theme-btn" id="themeToggle">🌙 夜间模式</button>
                <button class="btn" onclick="window.location.reload()">🔄 刷新</button>
                <button class="btn btn-primary" onclick="collapseAll()">📁 全部折叠</button>
                <button class="btn" onclick="window.location.href='/@admin'" id="adminBtn" style="{};">
                    ⚙️ 管理
                </button>
            </div>
        </div>
        <div class="project-info">
            <p>{}</p>
        </div>
        <div class="stats" id="stats">
            <span class="stats-item" id="fileCount">📄 文件 <span class="num">0</span></span>
            <span class="stats-item" id="dirCount">📁 目录 <span class="num">0</span></span>
        </div>
        <div class="file-tree">
            <div id="treeRoot" class="tree-root"></div>
        </div>
        <div class="footer">
            <p>⚡ 由 <span>Rust</span> 强力驱动 · 快速文件浏览</p>
        </div>
    </div>

    <!-- 返回顶部按钮 -->
    <button id="backToTop" class="back-to-top" onclick="scrollToTop()">⬆</button>

    <script>
        // ========== 从后端传入配置 ==========
        const showFileSize = {};

        // ========== 夜间模式 ==========
        (function() {{
            const themeToggle = document.getElementById('themeToggle');
            const html = document.documentElement;
            const body = document.body;

            const savedTheme = localStorage.getItem('nweb_theme');
            if (savedTheme === 'night') {{
                html.classList.add('night-mode');
                body.classList.add('night-mode');
                themeToggle.textContent = '☀️ 日间模式';
            }} else {{
                themeToggle.textContent = '🌙 夜间模式';
            }}

            themeToggle.addEventListener('click', function() {{
                html.classList.toggle('night-mode');
                body.classList.toggle('night-mode');
                const isNight = html.classList.contains('night-mode');
                themeToggle.textContent = isNight ? '☀️ 日间模式' : '🌙 夜间模式';
                localStorage.setItem('nweb_theme', isNight ? 'night' : 'day');
            }});
        }})();

        // ========== 文件浏览 ==========
        let loadedPaths = new Set();
        let loadingPaths = new Set();
        let totalFiles = 0;
        let totalDirs = 0;

        // ========== 图片预览器 ==========
        let imageViewerData = [];
        let imageViewerIndex = 0;
        let imageViewerZoom = 1;
        let imageViewerRotate = 0;

        function openImageViewer(imagePath) {{
            var images = [];
            var items = document.querySelectorAll('.entry.file-entry');
            for (var i = 0; i < items.length; i++) {{
                var el = items[i];
                var name = el.querySelector('.name')?.textContent || '';
                if (name && /\\.(jpg|jpeg|png|gif|bmp|webp|svg|ico)$/i.test(name)) {{
                    var dataImg = el.getAttribute('data-img');
                    if (dataImg) {{
                        images.push(dataImg);
                    }} else {{
                        var href = el.getAttribute('href');
                        if (href && href.startsWith('/')) {{
                            images.push(href);
                        }}
                    }}
                }}
            }}

            var uniqueImages = [];
            for (var j = 0; j < images.length; j++) {{
                if (uniqueImages.indexOf(images[j]) === -1) {{
                    uniqueImages.push(images[j]);
                }}
            }}

            if (uniqueImages.length === 0) {{
                imageViewerData = [imagePath];
                imageViewerIndex = 0;
            }} else {{
                var index = uniqueImages.indexOf(imagePath);
                imageViewerData = uniqueImages;
                imageViewerIndex = index >= 0 ? index : 0;
            }}

            imageViewerZoom = 1;
            imageViewerRotate = 0;
            showImageViewer();
        }}

        function showImageViewer() {{
            var existing = document.getElementById('imageViewerOverlay');
            if (existing) {{
                existing.remove();
            }}

            if (imageViewerData.length === 0) return;

            var overlay = document.createElement('div');
            overlay.id = 'imageViewerOverlay';
            overlay.innerHTML =
                '<div class=\"viewer-header\">' +
                '<span class=\"viewer-counter\">' + (imageViewerIndex + 1) + ' / ' + imageViewerData.length + '</span>' +
                '<div class=\"viewer-controls\">' +
                '<button class=\"viewer-btn\" onclick=\"imageViewerPrev()\" title=\"上一张\">◀</button>' +
                '<button class=\"viewer-btn\" onclick=\"imageViewerNext()\" title=\"下一张\">▶</button>' +
                '<button class=\"viewer-btn\" onclick=\"imageViewerZoomIn()\" title=\"放大\">➕</button>' +
                '<button class=\"viewer-btn\" onclick=\"imageViewerZoomOut()\" title=\"缩小\">➖</button>' +
                '<button class=\"viewer-btn\" onclick=\"imageViewerRotateImage()\" title=\"旋转\">🔄</button>' +
                '<button class=\"viewer-btn viewer-close\" onclick=\"closeImageViewer()\" title=\"关闭\">✕</button>' +
                '</div>' +
                '</div>' +
                '<div class=\"viewer-body\" id=\"viewerBody\">' +
                '<button class=\"viewer-nav viewer-prev\" onclick=\"imageViewerPrev()\">‹</button>' +
                '<div class=\"viewer-image-wrapper\">' +
                '<img id=\"viewerImage\" src=\"' + imageViewerData[imageViewerIndex] + '\" alt=\"图片预览\" />' +
                '</div>' +
                '<button class=\"viewer-nav viewer-next\" onclick=\"imageViewerNext()\">›</button>' +
                '</div>' +
                '<div class=\"viewer-footer\">' +
                '<span id=\"viewerFilename\">' + imageViewerData[imageViewerIndex].split('/').pop() + '</span>' +
                '<span id=\"viewerZoomInfo\">100%</span>' +
                '</div>';
            document.body.appendChild(overlay);

            document.addEventListener('keydown', imageViewerKeyHandler);

            document.getElementById('viewerBody').addEventListener('click', function(e) {{
                if (e.target === this) {{
                    closeImageViewer();
                }}
            }});

            document.getElementById('viewerImage').addEventListener('wheel', function(e) {{
                e.preventDefault();
                if (e.deltaY < 0) {{
                    imageViewerZoomIn();
                }} else {{
                    imageViewerZoomOut();
                }}
            }});

            updateViewerImage();
        }}

        function updateViewerImage() {{
            var img = document.getElementById('viewerImage');
            if (!img || imageViewerData.length === 0) return;

            img.src = imageViewerData[imageViewerIndex];
            img.style.transform = 'scale(' + imageViewerZoom + ') rotate(' + imageViewerRotate + 'deg)';

            var counter = document.querySelector('.viewer-counter');
            var filename = document.getElementById('viewerFilename');
            var zoomInfo = document.getElementById('viewerZoomInfo');

            if (counter) counter.textContent = (imageViewerIndex + 1) + ' / ' + imageViewerData.length;
            if (filename) filename.textContent = imageViewerData[imageViewerIndex].split('/').pop();
            if (zoomInfo) zoomInfo.textContent = Math.round(imageViewerZoom * 100) + '%';
        }}

        function imageViewerPrev() {{
            if (imageViewerData.length <= 1) return;
            imageViewerIndex = (imageViewerIndex - 1 + imageViewerData.length) % imageViewerData.length;
            imageViewerZoom = 1;
            imageViewerRotate = 0;
            updateViewerImage();
        }}

        function imageViewerNext() {{
            if (imageViewerData.length <= 1) return;
            imageViewerIndex = (imageViewerIndex + 1) % imageViewerData.length;
            imageViewerZoom = 1;
            imageViewerRotate = 0;
            updateViewerImage();
        }}

        function imageViewerZoomIn() {{
            imageViewerZoom = Math.min(imageViewerZoom + 0.1, 3);
            updateViewerImage();
        }}

        function imageViewerZoomOut() {{
            imageViewerZoom = Math.max(imageViewerZoom - 0.1, 0.1);
            updateViewerImage();
        }}

        function imageViewerRotateImage() {{
            imageViewerRotate = (imageViewerRotate + 90) % 360;
            updateViewerImage();
        }}

        function imageViewerKeyHandler(e) {{
            if (!document.getElementById('imageViewerOverlay')) return;

            switch(e.key) {{
                case 'Escape':
                    closeImageViewer();
                    break;
                case 'ArrowLeft':
                    e.preventDefault();
                    imageViewerPrev();
                    break;
                case 'ArrowRight':
                    e.preventDefault();
                    imageViewerNext();
                    break;
                case '+':
                case '=':
                    e.preventDefault();
                    imageViewerZoomIn();
                    break;
                case '-':
                case '_':
                    e.preventDefault();
                    imageViewerZoomOut();
                    break;
                case 'r':
                case 'R':
                    e.preventDefault();
                    imageViewerRotateImage();
                    break;
            }}
        }}

        function closeImageViewer() {{
            document.removeEventListener('keydown', imageViewerKeyHandler);
            var overlay = document.getElementById('imageViewerOverlay');
            if (overlay) {{
                overlay.remove();
            }}
        }}

        document.addEventListener('DOMContentLoaded', function() {{
            loadDirectory('');
            window.addEventListener('scroll', function() {{
                var btn = document.getElementById('backToTop');
                if (window.scrollY > 300) {{
                    btn.classList.add('show');
                }} else {{
                    btn.classList.remove('show');
                }}
            }});
        }});

        function scrollToTop() {{
            window.scrollTo({{ top: 0, behavior: 'smooth' }});
        }}

        async function loadDirectory(path, containerId = null) {{
            var container = containerId ? document.getElementById(containerId) : document.getElementById('treeRoot');
            if (loadedPaths.has(path) || loadingPaths.has(path)) return;
            loadingPaths.add(path);

            var loader = document.createElement('div');
            loader.className = 'loading';
            loader.innerHTML = '<div class=\"spinner\"></div><p>加载中...</p>';
            container.appendChild(loader);

            try {{
                var cleanPath = path.replace(/^\/+|\/+$/g, '');
                var response = await fetch('/api/tree/' + encodeURIComponent(cleanPath));
                if (!response.ok) {{
                    var errText = await response.text();
                    throw new Error(errText || 'HTTP ' + response.status);
                }}
                var data = await response.json();

                loader.remove();
                loadingPaths.delete(path);
                loadedPaths.add(path);

                var ul = document.createElement('ul');
                ul.className = 'children';
                ul.id = 'children-' + path.replace(/[^a-zA-Z0-9]/g, '_');

                var filesInDir = 0;
                var dirsInDir = 0;

                data.forEach(function(node) {{
                    var li = document.createElement('li');

                    if (node.is_dir) {{
                        dirsInDir++;
                        var entry = document.createElement('div');
                        entry.className = 'entry folder-entry';
                        var toggleSpan = document.createElement('span');
                        toggleSpan.className = 'toggle-icon';
                        toggleSpan.innerHTML = '▶';
                        var entryLeft = document.createElement('span');
                        entryLeft.className = 'entry-left';
                        entryLeft.innerHTML =
                            '<span class=\"icon\">📁</span>' +
                            '<span class=\"name\">' + node.name + '</span>';
                        entry.appendChild(toggleSpan);
                        entry.appendChild(entryLeft);

                        var fileInfo = document.createElement('span');
                        fileInfo.className = 'file-info';
                        fileInfo.textContent = '目录';
                        entry.appendChild(fileInfo);

                        var childrenContainer = document.createElement('div');
                        childrenContainer.className = 'children';
                        childrenContainer.style.display = 'none';
                        childrenContainer.id = 'children-container-' + node.path.replace(/[^a-zA-Z0-9]/g, '_');

                        entry.onclick = function(e) {{
                            e.stopPropagation();
                            var isOpen = childrenContainer.style.display !== 'none';
                            if (isOpen) {{
                                childrenContainer.style.display = 'none';
                                toggleSpan.className = 'toggle-icon';
                                toggleSpan.innerHTML = '▶';
                            }} else {{
                                childrenContainer.style.display = 'block';
                                toggleSpan.className = 'toggle-icon open';
                                toggleSpan.innerHTML = '▼';
                                var childId = 'children-' + node.path.replace(/[^a-zA-Z0-9]/g, '_');
                                if (!document.getElementById(childId)) {{
                                    loadDirectory(node.path, childrenContainer.id);
                                }}
                            }}
                        }};

                        li.appendChild(entry);
                        li.appendChild(childrenContainer);
                    }} else {{
                        filesInDir++;
                        var entry = document.createElement('a');
                        entry.className = 'entry file-entry';
                        var filePath = node.path || node.name;
                        var isImage = /\\.(jpg|jpeg|png|gif|bmp|webp|svg|ico)$/i.test(node.name);

                        var sizeHtml = '';
                        if (showFileSize) {{
                            sizeHtml = '<span class=\"file-info\">' + formatSize(node.size) + '</span>';
                        }}

                        if (isImage) {{
                            var imgPath = '/' + filePath;
                            entry.href = '#';
                            entry.setAttribute('data-img', imgPath);
                            (function(path) {{
                                entry.onclick = function(e) {{
                                    e.preventDefault();
                                    e.stopPropagation();
                                    openImageViewer(path);
                                }};
                            }})(imgPath);
                            entry.innerHTML =
                                '<span class=\"entry-left\">' +
                                '<span class=\"icon\">' + getFileIcon(node.name) + '</span>' +
                                '<span class=\"name\">' + node.name + '</span>' +
                                '</span>' +
                                sizeHtml +
                                '<span class=\"file-info\" style=\"color:#667eea;\">🖼️ 预览</span>';
                        }} else {{
                            entry.href = '/' + filePath;
                            entry.target = '_blank';
                            entry.innerHTML =
                                '<span class=\"entry-left\">' +
                                '<span class=\"icon\">' + getFileIcon(node.name) + '</span>' +
                                '<span class=\"name\">' + node.name + '</span>' +
                                '</span>' +
                                sizeHtml;
                        }}
                        li.appendChild(entry);
                    }}
                    ul.appendChild(li);
                }});

                if (data.length === 0) {{
                    var emptyLi = document.createElement('li');
                    emptyLi.style.color = '#999';
                    emptyLi.style.padding = '10px';
                    emptyLi.textContent = '雜鱼 这目录是空的';
                    ul.appendChild(emptyLi);
                }}

                container.innerHTML = '';
                container.appendChild(ul);

                totalFiles += filesInDir;
                totalDirs += dirsInDir;
                document.getElementById('fileCount').querySelector('.num').textContent = totalFiles;
                document.getElementById('dirCount').querySelector('.num').textContent = totalDirs;

            }} catch (error) {{
                loader.innerHTML = '<p style=\"color:#ef4444;\">❌ 加载失败: ' + error.message + '</p>';
                loadingPaths.delete(path);
                console.error('加载失败:', error);
            }}
        }}

        function getFileIcon(filename) {{
            var ext = filename.split('.').pop().toLowerCase();
            var icons = {{
                'html': '🌐', 'htm': '🌐',
                'css': '🎨',
                'js': '⚡', 'jsx': '⚡', 'ts': '⚡', 'tsx': '⚡',
                'rs': '🦀',
                'py': '🐍',
                'go': '🐹',
                'java': '☕',
                'c': '⚙️', 'cpp': '⚙️', 'h': '⚙️', 'hpp': '⚙️',
                'json': '📋',
                'xml': '📝', 'yaml': '📝', 'yml': '📝', 'toml': '📝',
                'md': '📖',
                'txt': '📄', 'log': '📄',
                'pdf': '📕',
                'jpg': '🖼️', 'jpeg': '🖼️', 'png': '🖼️', 'gif': '🖼️', 'svg': '🖼️', 'webp': '🖼️',
                'mp4': '🎬', 'avi': '🎬', 'mov': '🎬', 'mkv': '🎬',
                'mp3': '🎵', 'wav': '🎵', 'flac': '🎵',
                'zip': '📦', 'rar': '📦', '7z': '📦', 'tar': '📦', 'gz': '📦',
                'exe': '📦', 'msi': '📦', 'dmg': '📦', 'apk': '📦',
                'sh': '📃', 'bash': '📃', 'zsh': '📃', 'rb': '📃', 'fish': '📃',
                'dockerfile': '🐳',
                'gitignore': '📌', 'gitattributes': '📌',
                'lock': '🔒',
                'wasm': '⚙️',
                'proto': '📡',
            }};
            return icons[ext] || '📄';
        }}

        function formatSize(bytes) {{
            if (bytes === 0) return '0 B';
            var units = ['B', 'KB', 'MB', 'GB', 'TB'];
            var k = 1024;
            var i = Math.floor(Math.log(bytes) / Math.log(k));
            var size = (bytes / Math.pow(k, i)).toFixed(i > 0 ? 1 : 0);
            return size + ' ' + units[i];
        }}

        function collapseAll() {{
            var items = document.querySelectorAll('.children:not(:first-child)');
            for (var i = 0; i < items.length; i++) {{
                items[i].style.display = 'none';
            }}
            var icons = document.querySelectorAll('.toggle-icon');
            for (var j = 0; j < icons.length; j++) {{
                icons[j].className = 'toggle-icon';
                icons[j].innerHTML = '▶';
            }}
        }}
    </script>
</body>
</html>"#,
        // 参数对应
        title, bg_api, opacity, blur, blur, opacity, blur, title, admin_btn_style, description,
        show_file_size_js
    )
}
   // ===== 国际化 (中文/英文) =====
   const I18N = {
       "zh": {
           "theme-auto": "自动", "theme-system": "跟随系统", "theme-light": "白天", "theme-dark": "夜间",
           "lang-label": "中文",
           "bg": "背景", "bg-title": "更换背景", "bg-choose": "📁 选择图片", "bg-opacity-label": "透明度",
           "bg-card-trans": "卡片透明", "bg-remove": "移除背景",
           "devices": "设备", "online": "在线", "offline": "离线",
           "search-placeholder": "搜索设备...",
           "online-devices": "在线设备", "offline-devices": "离线设备",
           "os-label": "系统", "domain-label": "域名", "status-label": "状态", "last-seen-label": "最后在线",
           "self-label": "(本机)",
           "ping-btn": "测试连通性",
           "bg-file-selected": "已选择", "bg-set": "背景已设置", "bg-too-large": "图片过大，存储失败。请选择更小的图片。",
           "copy-ok": "已复制", "ping-testing": "测试中...", "ping-ok": "可达, 延迟", "ping-fail": "不可达", "ping-error": "测试失败",
           "error-title": "无法连接 Tailscale", "error-desc": "请确认宿主机的 tailscaled 正在运行，且 socket 已挂载到容器。"
       },
       "en": {
           "theme-auto": "Auto", "theme-system": "System", "theme-light": "Light", "theme-dark": "Dark",
           "lang-label": "English",
           "bg": "Background", "bg-title": "Change Background", "bg-choose": "📁 Choose Image", "bg-opacity-label": "Opacity",
           "bg-card-trans": "Transparent Cards", "bg-remove": "Remove",
           "devices": "Devices", "online": "Online", "offline": "Offline",
           "search-placeholder": "Search devices...",
           "online-devices": "Online Devices", "offline-devices": "Offline Devices",
           "os-label": "OS", "domain-label": "Domain", "status-label": "Status", "last-seen-label": "Last seen",
           "self-label": "(this device)",
           "ping-btn": "Test Connectivity",
           "bg-file-selected": "Selected", "bg-set": "Background set", "bg-too-large": "Image too large. Please choose a smaller one.",
           "copy-ok": "Copied", "ping-testing": "Testing...", "ping-ok": "Reachable,", "ping-fail": "Unreachable", "ping-error": "Test failed",
           "error-title": "Cannot connect to Tailscale", "error-desc": "Please ensure tailscaled is running on the host and the socket is mounted into the container."
       },
       "ru": {
           "theme-auto": "Авто", "theme-system": "Как в системе", "theme-light": "Светлая", "theme-dark": "Тёмная",
           "lang-label": "Русский",
           "bg": "Фон", "bg-title": "Сменить фон", "bg-choose": "📁 Выбрать изображение", "bg-opacity-label": "Прозрачность",
           "bg-card-trans": "Прозрачные карточки", "bg-remove": "Удалить",
           "devices": "Устройства", "online": "Онлайн", "offline": "Офлайн",
           "search-placeholder": "Поиск устройств...",
           "online-devices": "Онлайн устройства", "offline-devices": "Офлайн устройства",
           "os-label": "ОС", "domain-label": "Домен", "status-label": "Статус", "last-seen-label": "Был в сети",
           "self-label": "(это устройство)",
           "ping-btn": "Проверить связь",
           "bg-file-selected": "Выбрано", "bg-set": "Фон установлен", "bg-too-large": "Изображение слишком большое. Выберите меньшее.",
           "copy-ok": "Скопировано", "ping-testing": "Проверка...", "ping-ok": "Доступно, задержка", "ping-fail": "Недоступно", "ping-error": "Ошибка проверки",
           "error-title": "Нет подключения к Tailscale", "error-desc": "Убедитесь, что tailscaled запущен на хосте и socket подключён к контейнеру."
       }
   };

   function detectBrowserLang() {
       // 浏览器语言: zh/zh-CN/zh-TW → 中文; ru/RU → 俄语; 其他 → 英文
       const navLang = (navigator.language || navigator.userLanguage || "en").toLowerCase();
       if (navLang.startsWith("zh")) return "zh";
       if (navLang.startsWith("ru")) return "ru";
       return "en";
   }

   function currentLang() {
       // 用户手动选择过 → 用选择; 否则按浏览器语言
       const saved = localStorage.getItem("ts-lang");
       if (saved === "zh" || saved === "en" || saved === "ru") return saved;
       return detectBrowserLang();
   }

   function t(key) {
       const lang = currentLang();
       return (I18N[lang] && I18N[lang][key]) || (I18N["zh"][key]) || key;
   }

   function applyLang() {
       const lang = currentLang();
       // 文本元素
       document.querySelectorAll("[data-i18n]").forEach(el => {
           el.textContent = t(el.getAttribute("data-i18n"));
       });
       // placeholder
       document.querySelectorAll("[data-i18n-placeholder]").forEach(el => {
           el.setAttribute("placeholder", t(el.getAttribute("data-i18n-placeholder")));
       });
       // 语言按钮文字
       const langLabel = document.getElementById("langLabel");
       if (langLabel) langLabel.textContent = t("lang-label");
       document.documentElement.lang = lang === "zh" ? "zh-CN" : lang === "ru" ? "ru" : "en";
   }

   // ===== 主题切换 (跟随系统/白天/夜间/自动按时间 四态循环) =====
   const THEME_ICONS = {
       system: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zm0 2v16a8 8 0 0 1 0-16z"/></svg>',
       light: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 7a5 5 0 1 0 0 10 5 5 0 0 0 0-10zm0 3a2 2 0 1 1 0 4 2 2 0 0 1 0-4zM11 2h2v3h-2V2zm0 17h2v3h-2v-3zM2 11h3v2H2v-2zm17 0h3v2h-3v-2zM4.2 4.2l2.1 2.1-1.4 1.4-2.1-2.1 1.4-1.4zm12.9 12.9l2.1 2.1-1.4 1.4-2.1-2.1 1.4-1.4zM4.2 19.8l1.4-1.4 2.1 2.1-1.4 1.4-2.1-2.1zm14.7-14.7l-1.4 1.4-2.1-2.1 1.4-1.4 2.1 2.1z"/></svg>',
       dark: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M12.1 3a9 9 0 1 0 8.9 11 7.5 7.5 0 0 1-8.9-11z"/></svg>',
       auto: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zm1 2.1v15.8A8 8 0 0 1 13 4.1z"/></svg>'
   };
   const THEME_LABELS = { system: "跟随系统", light: "白天", dark: "夜间", auto: "自动" };

   // 夜间时段: 19:00 - 6:00 (按设备本地时间)
   function isNightTime() {
       const h = new Date().getHours();
       return h >= 19 || h < 6;
   }

   function resolveTheme(mode) {
       if (mode === "auto") {
           return isNightTime() ? "dark" : "light";
       }
       if (mode === "system") {
           return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
       }
       return mode;
   }

   function applyTheme(mode) {
       const resolved = resolveTheme(mode);
       document.documentElement.setAttribute("data-theme", resolved);
       document.getElementById("themeIcon").innerHTML = THEME_ICONS[mode];
       document.getElementById("themeLabel").textContent = t("theme-" + mode);
       localStorage.setItem("ts-theme", mode);
   }

   function toggleThemeMenu() {
       const menu = document.getElementById("themeMenu");
       const langMenu = document.getElementById("langMenu");
       const bgPanel = document.getElementById("bgPanel");
       bgPanel.classList.remove("show");
       langMenu.classList.remove("show");
       menu.classList.toggle("show");
   }

   function selectTheme(mode) {
       applyTheme(mode);
       document.getElementById("themeMenu").classList.remove("show");
   }

   function toggleLangMenu() {
       const menu = document.getElementById("langMenu");
       const themeMenu = document.getElementById("themeMenu");
       const bgPanel = document.getElementById("bgPanel");
       bgPanel.classList.remove("show");
       themeMenu.classList.remove("show");
       menu.classList.toggle("show");
   }

   function selectLang(lang) {
       localStorage.setItem("ts-lang", lang);
       applyLang();
       document.getElementById("langMenu").classList.remove("show");
   }

   // 点击外部关闭菜单
   document.addEventListener("click", (e) => {
       const themeMenu = document.getElementById("themeMenu");
       const langMenu = document.getElementById("langMenu");
       const themeBtn = document.getElementById("themeBtn");
       const langBtn = document.getElementById("langBtn");
       if (themeMenu.classList.contains("show") && !themeMenu.contains(e.target) && !themeBtn.contains(e.target)) {
           themeMenu.classList.remove("show");
       }
       if (langMenu.classList.contains("show") && !langMenu.contains(e.target) && !langBtn.contains(e.target)) {
           langMenu.classList.remove("show");
       }
   });

   // 初始化语言
   applyLang();

   // 初始化主题
   applyTheme(localStorage.getItem("ts-theme") || "auto");

   // ===== 背景图片功能 =====
   function loadBg() {
       const bg = localStorage.getItem("ts-bg");
       const opacity = localStorage.getItem("ts-bg-opacity");
       const layer = document.getElementById("bgLayer");
       const overlay = document.getElementById("bgOverlay");

       if (bg) {
           layer.style.backgroundImage = "url(" + bg + ")";
       } else {
           layer.style.backgroundImage = "none";
       }

       const op = opacity ? parseInt(opacity) : 85;
       document.documentElement.style.setProperty("--bg-opacity", (op / 100).toFixed(2));
       const slider = document.getElementById("bgOpacity");
       if (slider) slider.value = op;
       const label = document.getElementById("bgOpacityLabel");
       if (label) label.textContent = op + "%";
   }

   function toggleBgPanel() {
       const panel = document.getElementById("bgPanel");
       // 打开背景面板时关闭菜单
       if (!panel.classList.contains("show")) {
           document.getElementById("themeMenu").classList.remove("show");
           document.getElementById("langMenu").classList.remove("show");
       }
       panel.classList.toggle("show");
   }

   // 点击面板外部关闭背景面板
   document.addEventListener("click", (e) => {
       const panel = document.getElementById("bgPanel");
       const btn = document.getElementById("bgBtn");
       if (panel.classList.contains("show") && !panel.contains(e.target) && !btn.contains(e.target)) {
           panel.classList.remove("show");
       }
   });

   function handleBgFile(input) {
       const file = input.files[0];
       if (!file) return;

       // 显示文件名
       const nameEl = document.getElementById("bgFileName");
       if (nameEl) nameEl.textContent = t("bg-file-selected") + ": " + file.name;

       // 压缩图片 (限制 localStorage 大小, 最长边 1920px, JPEG 质量 0.8)
       const reader = new FileReader();
       reader.onload = function (e) {
           const img = new Image();
           img.onload = function () {
               const maxSize = 1920;
               let w = img.width, h = img.height;
               if (w > maxSize || h > maxSize) {
                   const ratio = Math.min(maxSize / w, maxSize / h);
                   w = Math.round(w * ratio);
                   h = Math.round(h * ratio);
               }
               const canvas = document.createElement("canvas");
               canvas.width = w;
               canvas.height = h;
               const ctx = canvas.getContext("2d");
               ctx.drawImage(img, 0, 0, w, h);
               const dataUrl = canvas.toDataURL("image/jpeg", 0.8);

               try {
                   localStorage.setItem("ts-bg", dataUrl);
                   loadBg();
                   // 设置成功后关闭面板
                   document.getElementById("bgPanel").classList.remove("show");
                   showToast(t("bg-set"));
               } catch (err) {
                   showToast(t("bg-too-large"));
               }
           };
           img.src = e.target.result;
       };
       reader.readAsDataURL(file);
       input.value = "";
   }

   function toggleCardTrans(checkbox) {
       document.body.classList.toggle("card-trans", checkbox.checked);
       localStorage.setItem("ts-card-trans", checkbox.checked ? "1" : "0");
   }

   function loadCardTrans() {
       const on = localStorage.getItem("ts-card-trans") === "1";
       document.body.classList.toggle("card-trans", on);
       const cb = document.getElementById("cardTrans");
       if (cb) cb.checked = on;
   }

   function setBgOpacity(val) {
       const op = parseInt(val);
       document.documentElement.style.setProperty("--bg-opacity", (op / 100).toFixed(2));
       document.getElementById("bgOpacityLabel").textContent = op + "%";
       localStorage.setItem("ts-bg-opacity", op);
   }

   function removeBg() {
       localStorage.removeItem("ts-bg");
       loadBg();
       document.getElementById("bgPanel").classList.remove("show");
   }

   // 初始化背景
   loadBg();
   loadCardTrans();

   // 自动(按时间)模式下, 每分钟检查一次时段切换
   setInterval(() => {
       const mode = localStorage.getItem("ts-theme") || "auto";
       if (mode === "auto") {
           const shouldDark = isNightTime();
           const isDark = document.documentElement.getAttribute("data-theme") === "dark";
           if (shouldDark !== isDark) {
               applyTheme(mode);
           }
       }
   }, 60000);

   const ICON_OK = '<svg viewBox="0 0 24 24" width="14" height="14" fill="#16a34a" style="vertical-align:-2px;margin-right:4px;"><path d="M9 16.2L4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4z"/></svg>';
   const ICON_FAIL = '<svg viewBox="0 0 24 24" width="14" height="14" fill="#dc2626" style="vertical-align:-2px;margin-right:4px;"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>';
   const ICON_WAIT = '<svg viewBox="0 0 24 24" width="14" height="14" fill="var(--text-muted)" style="vertical-align:-2px;margin-right:4px;"><path d="M12 6v6l4 2 1-1.7-3.2-1.7V6H12zM12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zm0 18a8 8 0 1 1 0-16 8 8 0 0 1 0 16z"/></svg>';

function copyIP(ip) {
   const input = document.createElement("input");
   input.value = ip;
   document.body.appendChild(input);
   input.select();
   document.execCommand("copy");
   input.remove();

   showToast(t("copy-ok") + " " + ip);
}

// 页面底部悬浮提示
function showToast(msg) {
   const toast = document.getElementById("toast");
   toast.innerHTML = msg;
   toast.style.display = "block";
   clearTimeout(toast._timer);
   toast._timer = setTimeout(() => { toast.style.display = "none"; }, 2000);
}

function toggleOffline() {
   const el = document.getElementById("offline");
   el.style.display = el.style.display === "none" ? "grid" : "none";
}

function searchDevice() {
   const key = document.getElementById("search").value.toLowerCase();
   document.querySelectorAll(".device").forEach(d => {
       d.style.display = d.innerText.toLowerCase().includes(key) ? "block" : "none";
   });
}

function pingDevice(ip, btn) {
   const resultDiv = btn.nextElementSibling;
   resultDiv.innerHTML = ICON_WAIT + t("ping-testing");
   resultDiv.className = "ping-result";

   fetch("/ping/" + ip)
       .then(r => r.json())
       .then(data => {
           if (data.reachable) {
               resultDiv.innerHTML = ICON_OK + t("ping-ok") + " " + data.latency_ms + "ms";
               resultDiv.className = "ping-result ping-ok";
           } else {
               resultDiv.innerHTML = ICON_FAIL + t("ping-fail");
               resultDiv.className = "ping-result ping-fail";
           }
       })
       .catch(() => {
           resultDiv.innerHTML = ICON_FAIL + t("ping-error");
           resultDiv.className = "ping-result ping-fail";
       });
}

// 自动刷新 (每 30 秒)
setInterval(() => {
   fetch("/panel")
       .then(r => r.text())
       .then(html => {
           const parser = new DOMParser();
           const doc = parser.parseFromString(html, "text/html");
           // 移除 script 标签 (避免重复加载 panel.js, 导致多个定时器叠加)
           doc.querySelectorAll("script").forEach(s => s.remove());
           document.body.innerHTML = doc.body.innerHTML;
           // 恢复主题状态 (刷新后按钮被重置, 重新应用)
           applyTheme(localStorage.getItem("ts-theme") || "auto");
           // 恢复语言
           applyLang();
           // 恢复背景状态
           loadBg();
           loadCardTrans();
       });
}, 30000);

(function () {
  "use strict";

  const I18N = {
    zh: {
      nav: {
        prototype: "原型",
        grips: "两种握法",
        features: "功能地图"
      },
      hero: {
        title: "取景台",
        lede: "手机端的相机与串流控制台。横竖屏共用一套控制逻辑，空闲时不再是一整屏黑色。",
        factCamera: "MJPEG / RTSP",
        factDesktop: "桌面端直连",
        factLocal: "仅局域网",
        caption: "实时原型 · 390 × 844"
      },
      states: {
        idle: "空闲",
        waiting: "等待连接",
        client: "已连接",
        settings: "设置",
        exposure: "曝光",
        zoom: "变焦",
        focus: "对焦",
        error: "端口占用",
        idleDesc: "服务器关闭时，预览区用镜头卡片替代黑屏，并把启动动作放在卡片里。",
        waitingDesc: "服务器已启动，桌面端尚未接入。预览显示等待状态，地址可直接复制。",
        clientDesc: "桌面端正在拉流。状态、分辨率和复制地址都在预览下方。",
        settingsDesc: "设置从底部升起；串流协议、分辨率、画质和中英切换都在这里。",
        exposureDesc: "曝光滑杆覆盖 −2 到 +2 EV，常用档位一键切换。",
        zoomDesc: "变焦滑杆从 1× 到 8×，当前倍率始终显示在滑杆上方。",
        focusDesc: "自动对焦和手动对焦在同一个面板内切换；手动时点击预览选择对焦点。",
        errorDesc: "8080 被占用时不闪退：明确告知原因，并提供重试和设置入口。"
      },
      grips: {
        title: "两种握法",
        lede: "横屏不是把竖屏拉宽。快捷调节移到右手边，设置变成侧板，预览保持最大。",
        headline: "横屏把控制放回拇指边。",
        body: "竖屏用底部 Dock 承担主操作；横屏把曝光、变焦、对焦、闪光和 DIM 收进右侧控制台，左侧留给完整预览。",
        point1: "预览、状态和 IP 仍在同一视线高度。",
        point2: "横屏左侧预留摄像头安全区，预览标签和取景框避开开孔。",
        point3: "底部电源、设置、切换摄像头位置不变，横竖屏形成肌肉记忆。"
      },
      features: {
        title: "功能地图",
        lede: "现有功能全部保留，只重新分配入口和信息层级。",
        colFunction: "功能",
        colPortrait: "竖屏入口",
        colLandscape: "横屏入口",
        rows: [
          ["打开 / 关闭预览", "顶部眼睛按钮、空闲卡片", "顶部眼睛按钮"],
          ["启动 / 停止服务器", "底部中央电源", "底部中央电源"],
          ["桌面连接状态", "顶部 LIVE 状态、预览底部", "顶部 LIVE 状态、预览底部"],
          ["IP 地址 / 端口", "顶部地址、复制按钮", "顶部地址、复制按钮"],
          ["前后摄像头", "底部右侧切换", "底部右侧切换"],
          ["闪光灯", "快捷轨 FLASH", "右侧控制台 FLASH"],
          ["曝光", "快捷轨 EXP → 底部面板", "右侧控制台 EXP → 侧板"],
          ["变焦", "快捷轨 ZOOM → 底部面板", "右侧控制台 ZOOM → 侧板"],
          ["自动 / 手动对焦", "快捷轨 FOCUS → 底部面板", "右侧控制台 FOCUS → 侧板"],
          ["DIM 息屏", "快捷轨 DIM", "右侧控制台 DIM"],
          ["分辨率", "设置 → 分辨率", "右侧设置侧板"],
          ["串流协议", "设置 → MJPEG / RTSP", "右侧设置侧板"],
          ["JPEG 画质", "设置 → 画质滑杆", "右侧设置侧板"],
          ["帮助与 API", "设置 → 帮助入口", "右侧设置侧板"],
          ["中英切换", "设置 → 语言", "设置 → 语言"],
          ["横竖屏布局", "自动跟随屏幕方向", "自动跟随屏幕方向"],
          ["摄像头安全区", "顶部状态栏避让", "横屏左侧 32px 安全区"],
          ["权限", "启动时申请相机 / 麦克风 / 本地网络", "同竖屏"]
        ]
      },
      notes: {
        oneTitle: "空闲不空",
        oneBody: "服务器关闭时用浅色镜头卡片替代黑屏，启动动作、协议和隐私范围都在卡片里说清楚。",
        twoTitle: "拇指区优先",
        twoBody: "电源固定在底部中央；EXP、ZOOM、FOCUS、FLASH、DIM 组成一条可横滑的调节轨，设置不抢主操作。",
        threeTitle: "一双眼睛",
        threeBody: "取景框四角的括号是唯一贯穿全局的视觉记号——品牌、预览、空状态和报错共用它，其他元素保持安静。"
      },
      footer: {
        note: "HTML 设计原型 · 不改动当前 Android 代码"
      },
      app: {
        network: "局域网",
        liveStatus: "LIVE",
        idleStatus: "IDLE",
        errorStatus: "ERROR",
        preview: "预览",
        previewOff: "预览关闭",
        idleTitle: "准备开播",
        idleBody: "启动服务器后，桌面端即可连接。也可以先打开本地预览。",
        hiddenTitle: "预览已隐藏",
        hiddenBody: "服务器仍在运行。显示预览，或打开设置调整串流参数。",
        start: "启动",
        stop: "停止",
        retry: "重试",
        showPreview: "显示预览",
        hidePreview: "隐藏预览",
        previewOnly: "仅打开预览",
        connected: "桌面端已连接",
        waiting: "等待桌面端连接",
        noClient: "暂无客户端",
        facingFront: "前摄",
        facingBack: "后摄",
        exp: "曝光",
        zoom: "变焦",
        focus: "对焦",
        flash: "闪光",
        dim: "息屏",
        auto: "自动",
        manual: "手动",
        on: "开",
        off: "关",
        settingsTitle: "设置",
        settingsKicker: "串流与界面",
        stream: "串流协议",
        streamHint: "MJPEG 兼容性最好；RTSP 延迟更低。",
        resolution: "分辨率",
        resolutionHint: "选项来自当前摄像头能力。",
        quality: "JPEG 画质",
        qualityHint: "只影响 MJPEG。数值越高，带宽越大。",
        language: "界面语言",
        help: "帮助与 API",
        helpHint: "查看连接地址和接口说明。",
        port: "监听端口",
        portHint: "USB 与 Wi-Fi 都使用这个端口。",
        close: "关闭",
        exposureTitle: "曝光",
        exposureHint: "调整画面明暗。先试 ±1 EV，再细调。",
        reset: "重置",
        zoomTitle: "变焦",
        zoomHint: "当前摄像头的数字变焦范围。",
        focusTitle: "对焦",
        focusAutoHint: "自动对焦会根据画面持续调整。",
        focusManualHint: "手动模式下，点击预览选择对焦点。",
        done: "完成",
        helpTitle: "连接与接口",
        helpLead: "桌面端连接手机后，可直接使用下面这些本地地址。",
        helpWifi: "Wi-Fi：保持同一局域网，打开这个视频地址。",
        helpUsb: "USB：先运行 adb forward，再用本机地址访问。",
        copied: "地址已复制",
        portChip: "端口 8080",
        errorTitle: "端口被占用",
        errorBody: "8080 已被另一个 AWA 实例占用。先停止旧实例，再重试。",
        errorSettings: "打开设置"
      }
    },
    en: {
      nav: {
        prototype: "Prototype",
        grips: "Two grips",
        features: "Feature map"
      },
      hero: {
        title: "Framing desk",
        lede: "Camera and streaming controls for the phone. Portrait and landscape share one logic; idle is no longer a black screen.",
        factCamera: "MJPEG / RTSP",
        factDesktop: "Direct to desktop",
        factLocal: "LAN only",
        caption: "Live prototype · 390 × 844"
      },
      states: {
        idle: "Idle",
        waiting: "Waiting",
        client: "Connected",
        settings: "Settings",
        exposure: "Exposure",
        zoom: "Zoom",
        focus: "Focus",
        error: "Port busy",
        idleDesc: "With the server off, a lens card replaces the black screen and carries the start action.",
        waitingDesc: "The server is on and the desktop has not joined yet. The address is ready to copy.",
        clientDesc: "The desktop is pulling the stream. Status, resolution and copy live under the preview.",
        settingsDesc: "Settings rise from the bottom: protocol, resolution, quality and language.",
        exposureDesc: "Exposure runs from −2 to +2 EV, with common stops one tap away.",
        zoomDesc: "Zoom runs from 1× to 8× and always shows the current multiplier.",
        focusDesc: "Auto and manual share one sheet; in manual, tap the preview to place focus.",
        errorDesc: "A busy port never crashes the app: say what happened and offer retry and settings."
      },
      grips: {
        title: "Two grips",
        lede: "Landscape is not portrait stretched. Quick controls move to the right hand, settings become a side panel, and the preview stays large.",
        headline: "Landscape puts controls back under the thumb.",
        body: "Portrait gives the bottom dock the primary actions. Landscape moves exposure, zoom, focus, flash and dim into a right-hand console, leaving the whole left side to the preview.",
        point1: "Preview, status and IP stay at one eye level.",
        point2: "Landscape reserves a camera safe zone on the left; labels and brackets clear the cutout.",
        point3: "Power, settings and camera switch keep their positions across orientations."
      },
      features: {
        title: "Feature map",
        lede: "Every current function is kept; only its entry point and hierarchy change.",
        colFunction: "Function",
        colPortrait: "Portrait entry",
        colLandscape: "Landscape entry",
        rows: [
          ["Preview on / off", "Top eye button, idle card", "Top eye button"],
          ["Start / stop server", "Bottom center power", "Bottom center power"],
          ["Desktop connection", "Top LIVE state, bottom of preview", "Top LIVE state, bottom of preview"],
          ["IP address / port", "Top address, copy button", "Top address, copy button"],
          ["Front / back camera", "Bottom right switch", "Bottom right switch"],
          ["Flash", "Quick rail FLASH", "Right console FLASH"],
          ["Exposure", "Quick rail EXP → bottom sheet", "Right console EXP → side sheet"],
          ["Zoom", "Quick rail ZOOM → bottom sheet", "Right console ZOOM → side sheet"],
          ["Auto / manual focus", "Quick rail FOCUS → bottom sheet", "Right console FOCUS → side sheet"],
          ["DIM", "Quick rail DIM", "Right console DIM"],
          ["Resolution", "Settings → resolution", "Right settings panel"],
          ["Stream protocol", "Settings → MJPEG / RTSP", "Right settings panel"],
          ["JPEG quality", "Settings → quality slider", "Right settings panel"],
          ["Help & API", "Settings → help", "Right settings panel"],
          ["Chinese / English", "Settings → language", "Settings → language"],
          ["Portrait / landscape", "Follows screen orientation", "Follows screen orientation"],
          ["Camera safe zone", "Top status bar clears the cutout", "32px safe strip on the left"],
          ["Permissions", "Requested at launch: camera / mic / local network", "Same as portrait"]
        ]
      },
      notes: {
        oneTitle: "Idle is a state",
        oneBody: "A paper lens card replaces the black screen; start, protocol and privacy all live inside the card.",
        twoTitle: "Thumb first",
        twoBody: "Power sits at the bottom center. EXP, ZOOM, FOCUS, FLASH and DIM form a scrollable rail; settings stay out of the way.",
        threeTitle: "One pair of corners",
        threeBody: "The viewfinder brackets are the only motif carried through the brand, preview, empty state and error. Everything else stays quiet."
      },
      footer: {
        note: "HTML design prototype · no Android code changed"
      },
      app: {
        network: "LAN",
        liveStatus: "LIVE",
        idleStatus: "IDLE",
        errorStatus: "ERROR",
        preview: "Preview",
        previewOff: "Preview off",
        idleTitle: "Ready when you are",
        idleBody: "Start the server and the desktop can connect. You can open a local preview first.",
        hiddenTitle: "Preview hidden",
        hiddenBody: "The server is still running. Show the preview or open settings to tune the stream.",
        start: "Start",
        stop: "Stop",
        retry: "Retry",
        showPreview: "Show preview",
        hidePreview: "Hide preview",
        previewOnly: "Preview only",
        connected: "Desktop connected",
        waiting: "Waiting for desktop",
        noClient: "No client",
        facingFront: "Front",
        facingBack: "Back",
        exp: "EXP",
        zoom: "ZOOM",
        focus: "FOCUS",
        flash: "FLASH",
        dim: "DIM",
        auto: "Auto",
        manual: "Manual",
        on: "On",
        off: "Off",
        settingsTitle: "Settings",
        settingsKicker: "Stream & interface",
        stream: "Stream protocol",
        streamHint: "MJPEG is the most compatible; RTSP has lower latency.",
        resolution: "Resolution",
        resolutionHint: "Options come from the current camera.",
        quality: "JPEG quality",
        qualityHint: "MJPEG only. Higher values use more bandwidth.",
        language: "Language",
        help: "Help & API",
        helpHint: "See the connection address and endpoints.",
        port: "Listen port",
        portHint: "Used by both USB and Wi-Fi.",
        close: "Close",
        exposureTitle: "Exposure",
        exposureHint: "Brighten or darken the frame. Try ±1 EV first, then fine-tune.",
        reset: "Reset",
        zoomTitle: "Zoom",
        zoomHint: "The digital zoom range of the current camera.",
        focusTitle: "Focus",
        focusAutoHint: "Auto focus keeps adjusting as the frame changes.",
        focusManualHint: "In manual mode, tap the preview to place focus.",
        done: "Done",
        helpTitle: "Connection & API",
        helpLead: "Once the desktop reaches the phone, these local addresses are ready.",
        helpWifi: "Wi-Fi: stay on the same LAN and open the video URL.",
        helpUsb: "USB: run adb forward, then use the local address.",
        copied: "Address copied",
        portChip: "Port 8080",
        errorTitle: "Port is busy",
        errorBody: "Another AWA instance is already using 8080. Stop it, then retry.",
        errorSettings: "Open settings"
      }
    }
  };

  const STATE_META = {
    idle: { title: "states.idle", desc: "states.idleDesc" },
    waiting: { title: "states.waiting", desc: "states.waitingDesc" },
    client: { title: "states.client", desc: "states.clientDesc" },
    settings: { title: "states.settings", desc: "states.settingsDesc" },
    exposure: { title: "states.exposure", desc: "states.exposureDesc" },
    zoom: { title: "states.zoom", desc: "states.zoomDesc" },
    focus: { title: "states.focus", desc: "states.focusDesc" },
    help: { title: "app.helpTitle", desc: "app.helpLead" },
    error: { title: "states.error", desc: "states.errorDesc" }
  };

  const state = {
    lang: "zh",
    mode: "idle",
    sheet: null,
    preview: false,
    exposure: 0.3,
    zoom: 1,
    focusMode: "auto",
    focusValue: 0.55,
    flash: false,
    dim: false,
    facing: "back",
    stream: "MJPEG",
    resolution: "720p",
    quality: 78,
    toast: null
  };

  let toastTimer = null;

  const $ = (selector, root = document) => root.querySelector(selector);
  const $$ = (selector, root = document) => Array.from(root.querySelectorAll(selector));
  const MATERIAL_ICONS = new Set(["settings", "camera-switch", "power", "close", "link"]);
  const icon = (name, className = "") =>
    `<svg class="icon ${MATERIAL_ICONS.has(name) ? "is-material" : ""} ${className}" aria-hidden="true"><use href="#i-${name}"></use></svg>`;

  function getPath(obj, path) {
    return path.split(".").reduce((value, key) => (value == null ? value : value[key]), obj);
  }

  function t(path) {
    return getPath(I18N[state.lang], path) ?? path;
  }

  function escapeHtml(value) {
    return String(value)
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;");
  }

  function formatEV(value) {
    const number = Math.round(value * 10) / 10;
    if (number === 0) return "0.0";
    return `${number > 0 ? "+" : "−"}${Math.abs(number).toFixed(1)}`;
  }

  function formatZoom(value) {
    return `${value.toFixed(1)}×`;
  }

  function resolutionLabel(value) {
    const dimensions = {
      "480p": "640 × 480",
      "720p": "1280 × 720",
      "1080p": "1920 × 1080",
      "4K": "3840 × 2160"
    };
    return `${value} · ${dimensions[value]}`;
  }

  function currentView() {
    if (state.sheet) return state.sheet;
    return state.mode;
  }

  function setView(view) {
    state.sheet = null;
    state.preview = false;

    if (view === "idle") {
      state.mode = "idle";
    } else if (view === "waiting") {
      state.mode = "waiting";
      state.preview = true;
    } else if (view === "client") {
      state.mode = "client";
      state.preview = true;
    } else if (view === "error") {
      state.mode = "error";
    } else {
      state.mode = "client";
      state.preview = true;
      state.sheet = view;
    }
    render();
  }

  function showToast(key) {
    state.toast = key;
    renderPhones();
    window.clearTimeout(toastTimer);
    toastTimer = window.setTimeout(() => {
      state.toast = null;
      renderPhones();
    }, 1800);
  }

  function applyStaticText() {
    document.documentElement.lang = state.lang === "zh" ? "zh-CN" : "en";
    document.title = state.lang === "zh"
      ? "AWA 取景台 · Framing Desk Prototype"
      : "AWA Framing Desk · Prototype";

    $$("[data-i18n]").forEach((element) => {
      const value = t(element.dataset.i18n);
      if (typeof value === "string") element.textContent = value;
    });

    $$("[data-i18n-aria]").forEach((element) => {
      const value = t(element.dataset.i18nAria);
      if (typeof value === "string") element.setAttribute("aria-label", value);
    });

    $$(".lang-button").forEach((button) => {
      button.classList.toggle("is-active", button.dataset.lang === state.lang);
    });

    renderFeatureTable();
    renderStateNote();
    renderStatePicker();
  }

  function renderFeatureTable() {
    const rows = t("features.rows");
    const body = $("#feature-body");
    if (!Array.isArray(rows) || !body) return;
    body.innerHTML = rows
      .map(
        (row) => `
        <tr>
          <td>${escapeHtml(row[0])}</td>
          <td>${escapeHtml(row[1])}</td>
          <td>${escapeHtml(row[2])}</td>
        </tr>`
      )
      .join("");
  }

  function renderStateNote() {
    const view = currentView();
    const meta = STATE_META[view] || STATE_META.idle;
    const title = $("#state-title");
    const description = $("#state-desc");
    if (title) title.textContent = t(meta.title);
    if (description) description.textContent = t(meta.desc);
  }

  function renderStatePicker() {
    const view = currentView();
    const pickerKey = view === "help" ? "settings" : view;
    $$(".state-chip").forEach((chip) => {
      chip.classList.toggle("is-active", chip.dataset.view === pickerKey);
    });
  }

  function render() {
    renderPhones();
  }

  function renderPhones() {
    const portrait = $("#app-portrait");
    const landscape = $("#app-landscape");
    if (portrait) portrait.innerHTML = appMarkup("portrait");
    if (landscape) landscape.innerHTML = appMarkup("landscape");
    renderStateNote();
    renderStatePicker();
    requestAnimationFrame(fitLandscape);
  }

  function fitLandscape() {
    const stage = $(".landscape-stage");
    const device = stage?.querySelector(".device--landscape");
    if (!stage || !device) return;
    const available = Math.max(0, stage.clientWidth);
    const scale = Math.min(1, available / 780);
    stage.style.setProperty("--landscape-scale", scale.toFixed(4));
    stage.style.height = `${Math.ceil(360 * scale) + 12}px`;
  }

  function sceneMarkup() {
    const frontClass = state.facing === "front" ? "is-front" : "";
    return `
      <div class="viewport-scene ${frontClass}" role="img" aria-label="${t("app.preview")}">
        <svg viewBox="0 0 320 480" preserveAspectRatio="xMidYMid slice">
          <rect class="scene-wall" x="0" y="0" width="320" height="480"/>
          <rect class="scene-wall-2" x="0" y="0" width="118" height="480"/>
          <rect class="scene-window" x="155" y="46" width="188" height="205" rx="2"/>
          <path class="scene-window-line" d="M249 46v205M155 148h188"/>
          <path class="scene-light" d="M155 251h188l-72 80H190Z" opacity=".28"/>
          <rect class="scene-table" x="0" y="307" width="320" height="173"/>
          <rect class="scene-table-top" x="0" y="307" width="320" height="12"/>
          <rect class="scene-shadow" x="38" y="316" width="230" height="10" rx="5"/>
          <rect class="scene-book" x="37" y="286" width="64" height="21" rx="2" transform="rotate(-5 69 296)"/>
          <rect class="scene-book-2" x="45" y="278" width="58" height="13" rx="2" transform="rotate(2 74 284)"/>
          <path class="scene-plant" d="M38 285c-20-28-14-53 2-62 7 27 7 41 2 64Z"/>
          <path class="scene-plant-2" d="M44 286c5-41 23-62 47-65-3 34-18 56-44 66Z"/>
          <path class="scene-plant" d="M45 287c-5-38-28-48-41-42 8 27 21 38 38 48Z"/>
          <path class="scene-pot" d="M35 286h30l-5 30H40Z"/>
          <rect class="scene-camera" x="189" y="267" width="82" height="48" rx="7"/>
          <rect class="scene-camera-2" x="207" y="256" width="44" height="16" rx="5"/>
          <circle class="scene-lens" cx="230" cy="292" r="18"/>
          <circle class="scene-lens" cx="230" cy="292" r="8"/>
          <rect class="scene-shadow" x="185" y="315" width="91" height="7" rx="3"/>
        </svg>
      </div>`;
  }

  function idleCardMarkup() {
    const running = state.mode === "waiting" || state.mode === "client";
    const hidden = running && !state.preview;
    const title = hidden ? t("app.hiddenTitle") : t("app.idleTitle");
    const body = hidden ? t("app.hiddenBody") : t("app.idleBody");
    const primaryLabel = hidden ? t("app.showPreview") : t("app.start");
    const primaryAction = hidden ? "preview" : "power";
    const secondaryLabel = hidden ? t("app.stop") : t("app.previewOnly");
    const secondaryAction = hidden ? "power" : "preview";
    return `
      <div class="idle-card">
        <div class="idle-kicker">${t("app.previewOff")}</div>
        <div class="idle-center">
          <div class="idle-mark">◎</div>
          <h2>${title}</h2>
          <p>${body}</p>
          <div class="idle-actions">
            <button class="btn-primary" type="button" data-action="${primaryAction}">${icon("power")} ${primaryLabel}</button>
            <button class="btn-quiet" type="button" data-action="${secondaryAction}">${secondaryLabel}</button>
          </div>
        </div>
        <div class="idle-target" aria-hidden="true"><span></span></div>
        <div class="idle-foot">
          <span class="meta-chip">USB</span>
          <span class="meta-chip">Wi-Fi</span>
          <span class="meta-chip">${t("app.portChip")}</span>
        </div>
        <div class="idle-brackets" aria-hidden="true"><span class="bracket-b"></span></div>
      </div>`;
  }

  function errorCardMarkup() {
    return `
      <div class="error-card">
        <div class="idle-kicker">${t("app.errorStatus")}</div>
        <div class="idle-center">
          <div class="idle-mark">!</div>
          <h2>${t("app.errorTitle")}</h2>
          <p>${t("app.errorBody")}</p>
          <div class="idle-actions">
            <button class="btn-primary" type="button" data-action="retry">${t("app.retry")}</button>
            <button class="btn-quiet" type="button" data-action="sheet" data-sheet="settings">${t("app.errorSettings")}</button>
          </div>
        </div>
        <div class="idle-foot">
          <span class="meta-chip">${t("app.portChip")}</span>
        </div>
        <div class="idle-brackets" aria-hidden="true"><span class="bracket-b"></span></div>
      </div>`;
  }

  function previewMarkup() {
    if (state.mode === "error") return errorCardMarkup();
    if (state.mode === "idle" || !state.preview) return idleCardMarkup();

    const connected = state.mode === "client";
    return `
      ${sceneMarkup()}
      <div class="preview-brackets" aria-hidden="true"><span class="bracket-b"></span></div>
      <div class="preview-top">
        <span class="view-chip">${t("app.preview")}</span>
        <span class="view-chip">${state.stream} · ${state.resolution}</span>
      </div>
      <div class="preview-bottom">
        <div class="view-bottom-copy">
          <div class="view-title"><span class="status-dot ${connected ? "is-live" : ""}" style="display:inline-block;margin-right:6px;vertical-align:1px"></span>${connected ? t("app.connected") : t("app.waiting")}</div>
          <div class="view-subtitle">${state.facing === "front" ? t("app.facingFront") : t("app.facingBack")} · 192.168.2.3:8080</div>
        </div>
        <button class="copy-button" type="button" data-action="copy" aria-label="${t("app.helpWifi")}">${icon("link")}</button>
      </div>
      ${state.focusMode === "manual" ? '<span class="focus-ring" aria-hidden="true"></span>' : ""}
      ${state.dim ? `<div class="dim-layer"><span class="dim-label">${t("app.dim")}</span></div>` : ""}`;
  }

  function quickButton(name, action, label, value, extraClass = "", sheet = "") {
    return `
      <button class="quick ${extraClass}" type="button" data-action="${action}" ${sheet ? `data-sheet="${sheet}"` : ""}>
        <span class="quick-label">${label}</span>
        <span class="quick-value" data-bind="${name}">${value}</span>
      </button>`;
  }

  function quickbarMarkup(containerClass = "quickbar") {
    const flashOn = state.flash;
    const dimOn = state.dim;
    return `
      <div class="${containerClass}">
        ${quickButton("exposure", "sheet", t("app.exp"), formatEV(state.exposure), "", "exposure")}
        ${quickButton("zoom", "sheet", t("app.zoom"), formatZoom(state.zoom), "", "zoom")}
        ${quickButton("focus", "sheet", t("app.focus"), state.focusMode === "auto" ? t("app.auto") : t("app.manual"), "", "focus")}
        ${quickButton("flash", "flash", t("app.flash"), state.flash ? t("app.on") : t("app.off"), flashOn ? "is-on" : "")}
        ${quickButton("dim", "dim", t("app.dim"), state.dim ? t("app.on") : t("app.off"), dimOn ? "is-on" : "")}
      </div>`;
  }

  function dockMarkup() {
    const running = state.mode === "waiting" || state.mode === "client";
    const error = state.mode === "error";
    return `
      <div class="dock">
        <div class="dock-side">
          <button class="icon-button" type="button" data-action="sheet" data-sheet="settings" aria-label="${t("app.settingsTitle")}">${icon("settings")}</button>
        </div>
        <div class="power-wrap">
          <button class="power ${running ? "is-live" : ""} ${error ? "is-error" : ""}" type="button" data-action="power" aria-label="${running ? t("app.stop") : t("app.start")}">
            ${icon("power")}
            <span class="power-label">${error ? t("app.retry") : running ? t("app.stop") : t("app.start")}</span>
          </button>
        </div>
        <div class="dock-side is-right">
          <button class="icon-button" type="button" data-action="camera" aria-label="${state.facing === "front" ? t("app.facingFront") : t("app.facingBack")}">${icon("camera-switch")}</button>
        </div>
      </div>`;
  }

  function settingRow(label, hint, control) {
    return `
      <div class="setting-row">
        <div>
          <span class="setting-label">${label}</span>
          <span class="setting-hint">${hint}</span>
        </div>
        ${control}
      </div>`;
  }

  function settingsContent() {
    const streamSeg = `
      <div class="seg" role="group" aria-label="${t("app.stream")}">
        <button type="button" data-action="stream" data-value="MJPEG" class="${state.stream === "MJPEG" ? "is-active" : ""}">MJPEG</button>
        <button type="button" data-action="stream" data-value="RTSP" class="${state.stream === "RTSP" ? "is-active" : ""}">RTSP</button>
      </div>`;
    const languageSeg = `
      <div class="seg" role="group" aria-label="${t("app.language")}">
        <button type="button" data-action="lang" data-lang="zh" class="${state.lang === "zh" ? "is-active" : ""}">中文</button>
        <button type="button" data-action="lang" data-lang="en" class="${state.lang === "en" ? "is-active" : ""}">EN</button>
      </div>`;
    const resolutionOptions = ["480p", "720p", "1080p", "4K"]
      .map((value) => `<option value="${value}" ${state.resolution === value ? "selected" : ""}>${resolutionLabel(value)}</option>`)
      .join("");

    return `
      <div class="sheet-head">
        <div>
          <h3 class="sheet-title">${t("app.settingsTitle")}</h3>
        </div>
        <button class="sheet-close" type="button" data-action="close-sheet" aria-label="${t("app.close")}">${icon("close")}</button>
      </div>
      <div class="sheet-body">
        ${settingRow(t("app.stream"), t("app.streamHint"), streamSeg)}
        <label class="field">
          <span class="field-label"><span>${t("app.resolution")}</span><span class="field-value">${state.resolution}</span></span>
          <select class="select" data-select="resolution">${resolutionOptions}</select>
        </label>
        ${state.stream === "MJPEG" ? `
          <label class="field">
            <span class="field-label"><span>${t("app.quality")}</span><span class="field-value" data-bind="quality">${state.quality}</span></span>
            <input type="range" min="10" max="100" step="1" value="${state.quality}" data-slider="quality" aria-label="${t("app.quality")}">
          </label>` : ""}
        ${settingRow(t("app.language"), "", languageSeg)}
        ${settingRow(t("app.port"), t("app.portHint"), `<span class="field-value">8080</span>`)}
        <button class="help-row" type="button" data-action="sheet" data-sheet="help">
          <span>${t("app.help")}</span><span class="field-value">${t("app.helpHint")}</span><span class="console-chevron">›</span>
        </button>
      </div>`;
  }

  function exposureContent() {
    return `
      <div class="sheet-head">
        <div>
          <h3 class="sheet-title">${t("app.exposureTitle")}</h3>
        </div>
        <button class="sheet-close" type="button" data-action="close-sheet" aria-label="${t("app.close")}">${icon("close")}</button>
      </div>
      <div class="sheet-value"><span data-bind="exposure-value">${formatEV(state.exposure)}</span><span>EV</span></div>
      <p class="focus-help">${t("app.exposureHint")}</p>
      <input type="range" min="-2" max="2" step="0.1" value="${state.exposure}" data-slider="exposure" aria-label="${t("app.exposureTitle")}">
      <div class="preset-row">
        <button class="preset" type="button" data-action="exposure-preset" data-value="-1">−1 EV</button>
        <button class="preset" type="button" data-action="exposure-preset" data-value="0">0 EV</button>
        <button class="preset" type="button" data-action="exposure-preset" data-value="1">+1 EV</button>
        <button class="preset" type="button" data-action="exposure-preset" data-value="reset">${t("app.reset")}</button>
      </div>
      <div class="idle-actions">
        <button class="btn-primary" type="button" data-action="close-sheet">${t("app.done")}</button>
      </div>`;
  }

  function zoomContent() {
    return `
      <div class="sheet-head">
        <div>
          <h3 class="sheet-title">${t("app.zoomTitle")}</h3>
        </div>
        <button class="sheet-close" type="button" data-action="close-sheet" aria-label="${t("app.close")}">${icon("close")}</button>
      </div>
      <div class="sheet-value"><span data-bind="zoom-value">${formatZoom(state.zoom)}</span></div>
      <p class="focus-help">${t("app.zoomHint")}</p>
      <input type="range" min="1" max="8" step="0.1" value="${state.zoom}" data-slider="zoom" aria-label="${t("app.zoomTitle")}">
      <div class="preset-row">
        <button class="preset" type="button" data-action="zoom-preset" data-value="1">1×</button>
        <button class="preset" type="button" data-action="zoom-preset" data-value="2">2×</button>
        <button class="preset" type="button" data-action="zoom-preset" data-value="4">4×</button>
        <button class="preset" type="button" data-action="zoom-preset" data-value="reset">${t("app.reset")}</button>
      </div>
      <div class="idle-actions">
        <button class="btn-primary" type="button" data-action="close-sheet">${t("app.done")}</button>
      </div>`;
  }

  function focusContent() {
    const focusSeg = `
      <div class="seg" role="group" aria-label="${t("app.focusTitle")}">
        <button type="button" data-action="focus-mode" data-value="auto" class="${state.focusMode === "auto" ? "is-active" : ""}">${t("app.auto")}</button>
        <button type="button" data-action="focus-mode" data-value="manual" class="${state.focusMode === "manual" ? "is-active" : ""}">${t("app.manual")}</button>
      </div>`;
    return `
      <div class="sheet-head">
        <div>
          <h3 class="sheet-title">${t("app.focusTitle")}</h3>
        </div>
        <button class="sheet-close" type="button" data-action="close-sheet" aria-label="${t("app.close")}">${icon("close")}</button>
      </div>
      ${settingRow(t("app.focusTitle"), "", focusSeg)}
      <p class="focus-help">${state.focusMode === "auto" ? t("app.focusAutoHint") : t("app.focusManualHint")}</p>
      <label class="field">
        <span class="field-label"><span>${t("app.focus")}</span><span class="field-value" data-bind="focus-value">${Math.round(state.focusValue * 100)}%</span></span>
        <input type="range" min="0" max="1" step="0.01" value="${state.focusValue}" data-slider="focusValue" aria-label="${t("app.focusTitle")}">
      </label>
      <div class="idle-actions">
        <button class="btn-primary" type="button" data-action="close-sheet">${t("app.done")}</button>
      </div>`;
  }

  function helpContent() {
    return `
      <div class="sheet-head">
        <div>
          <h3 class="sheet-title">${t("app.helpTitle")}</h3>
        </div>
        <button class="sheet-close" type="button" data-action="close-sheet" aria-label="${t("app.close")}">${icon("close")}</button>
      </div>
      <div class="sheet-body help-body">
        <p class="help-lead">${t("app.helpLead")}</p>
        <div class="code-block"><code>http://192.168.2.3:8080/video</code><button class="copy-text" type="button" data-action="copy">${copied ? t("app.copied") : t("app.copyAddress")}</button></div>
        <p class="help-note">${t("app.helpWifi")}</p>
        <p class="help-note">${t("app.helpUsb")}</p>
        <div class="help-actions">
          <button class="btn-primary" type="button" data-action="sheet" data-sheet="settings">${t("app.settingsTitle")}</button>
        </div>
      </div>`;
  }

  function sheetMarkup(isLandscape) {
    if (!state.sheet) return "";
    const content = state.sheet === "settings"
      ? settingsContent()
      : state.sheet === "exposure"
        ? exposureContent()
        : state.sheet === "zoom"
          ? zoomContent()
          : state.sheet === "focus"
            ? focusContent()
            : helpContent();
    const side = isLandscape;
    return `
      <div class="sheet-backdrop" data-action="close-sheet"></div>
      <div class="sheet ${side ? "sheet--side" : ""}">
        <span class="sheet-grip" aria-hidden="true"></span>
        ${content}
      </div>`;
  }

  function landscapeConsoleMarkup() {
    const running = state.mode === "waiting" || state.mode === "client";
    const error = state.mode === "error";
    const statusKey = error ? "app.errorStatus" : running ? "app.liveStatus" : "app.idleStatus";
    const address = error || running ? "192.168.2.3:8080" : "—";

    const control = (name, label, value, action, extra = "", sheet = "") => `
      <button class="console-control ${extra}" type="button" data-action="${action}" ${sheet ? `data-sheet="${sheet}"` : ""}>
        <span class="console-control-copy">
          <span class="console-control-label">${label}</span>
          <span class="console-control-value" data-bind="${name}">${value}</span>
        </span>
        ${sheet ? '<span class="console-chevron">›</span>' : ""}
      </button>`;

    return `
      <aside class="landscape-console">
        <div class="console-head">
          <span class="status-live"><span class="status-dot ${error ? "is-error" : running ? "is-live" : ""}"></span>${t(statusKey)}</span>
          <span class="console-address">${address}</span>
          <button class="preview-glyph" type="button" data-action="preview" aria-label="${state.preview ? t("app.hidePreview") : t("app.showPreview")}">${state.preview ? "\u25C9" : "\u25CB"}</button>
        </div>
        <div class="console-controls">
          ${control("exposure", t("app.exp"), formatEV(state.exposure), "sheet", "", "exposure")}
          ${control("zoom", t("app.zoom"), formatZoom(state.zoom), "sheet", "", "zoom")}
          ${control("focus", t("app.focus"), state.focusMode === "auto" ? t("app.auto") : t("app.manual"), "sheet", "", "focus")}
          ${control("flash", t("app.flash"), state.flash ? t("app.on") : t("app.off"), "flash", state.flash ? "is-on" : "")}
          ${control("dim", t("app.dim"), state.dim ? t("app.on") : t("app.off"), "dim", state.dim ? "is-on" : "")}
        </div>
        <div class="console-footer">
          <button class="console-icon" type="button" data-action="sheet" data-sheet="settings" aria-label="${t("app.settingsTitle")}">${icon("settings")}</button>
          <div class="power-wrap">
            <button class="power ${running ? "is-live" : ""} ${error ? "is-error" : ""}" type="button" data-action="power" aria-label="${running ? t("app.stop") : t("app.start")}">
              ${icon("power")}
              <span class="power-label">${error ? t("app.retry") : running ? t("app.stop") : t("app.start")}</span>
            </button>
          </div>
          <button class="console-icon" type="button" data-action="camera" aria-label="${state.facing === "front" ? t("app.facingFront") : t("app.facingBack")}">${icon("camera-switch")}</button>
        </div>
      </aside>`;
  }

  function appMarkup(orientation) {
    const isLandscape = orientation === "landscape";
    const running = state.mode === "waiting" || state.mode === "client";
    const error = state.mode === "error";
    const statusKey = error ? "app.errorStatus" : running ? "app.liveStatus" : "app.idleStatus";
    const address = state.mode === "error" ? "192.168.2.3:8080" : running ? "192.168.2.3:8080" : "—";

    if (isLandscape) {
      return `
        <div class="app app--landscape">
          <div class="landscape-preview">
            <span class="camera-cutout" aria-hidden="true"></span>
            ${previewMarkup()}
          </div>
          ${landscapeConsoleMarkup()}
          ${sheetMarkup(true)}
          ${state.toast ? `<div class="toast">${t(state.toast)}</div>` : ""}
        </div>`;
    }

    return `
      <div class="app app--portrait">
        <header class="app-top">
          <div class="app-top-left">
            <span class="status-live"><span class="status-dot ${error ? "is-error" : running ? "is-live" : ""}"></span>${t(statusKey)}</span>
        </div>
        <div class="app-top-right">
          <span class="app-address">${address}</span>
          <button class="preview-glyph" type="button" data-action="preview" aria-label="${state.preview ? t("app.hidePreview") : t("app.showPreview")}">${state.preview ? "\u25C9" : "\u25CB"}</button>
        </div>
      </header>
        <div class="app-body">
          <div class="viewport-wrap">
            <div class="viewport">${previewMarkup()}</div>
          </div>
        </div>
        ${quickbarMarkup("quickbar")}
        ${dockMarkup()}
        ${sheetMarkup(false)}
        ${state.toast ? `<div class="toast">${t(state.toast)}</div>` : ""}
      </div>`;
  }

  function updateValueLabels() {
    $$('[data-bind="exposure"]').forEach((element) => {
      element.textContent = formatEV(state.exposure);
    });
    $$('[data-bind="exposure-value"]').forEach((element) => {
      element.textContent = formatEV(state.exposure);
    });
    $$('[data-bind="zoom"]').forEach((element) => {
      element.textContent = formatZoom(state.zoom);
    });
    $$('[data-bind="zoom-value"]').forEach((element) => {
      element.textContent = formatZoom(state.zoom);
    });
    $$('[data-bind="focus"]').forEach((element) => {
      element.textContent = state.focusMode === "auto" ? t("app.auto") : t("app.manual");
    });
    $$('[data-bind="focus-value"]').forEach((element) => {
      element.textContent = `${Math.round(state.focusValue * 100)}%`;
    });
    $$('[data-bind="quality"]').forEach((element) => {
      element.textContent = state.quality;
    });
  }

  function setLanguage(lang) {
    if (lang !== "zh" && lang !== "en") return;
    state.lang = lang;
    applyStaticText();
    renderPhones();
  }

  function copyAddress() {
    const value = "http://192.168.2.3:8080/video";
    if (navigator.clipboard && window.isSecureContext) {
      navigator.clipboard.writeText(value).catch(() => {});
    }
    showToast("app.copied");
  }

  function handleAction(action, button) {
    switch (action) {
      case "lang":
        setLanguage(button.dataset.lang);
        break;
      case "sheet":
        state.preview = true;
        state.sheet = button.dataset.sheet;
        if (state.sheet === "settings" || state.sheet === "help") state.mode = "client";
        render();
        break;
      case "close-sheet":
        state.sheet = null;
        render();
        break;
      case "preview":
        state.preview = !state.preview;
        render();
        break;
      case "power":
        if (state.mode === "error") {
          setView("waiting");
        } else if (state.mode === "idle") {
          state.mode = "waiting";
          state.preview = true;
          state.sheet = null;
          render();
        } else {
          state.mode = "idle";
          state.preview = false;
          state.sheet = null;
          render();
        }
        break;
      case "camera":
        state.facing = state.facing === "back" ? "front" : "back";
        render();
        break;
      case "flash":
        state.flash = !state.flash;
        render();
        break;
      case "dim":
        state.dim = !state.dim;
        render();
        break;
      case "retry":
        setView("waiting");
        break;
      case "copy":
        copyAddress();
        break;
      case "stream":
        state.stream = button.dataset.value;
        render();
        break;
      case "focus-mode":
        state.focusMode = button.dataset.value;
        render();
        break;
      case "exposure-preset":
        state.exposure = button.dataset.value === "reset" ? 0 : Number(button.dataset.value);
        render();
        break;
      case "zoom-preset":
        state.zoom = button.dataset.value === "reset" ? 1 : Number(button.dataset.value);
        render();
        break;
      default:
        break;
    }
  }

  document.addEventListener("click", (event) => {
    const viewButton = event.target.closest("[data-view]");
    if (viewButton) {
      setView(viewButton.dataset.view);
      return;
    }

    const actionButton = event.target.closest("[data-action]");
    if (actionButton) {
      handleAction(actionButton.dataset.action, actionButton);
    }
  });

  document.addEventListener("input", (event) => {
    const slider = event.target.closest("[data-slider]");
    if (!slider) return;
    const key = slider.dataset.slider;
    const value = Number(slider.value);
    if (!Number.isFinite(value)) return;
    state[key] = value;
    updateValueLabels();
  });

  document.addEventListener("change", (event) => {
    const select = event.target.closest("[data-select]");
    if (!select) return;
    if (select.dataset.select === "resolution") {
      state.resolution = select.value;
      render();
    }
  });

  function init() {
    const params = new URLSearchParams(window.location.search);
    if (params.get("lang") === "en") state.lang = "en";
    const requestedView = params.get("view");
    window.addEventListener("resize", fitLandscape);
    applyStaticText();
    if (requestedView && STATE_META[requestedView]) {
      setView(requestedView);
    } else {
      setView("idle");
    }
  }

  init();
})();

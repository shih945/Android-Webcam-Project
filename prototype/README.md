# AWA 取景台 · Framing Desk 原型

这是手机端 AWA 的 HTML 设计原型，**不改动当前 Android 代码**。

## 打开方式

直接打开单文件版本：

```text
prototype/awa-redesign.html
```

也可以打开带独立 CSS/JS 的开发版本：

```text
prototype/index.html
```

无需构建，也不需要后端。

## 切换状态

页面左侧的状态按钮可以切换全部演示状态：

```text
空闲 / 等待连接 / 已连接 / 设置 / 曝光 / 变焦 / 对焦 / 端口占用
```

也可以直接用 URL 参数打开指定状态：

```text
prototype/index.html?view=idle
prototype/index.html?view=waiting
prototype/index.html?view=client
prototype/index.html?view=settings
prototype/index.html?view=exposure
prototype/index.html?view=zoom
prototype/index.html?view=focus
prototype/index.html?view=error
```

语言参数：

```text
prototype/index.html?view=waiting&lang=zh
prototype/index.html?view=waiting&lang=en
```

## 设计方向

- **方向**：夜间取景台，不是全黑相机界面。
- **签名元素**：取景框四角的括号，贯穿品牌、预览、空状态、报错。
- **空闲态**：浅色镜头卡片替代黑屏，启动服务器和仅预览两个动作放在卡片里。
- **竖屏**：顶部状态 + 大预览 + 底部快捷轨 + Dock。
- **横屏**：左侧整块预览，右侧独立控制台；曝光、变焦、对焦、闪光、DIM 全部在右手边。
- **摄像头安全区**：横屏左侧预留 34px 安全区，预览标签、取景框、底部连接条和空状态内容都避开开孔；落地时用 `WindowInsets.displayCutout` / `safeDrawing` 动态计算。
- **设置**：竖屏底部 Sheet，横屏右侧侧板。
- **国际化**：全部界面文案支持中英切换，包括状态、设置、错误和帮助。

## 功能覆盖

| 功能 | 位置 |
|---|---|
| 打开 / 关闭预览 | 顶部眼睛按钮、空闲卡片 |
| 启动 / 停止服务器 | 底部中央电源 |
| 桌面连接状态 | 顶部 LIVE / 连接条 |
| IP 地址 / 端口 | 顶部地址、复制按钮 |
| 前后摄像头 | Dock 右侧 |
| 闪光灯 | 快捷轨 / 右侧控制台 |
| 曝光 | EXP 面板 |
| 变焦 | ZOOM 面板 |
| 自动 / 手动对焦 | FOCUS 面板 |
| DIM 息屏 | 快捷轨 / 右侧控制台 |
| 分辨率 | 设置 |
| MJPEG / RTSP | 设置 |
| JPEG 画质 | 设置 |
| 帮助与 API | 设置 |
| 中英切换 | 页面顶部、应用设置 |
| 横竖屏 | 两套布局，同一套控制逻辑 |

## 原型截图

截图在 `prototype/shots/`，包括空闲、等待、已连接、横屏分栏、设置和英文界面。

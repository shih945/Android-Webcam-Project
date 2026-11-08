# AWC（egui 原型）

这是 AWC 桌面端的 egui 版本原型，用来验证「Rust 视频直通 + 降采样预览」这条链路。

## shadcn-egui

界面基于 [shadcn-egui](https://crates.io/crates/shadcn-egui) 0.7.0（egui/eframe 0.34）：

- 启动时应用 `Theme::vega()`（shadcn v4 vega 风格）；
- 使用它的 `Button`、`Card`、`Badge`、`Input`、`Switch` 组件；
- 左右布局：顶部状态栏 + 左侧 Card 控制区 + 中间预览 Card + 底部日志；
- 浅色/深色由 shadcn 主题的两套 palette 提供，`System / Light / Dark` 仍可手动切换。

> 仓库里同时带 Claude Code 插件（`plugin/skills/build-ui`）。当前 DSH 环境没有 Claude Code 插件市场，
> 因此这里按 crate 的官方方式集成设计系统；组件 API 和主题配置与技能的约定一致。

## 当前能力

- **视频主路径**：`StreamReceiver`（MJPEG/RTSP）→ `DecodedFrame` → `PreviewTap`（可选）→ `SoftcamSink` → `softcam.dll`
- **预览旁路**：Throttled（默认 8 fps、640×360）、Full rate、Native 三种模式；关闭时 `PreviewTap::maybe_tap` 直接返回，不做任何拷贝
- **虚拟摄像头**：动态加载 `softcam.dll`，首帧按实际宽高创建，尺寸变化自动重建；容量 1 的帧槽，满则丢最旧帧并计数
- **测试源**：本地彩条 + 移动方块 + 扫描线 + 二进制帧计数，喂同一条管线，不需要手机
- **画面变换**：旋转 0° / 90° / 180° / 270°，以及左右翻转；在解码后、预览和 softcam 之前统一处理，90°/270° 会交换输出宽高并触发虚拟摄像头重建
- **控制面**：ADB 设备 / Wi-Fi、`/features` 探测、分辨率切换、前后摄像头切换、虚拟摄像头启停、Setup（`regsvr32`，注册方式未改）
- **不经过 UI**：解码后的 BGR 帧只存在于视频模块；egui 只拿到 `PreviewFrame`（RGBA）和原子计数

## 打包客户端

项目根目录提供两个入口：

```powershell
# 推荐：绕过 PowerShell 执行策略
.\package-awc.cmd

# 或直接运行脚本
powershell -NoProfile -ExecutionPolicy Bypass -File .\package-awc.ps1
```

- 只编译不打包：`.\package-awc.cmd -SkipBuild`
- 只生成目录不生成 zip：`.\package-awc.cmd -NoZip`
- 输出：`dist\AWC-Client-<版本>\` 和 `dist\AWC-Client-<版本>.zip`
- `package-awc.ps1` 开头是配置块：Rust 工具链、MinGW、libclang、FFmpeg、cargo 镜像都在那里；
- AWC 源码、`.tools`、`dist` 都相对脚本解析，整个项目迁移后只需按新机器修改开头的外部路径；
- 本机的外部 SDK/工具放在 `D:\app`：`jdk-21`、`gradle-9.6.1`、`android-sdk`、`ffmpeg81`、`clang-include`；`cargo-home`、`gradle-home` 等用户/编译缓存仍放在项目 `.tools`；
- 脚本会读取版本号、复制 `awc.exe`、`softcam.dll`、FFmpeg DLL、`adb\`、`LICENSE`，并生成客户端 `README.txt`；
- 最后输出 zip 的 SHA256。

## 构建（本机）

本机没有 MSVC，使用 `D:\app` 下的 GNU 工具链、MinGW、FFmpeg 8.1 和 cargo 镜像。

```powershell
$toolchain = 'D:\app\rust\rustup-home\toolchains\stable-x86_64-pc-windows-gnu\bin'
$env:CARGO_HOME = 'D:\desktop\Android-Webcam-Project\.tools\cargo-home'
$env:PATH = "$toolchain;D:\app\mingw64\bin;" + $env:PATH
$env:CC = 'gcc'
$env:AR = 'ar'
$env:LIBCLANG_PATH = 'D:\app\Huawei\sdk\23\native\llvm\bin'
$env:BINDGEN_EXTRA_CLANG_ARGS = '-ID:/app/clang-include -ID:/app/mingw64/x86_64-w64-mingw32/include --target=x86_64-w64-windows-gnu'
$env:FFMPEG_DIR = 'D:\app\ffmpeg81\ffmpeg-n8.1-latest-win64-gpl-shared-8.1'
$env:CARGO_TARGET_DIR = 'D:\desktop\Android-Webcam-Project\.tools\target-awc'

cargo build --target x86_64-pc-windows-gnu
cargo test --target x86_64-pc-windows-gnu
```

构建脚本会把 `softcam.dll`、`FFMPEG_DIR\bin\*.dll` 和 `adb\` 复制到可执行文件旁边。

## 运行

```powershell
# 带测试源启动（推荐先看这个）
.\awc.exe --test-pattern

# 测试源 + 自动启动虚拟摄像头
.\awc.exe --test-pattern --start-cam

# 直连 Wi-Fi 手机（会先拉 /features，再自动开始 MJPEG/RTSP）
.\awc.exe --wifi 192.168.2.4:8080 --start-cam

# 同上，但关闭预览（用于验证预览零开销）
.\awc.exe --wifi 192.168.2.4:8080 --start-cam --no-preview

# 普通启动
.\awc.exe
```

> 实时预览/虚拟摄像头请用 `cargo build --release`。debug 构建下 JPEG 解码和降采样未优化，
> 640×480 也可能只有 2 fps；release 构建同一路流可到 ~26–30 fps。

测试源启动后：

1. 顶部状态显示 `Test pattern · 1280×720 @ 30 fps`；
2. 中间预览显示彩条、移动方块、扫描线和二进制帧计数；
3. 左侧 **VIRTUAL CAMERA → Start virtual camera** 可把同一路测试帧送进 `softcam.dll`；
4. 左侧 **PREVIEW** 可以开关预览、调 1–15 fps、选 360p/480p/720p；
5. 日志区显示 `Stream: N frames · ...`、`preview N taps`、`Sent / Dropped / FPS`。

`--start-cam` 会自动启动虚拟摄像头，适合验证 `softcam.dll` 加载、`scCreateCamera` 和 `scSendFrame`，不依赖手机。

## 验证点

- 测试源 30 fps：`Stream: N frames` 的增长速率应接近 30/s；预览 `preview taps` 接近 8/s。
- 关掉预览：`preview taps` 停止增长，`Stream` 帧率不变。
- 点 **Start virtual camera**：`Output` 出现 `1280×720`，`Sent` 持续增长；Zoom/Teams/OBS 应能枚举到 `AWC Virtual Cam`。
- 切换测试源分辨率后重启测试源：sink 会自动重建摄像头，不需要手动重启虚拟摄像头。

## 国际化

- 支持 English / 简体中文；首次启动跟随 Windows 显示语言。
- 顶栏右侧语言下拉可即时切换，写入 `%APPDATA%\AWC\config.json` 的 `language`。
- `AWC_LANG=en|zh|system` 可在启动时覆盖语言。
- 中文界面会自动从 Windows 字体目录加载 Microsoft YaHei / SimHei 作为 CJK fallback。
- 核心层日志仍保留英文技术原文，UI 标签、按钮、状态、预览说明已翻译。

## 布局

- 页面四周 **16 logical** padding；顶栏、侧栏、主区之间有明确 gutter；
- 三层 surface：canvas `#F4F4F5` / sidebar `#FAFAFA` / card `#FFFFFF`；
- 侧栏改为 **Setup / Camera / Phone / Test** 四个 Tab，不再依赖不可见的滚动条；
- 预览控制放在主 Card 顶部，预览图四边留 12 logical inset，底部不再贴边；
- 日志默认收起，顶部 `Log` 打开。

## 主题

- 默认跟随系统；顶部下拉可切 `System / Light / Dark`。
- egui 0.34 在运行时 `ctx.set_theme` 会破坏 shadcn 字体的 glyph atlas（文字变成乱码）。所以 AWC 始终让 egui 停留在 Light 模式，把目标主题的 palette 填进 shadcn 的 `light` slot 来实现明暗切换；字形 atlas 不再变化。
- `AWC_THEME=light|dark|system` 可在启动时覆盖配置。

- 默认 **System**：跟随 Windows 浅色/深色主题。
- 顶部右侧下拉框可手动切 **System / Light / Dark**，选择写入 `AWC` 配置。
- 浅色主题下所有面板、文字、按钮和状态色都有对应调色板，不是简单反色。

## 预览模式与实测

预览有三种模式（左侧 PREVIEW）：

| 模式 | 行为 | 720p 实测主路 | 预览开销 |
|---|---|---|---|
| Throttled | 降采样到 640×360，限制 8 fps | ~25.8 fps | 4.72 ms/帧 |
| Full rate (downscaled) | 每帧降采样到 640×360 | ~26.0 fps | 4.77 ms/帧 |
| Native (full quality) | 原始分辨率、每帧 | ~26.0 fps | 1.77 ms/帧 |
| 关闭预览 | 不产生预览帧 | ~25.9 fps | 0 |

测试条件：手机 1280×720 MJPEG、release 构建、虚拟摄像头同时运行。结论是 **720p 下预览模式对主路推流没有可测影响**；主路瓶颈在 JPEG 解码/解析（手机原始流约 31 fps，桌面端约 26 fps），不在预览。原生预览反而比面积平均降采样便宜，因为少了一次缩放计算。

> 4K 原生预览会每帧复制约 8.3 MB 并上传纹理，开销显著高于 720p；高分辨率下建议先用 Throttled 或 Full rate。

## 已知限制

- 手机 MJPEG 已用真机联调；RTSP 分支沿用原有 FFmpeg/D3D11VA 配置，尚未实测。
- 相机对焦/曝光/闪光灯滑块尚未移植；当前只保留分辨率、前后摄像头切换。
- 首次运行不会自动提权安装；点顶部 **Setup** 执行原有 `regsvr32` 流程。
- 4K 原生预览未做完整测量，预计开销明显高于 720p。

## 目录

```
src/
  app.rs                 egui 界面
  core/
    controller.rs        控制线程：连接、测试源、sink、ADB、Setup
    state.rs             SharedState / UiSnapshot
    config.rs            %APPDATA%\AWC\config.json
    phone.rs             /features、/control
    adb.rs               adb devices / forward
    installer.rs         softcam 注册检查
  video/
    frame.rs             DecodedFrame（packed BGR24 + PTS）
    source.rs            MJPEG / RTSP 解码线程
    test_pattern.rs      本地测试源
    preview.rs           降采样 + 单槽预览
    sink.rs              VirtualCamSink / SoftcamSink
```

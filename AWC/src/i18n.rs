//! Minimal two-language UI catalogue.
//!
//! English source strings double as keys, so a missing translation falls back
//! to English instead of showing a blank label.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Zh,
}

impl Lang {
    pub fn from_config(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "zh" | "zh-cn" | "zh_cn" | "chinese" => Self::Zh,
            "en" | "en-us" | "en_us" | "english" => Self::En,
            _ => system_lang(),
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Zh => "zh",
        }
    }
}

pub fn system_lang() -> Lang {
    #[cfg(windows)]
    {
        // LANGID low 10 bits are the primary language; 0x04 is Chinese.
        let id = unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() };
        if id & 0x03ff == 0x04 {
            return Lang::Zh;
        }
    }
    #[cfg(not(windows))]
    {
        if let Ok(value) = std::env::var("LANG") {
            if value.to_ascii_lowercase().starts_with("zh") {
                return Lang::Zh;
            }
        }
    }
    Lang::En
}

pub fn tr(lang: Lang, key: &'static str) -> &'static str {
    if lang == Lang::En {
        return key;
    }
    match key {
        "Setup" => "设置",
        "Camera" => "摄像头",
        "Test" => "测试",
        "Log" => "日志",
        "System" => "跟随系统",
        "Light" => "浅色",
        "Dark" => "深色",
        "Not registered" => "未注册",
        "TEST" => "测试",
        "Connection" => "连接",
        "USB or Wi-Fi" => "USB 或 Wi-Fi",
        "Virtual camera" => "虚拟摄像头",
        "DirectShow output" => "DirectShow 输出",
        "Phone" => "手机",
        "Camera controls" => "相机控制",
        "Test source" => "测试源",
        "No phone required" => "无需手机",
        "Phone (ADB)" => "手机 (ADB)",
        "Select a device" => "选择设备",
        "Refresh devices" => "刷新设备",
        "Phone IP" => "手机 IP",
        "Connecting…" => "连接中…",
        "Disconnect" => "断开连接",
        "Connect phone" => "连接手机",
        "Connect" => "连接",
        "Stop test pattern" => "停止测试画面",
        "Start test pattern" => "启动测试画面",
        "Local colour bars; feeds the same preview + virtual-camera path." => {
            "本地彩条；与预览、虚拟摄像头走同一条通路。"
        }
        "Test pattern {}×{} @ {} fps" => "测试画面 {}×{} @ {} fps",
        "Stop virtual camera" => "停止虚拟摄像头",
        "Start virtual camera" => "启动虚拟摄像头",
        "In use by an app" => "正在被应用使用",
        "Waiting for Zoom / Teams / OBS" => "等待 Zoom / Teams / OBS 使用",
        "Not running" => "未运行",
        "waiting for first frame" => "等待首帧",
        "Output {resolution} · frame #{} · source {:.1}s · output {:.1}s" => {
            "输出 {resolution} · 帧 #{} · 源 {:.1}s · 输出 {:.1}s"
        }
        "Sent" => "已发送",
        "Dropped" => "已丢弃",
        "FPS" => "帧率",
        "{} frames · {} errors · {} reconnects · {:.1} MB in · {} preview taps" => {
            "{} 帧 · {} 错误 · {} 次重连 · 输入 {:.1} MB · {} 次预览取帧"
        }
        "Throttled (lowest cost)" => "限帧（最低开销）",
        "Full rate, downscaled" => "全帧率，降分辨率",
        "Native resolution + rate" => "原始分辨率 + 帧率",
        "Preview work {:.2} ms/frame (last {:.2} ms) · {} taps · decode thread" => {
            "预览开销 {:.2} ms/帧（最近 {:.2} ms）· {} 次取帧 · 解码线程"
        }
        "Downsample + fps cap. Cheapest; the decode thread does little extra work." => {
            "降采样 + 限帧。开销最低，解码线程几乎不增加工作。"
        }
        "Downsample every decoded frame. More decode-thread work, no fps cap." => {
            "每帧都降采样。解码线程工作更多，但不限制帧率。"
        }
        "Full resolution every frame. Most expensive; can reduce main-stream fps." => {
            "每帧都使用全分辨率。开销最大，可能降低主路帧率。"
        }
        "Connect to load camera controls." => "连接手机后加载相机控制。",
        "Protocol: {}" => "协议：{}",
        "Resolution" => "分辨率",
        "Switch camera" => "切换摄像头",
        "Manual focus available" => "支持手动对焦",
        "Zoom range: {:.1}× – {:.1}×" => "变焦范围：{:.1}× – {:.1}×",
        "Exposure range: {} – {}" => "曝光范围：{} – {}",
        "{:.1} fps stream · {:.1} fps output" => "输入 {:.1} fps · 输出 {:.1} fps",
        "Preview" => "预览",
        "Rotation" => "旋转",
        "Mirror" => "左右翻转",
        "Preview off" => "预览已关闭",
        "Turn on Preview above." => "请在上方打开预览。",
        "No signal" => "无信号",
        "Connect to the phone and start the stream." => "连接手机并启动视频流。",
        "LOG" => "日志",
        "Clear" => "清空",
        "{} fps in · {} fps out · {} dropped/s" => "输入 {} fps · 输出 {} fps · 丢弃 {} /s",
        "Idle" => "空闲",
        "Connecting" => "连接中",
        "Live" => "实时",
        "Error" => "错误",
        "Throttled" => "限帧",
        "Full rate (downscaled)" => "全帧率（降分辨率）",
        "Native (full quality)" => "原始（全画质）",
        "custom" => "自定义",
        "Language" => "语言",
        _ => key,
    }
}

pub fn test_pattern(lang: Lang, width: u32, height: u32, fps: u32) -> String {
    match lang {
        Lang::En => format!("Test pattern {width}×{height} @ {fps} fps"),
        Lang::Zh => format!("测试画面 {width}×{height} @ {fps} fps"),
    }
}

pub fn output_stats(lang: Lang, resolution: &str, seq: u64, source: f32, output: f32) -> String {
    match lang {
        Lang::En => format!(
            "Output {resolution} · frame #{seq} · source {source:.1}s · output {output:.1}s"
        ),
        Lang::Zh => format!(
            "输出 {resolution} · 帧 #{seq} · 源 {source:.1}s · 输出 {output:.1}s"
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn stream_summary(
    lang: Lang,
    frames: u64,
    errors: u64,
    reconnects: u64,
    megabytes: f64,
    taps: u64,
) -> String {
    match lang {
        Lang::En => format!(
            "{frames} frames · {errors} errors · {reconnects} reconnects · {megabytes:.1} MB in · {taps} preview taps"
        ),
        Lang::Zh => format!(
            "{frames} 帧 · {errors} 错误 · {reconnects} 次重连 · 输入 {megabytes:.1} MB · {taps} 次预览取帧"
        ),
    }
}

pub fn preview_work(lang: Lang, average_ms: f64, last_ms: f64, taps: u64) -> String {
    match lang {
        Lang::En => format!(
            "Preview work {average_ms:.2} ms/frame (last {last_ms:.2} ms) · {taps} taps · decode thread"
        ),
        Lang::Zh => format!(
            "预览开销 {average_ms:.2} ms/帧（最近 {last_ms:.2} ms）· {taps} 次取帧 · 解码线程"
        ),
    }
}

pub fn protocol(lang: Lang, value: &str) -> String {
    match lang {
        Lang::En => format!("Protocol: {value}"),
        Lang::Zh => format!("协议：{value}"),
    }
}

pub fn zoom_range(lang: Lang, min: f32, max: f32) -> String {
    match lang {
        Lang::En => format!("Zoom range: {min:.1}× – {max:.1}×"),
        Lang::Zh => format!("变焦范围：{min:.1}× – {max:.1}×"),
    }
}

pub fn exposure_range(lang: Lang, lower: i32, upper: i32) -> String {
    match lang {
        Lang::En => format!("Exposure range: {lower} – {upper}"),
        Lang::Zh => format!("曝光范围：{lower} – {upper}"),
    }
}

pub fn fps_stream_output(lang: Lang, stream: f32, output: f32) -> String {
    match lang {
        Lang::En => format!("{stream:.1} fps stream · {output:.1} fps output"),
        Lang::Zh => format!("输入 {stream:.1} fps · 输出 {output:.1} fps"),
    }
}

pub fn fps_in_out_dropped(lang: Lang, input: f32, output: f32, dropped: u64) -> String {
    match lang {
        Lang::En => format!("{input} fps in · {output} fps out · {dropped} dropped/s"),
        Lang::Zh => format!("输入 {input} fps · 输出 {output} fps · 丢弃 {dropped} /s"),
    }
}

/// Translate the few status messages the controller publishes.
pub fn localize_message(lang: Lang, message: &str) -> String {    if lang == Lang::En {
        return message.to_string();
    }
    match message {
        "Ready to connect." => "准备连接。".to_string(),
        "Disconnected" => "已断开".to_string(),
        "Connecting" => "连接中".to_string(),
        "Virtual camera started" => "虚拟摄像头已启动".to_string(),
        "Virtual camera stopped" => "虚拟摄像头已停止".to_string(),
        "Test pattern stopped" => "测试画面已停止".to_string(),
        "Pick a resolution after connecting" => "连接后请选择分辨率".to_string(),
        "Connect to the phone first" => "请先连接手机".to_string(),
        _ => {
            if let Some(rest) = message.strip_prefix("Live · ") {
                return format!("实时 · {}", rest.replace("resolution(s)", "个分辨率"));
            }
            if let Some(rest) = message.strip_prefix("Querying ") {
                return format!("正在查询 {rest}");
            }
            if let Some(rest) = message.strip_prefix("Connection failed: ") {
                return format!("连接失败：{rest}");
            }
            if let Some(rest) = message.strip_prefix("Test pattern · ") {
                return format!("测试画面 · {rest}");
            }
            if let Some(rest) = message.strip_prefix("Test pattern started: ") {
                return format!("测试画面已启动：{rest}");
            }
            if let Some(rest) = message.strip_prefix("Resolution requested: ") {
                return format!("已请求分辨率：{rest}");
            }
            if let Some(rest) = message.strip_prefix("ADB: ") {
                return format!("ADB：{rest}");
            }
            if let Some(rest) = message.strip_prefix("Virtual camera: ") {
                return format!("虚拟摄像头：{rest}");
            }
            message.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_languages_parse() {
        assert_eq!(Lang::from_config("zh-CN"), Lang::Zh);
        assert_eq!(Lang::from_config("EN"), Lang::En);
    }

    #[test]
    fn chinese_labels_and_fallback() {
        assert_eq!(tr(Lang::Zh, "Setup"), "设置");
        assert_eq!(tr(Lang::Zh, "not-a-real-key"), "not-a-real-key");
        assert_eq!(tr(Lang::En, "Setup"), "Setup");
    }

    #[test]
    fn status_messages_localize() {
        assert_eq!(
            localize_message(Lang::Zh, "Live · MJPEG · 4 resolution(s)"),
            "实时 · MJPEG · 4 个分辨率"
        );
        assert_eq!(localize_message(Lang::En, "Disconnected"), "Disconnected");
    }
}

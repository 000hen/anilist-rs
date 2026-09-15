# anilist-rs

[English](README-en.md)

使用 Rust 與 [Iced](https://iced.rs) 打造的桌面動畫新番時間表應用程式。自動抓取當季番表，將日本播出時間轉換為本地時區，以一週為單位呈現整齊的播出時刻表。

## ✨ 功能

- 📅 依年份與季度查詢當季動畫播出時間表
- 🕐 自動將日本播出時間（JST）轉換為你的本地時區
- 📺 以「從今天開始」的一週為順序，清楚列出每日播出的動畫
- 🖼️ 封面圖片延遲載入（Lazy Loading），搭配 Viewport 預載，節省頻寬
- 🪟 多視窗架構——點擊動畫卡片即可開啟獨立的詳細資訊視窗
- 🔗 詳細視窗中可直接點擊串流平台圖示（Netflix、巴哈動畫瘋、Prime Video 等），以預設瀏覽器開啟

## 📦 專案結構

```
anilist-rs
├── anilist-core          # 核心資料模型與時區轉換邏輯
│                           Anime、AnimeTime、AnimeSeason、Minute、ScheduleDay
├── anilist-source        # AnimeParser / AnimeSource trait 抽象層與錯誤類型
├── anilist-nextjs        # 輕量 Next.js hydration / Flight 解析器
├── anilist-nextjs-ffi    # 可獨立建置的 Next.js UniFFI
├── anilist-ffi           # 動態 parser / fetcher FFI，可選 HTTP、來源與系統時區功能
├── anilist-youranimes    # youranimes.tw 資料來源實作（HTML / RSC JSON 解析）
└── anilist-iced          # Iced GUI 多視窗桌面應用程式
```

## 🛠️ 技術棧

### Rust / Kotlin FFI

可獨立建置 Next.js FFI；若只需要 YourAnimes parser，可使用
`--no-default-features --features youranimes`，避免打包 HTTP / Tokio 與系統時區資料。
預設則包含 YourAnimes、HTTP 與系統時區支援。建置方式、Kotlin 綁定遷移與解析 API 請見
[FFI 說明](docs/ffi.md)。

| 類別       | 套件                                              |
| ---------- | ------------------------------------------------- |
| 語言       | Rust（Edition 2024）                              |
| GUI 框架   | [Iced](https://iced.rs) 0.14（daemon 多視窗模式） |
| HTTP 請求  | reqwest                                           |
| HTML 解析  | html5gum（不建立 DOM）                            |
| 時區處理   | chrono、chrono-tz、iana-time-zone                 |
| 非同步執行 | tokio                                             |
| 序列化     | serde / serde_json                                |
| 系統整合   | open（開啟預設瀏覽器）                            |

## 🚀 開始使用

### 環境需求

- [Rust](https://rustup.rs/)（建議使用最新穩定版）

### 建置與執行

```bash
# 複製專案
git clone https://github.com/Muisnow/anilist-rs.git
cd anilist-rs

# 執行
cargo run -p anilist-iced

# 或建置 release 版本
cargo build --release
```

## 🏗️ 架構

```
anilist-core（資料模型 & 時區邏輯）
    ↑
anilist-source（AnimeParser / AnimeSource trait 介面）
    ↑
anilist-youranimes（youranimes.tw parser / fetcher 實作）
    ↑
anilist-iced（多視窗 GUI 應用程式）
```

核心採用 trait 抽象設計。`AnimeParser` 定義同步的 `parse_list`、`parse_search`、`parse_detail`；
`AnimeSource` 則定義非同步的 `list`、`search`、`detail`。FFI 透過 source ID 動態建立對應的
`NativeAnimeParser` 或 `NativeAnimeFetcher`，因此新增來源時不需要為每個來源建立另一套 FFI API。

時區轉換由 `anilist-core` 的 `AnimeTime::to_zone()` 負責，能正確處理跨日、跨星期的邊界情況。

## 📄 授權

本專案採用 [MIT License](LICENSE) 授權。

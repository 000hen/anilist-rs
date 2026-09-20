# anilist-rs

[English](README-en.md)

以 Rust 實作的動畫資料來源、Next.js 解析器、UniFFI 函式庫，以及使用
[Iced](https://iced.rs) 打造的桌面新番時間表。桌面程式會自動抓取當季番表，將日本播出時間
轉換為本地時區，並以今天開始的一週順序呈現。

## ✨ 功能

- 📅 桌面程式啟動時自動載入目前年份與季度的動畫番表
- 🕐 自動將日本播出時間（JST）轉換為你的本地時區
- 📺 以「從今天開始」的一週為順序，清楚列出每日播出的動畫
- 🖼️ 封面圖片延遲載入（Lazy Loading），搭配 Viewport 預載，節省頻寬
- 🪟 多視窗架構——點擊動畫卡片即可開啟獨立的詳細資訊視窗
- 🔗 詳細視窗中可直接點擊串流平台圖示（Netflix、巴哈動畫瘋、Prime Video 等），以預設瀏覽器開啟

桌面介面目前不提供季度選擇或搜尋；依年份／季度取得番表，以及搜尋／詳細資料功能，
則由資料來源與 FFI API 提供。

## 📦 專案結構

```
anilist-rs
├── anilist-core          # 核心資料模型與時區轉換邏輯
│                           Anime、AnimeTime、AnimeSeason、Minute、ScheduleDay
├── anilist-source        # Parser、來源協定、HTTP transport 與 fetcher trait
├── anilist-nextjs        # 輕量 Next.js hydration / Flight 解析器
├── anilist-nextjs-ffi    # 可獨立建置的 Next.js UniFFI
├── anilist-youranimes    # youranimes.tw 資料來源實作（HTML / RSC JSON 解析）
├── anilist-ffi           # 動態 source / parser / fetcher UniFFI
├── anilist-iced          # Iced GUI 多視窗桌面應用程式
└── uniffi-bindgen        # 從建置產物產生外部語言綁定的工具
```

## 🛠️ 技術棧

### Rust / Kotlin FFI

可獨立建置 Next.js FFI。`anilist-ffi` 提供三種整合層級：`NativeAnimeSource` 產生通用
`HttpRequest` 並解析回應，讓宿主自行執行網路請求；`NativeAnimeParser` 只解析宿主提供的內容；
`NativeAnimeFetcher` 則使用 Rust 的 HTTP client 完成整個流程。

Android 等自行管理網路的宿主可使用 `--no-default-features --features all-sources`，
避免打包 reqwest、Tokio 與系統時區資料。預設建置包含所有來源、Rust HTTP 與系統時區支援。
目前唯一的來源 ID 是 `youranimes`。建置方式、Kotlin 綁定與完整 API 請見
[FFI 說明](docs/ffi.md)。

| 類別      | 套件                                         |
|---------|--------------------------------------------|
| 語言      | Rust（Edition 2024）                         |
| GUI 框架  | [Iced](https://iced.rs) 0.14（daemon 多視窗模式） |
| HTTP 請求 | reqwest                                    |
| HTML 解析 | html5gum（不建立 DOM）                          |
| 時區處理    | chrono、chrono-tz、iana-time-zone            |
| 非同步執行   | tokio                                      |
| 序列化     | serde / serde_json                         |
| 系統整合    | open（開啟預設瀏覽器）                              |

## 🚀 開始使用

### 環境需求

- [Rust](https://rustup.rs/)（建議使用最新穩定版）

### 建置與執行

```bash
# 複製專案
git clone https://github.com/000hen/anilist-rs.git
cd anilist-rs

# 執行
cargo run -p anilist-iced

# 或只建置桌面程式的 release 版本
cargo build --release -p anilist-iced
```

## 🏗️ 架構

```
anilist-core ─────────┐
                      ├─ anilist-youranimes ─┬─ anilist-iced
anilist-source ───────┤                      └─ anilist-ffi
anilist-nextjs ───────┘
```

核心採用 trait 抽象設計。`AnimeParser` 定義同步解析；`AnimeSource` 在此之上定義來源所需的
`list_request`、`search_request`、`detail_request`，但不綁定 HTTP client；`AnimeFetcher` 才是
執行非同步 `list`、`search`、`detail` 的完整 fetcher。`YourAnimesFetcher<C>` 可搭配任意
`HttpClient`，預設 Rust HTTP 功能則使用 reqwest。

FFI 透過 source ID 動態建立 `NativeAnimeSource`、`NativeAnimeParser` 或 `NativeAnimeFetcher`。
宿主可選擇自行執行 `HttpRequest`，或讓 Rust 擁有網路流程，而不需要為每個來源建立另一套 FFI API。

時區轉換由 `anilist-core` 的 `AnimeTime::to_zone()` 負責，能正確處理跨日、跨星期的邊界情況。

## 📄 授權

本專案採用 [MIT License](LICENSE) 授權。

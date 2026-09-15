# anilist-rs

[中文](README.md)

A desktop anime seasonal broadcast timetable app built with Rust and [Iced](https://iced.rs). Fetches the current season's anime schedule, converts Japanese airing times to your local timezone, and presents a clean weekly timetable.

## ✨ Features

- 📅 Query anime broadcast schedules by year and season
- 🕐 Automatic timezone conversion from JST to your local timezone
- 📺 Weekly timetable starting from today's weekday
- 🖼️ Lazy image loading with viewport anticipation for smooth scrolling
- 🪟 Multi-window architecture — click an anime card to open a dedicated detail window
- 🔗 Clickable streaming platform icons (Netflix, Bahamut Anime Crazy, Prime Video, etc.) that open directly in your default browser

## 📦 Project Structure

```
anilist-rs
├── anilist-core          # Core data models & timezone conversion logic
│                           Anime, AnimeTime, AnimeSeason, Minute, ScheduleDay
├── anilist-source        # AnimeParser / AnimeSource traits & shared errors
├── anilist-nextjs        # Lightweight Next.js hydration / Flight parser
├── anilist-nextjs-ffi    # Independently buildable Next.js UniFFI
├── anilist-ffi           # Dynamic parser/fetcher FFI with optional HTTP, source and timezone support
├── anilist-youranimes    # youranimes.tw parser/fetcher implementation
└── anilist-iced          # Iced GUI multi-window desktop application
```

## 🛠️ Tech Stack

### Rust / Kotlin FFI

The standalone Next.js FFI can be built independently. For a YourAnimes
parser-only build without HTTP/Tokio or system timezone support, use
`--no-default-features --features youranimes`. The default build includes
YourAnimes, HTTP fetching, and system timezone support. See the
[FFI guide](docs/ffi.md) for build variants, API contracts, and Kotlin migration.

| Category       | Crate                                                   |
| -------------- | ------------------------------------------------------- |
| Language       | Rust (Edition 2024)                                     |
| GUI Framework  | [Iced](https://iced.rs) 0.14 (daemon multi-window mode) |
| HTTP Client    | reqwest                                                 |
| HTML Parsing   | html5gum (no DOM)                                       |
| Timezone       | chrono, chrono-tz, iana-time-zone                       |
| Async Runtime  | tokio                                                   |
| Serialization  | serde / serde_json                                      |
| OS Integration | open (launch default browser)                           |

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable recommended)

### Build & Run

```bash
# Clone the repo
git clone https://github.com/Muisnow/anilist-rs.git
cd anilist-rs

# Run
cargo run -p anilist-iced

# Or build for release
cargo build --release
```

## 🏗️ Architecture

```
anilist-core (data models & timezone logic)
    ↑
anilist-source (AnimeParser / AnimeSource trait interfaces)
    ↑
anilist-youranimes (youranimes.tw parser / fetcher implementation)
    ↑
anilist-iced (multi-window GUI application)
```

The project uses trait-based source abstraction. `AnimeParser` provides synchronous
`parse_list`, `parse_search`, and `parse_detail` operations. `AnimeSource` provides
asynchronous `list`, `search`, and `detail` operations. The FFI constructs a
`NativeAnimeParser` or `NativeAnimeFetcher` dynamically from a source ID, so a
new source does not need its own parallel FFI API surface.

Timezone conversion is handled by `AnimeTime::to_zone()` in `anilist-core`, which correctly accounts for day and weekday boundary crossings.

## 📄 License

This project is licensed under the [MIT License](LICENSE).

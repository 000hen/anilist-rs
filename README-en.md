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
├── anilist-source        # AnimeSource trait abstraction & error types
├── anilist-nextjs        # Lightweight Next.js hydration / Flight parser
├── anilist-nextjs-ffi    # Independently buildable Next.js UniFFI
├── anilist-ffi           # All parser FFI with optional HTTP and system timezone
├── anilist-youranimes    # youranimes.tw source implementation (HTML / RSC JSON parsing)
└── anilist-iced          # Iced GUI multi-window desktop application
```

## 🛠️ Tech Stack

| Category       | Crate                                                   |
| -------------- | ------------------------------------------------------- |
| Language       | Rust (Edition 2024)                                     |
| GUI Framework  | [Iced](https://iced.rs) 0.14 (daemon multi-window mode) |
| HTTP Client    | reqwest                                                 |
| HTML Parsing   | html5gum (no DOM)                                        |
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

For standalone Next.js FFI, parser-only builds without HTTP/Tokio, and default
HTTP-enabled builds, see the [FFI guide](docs/ffi.md). It includes API contracts
and the generated Kotlin binding migration.

```
anilist-core (data models & timezone logic)
    ↑
anilist-source (AnimeSource trait interface)
    ↑
anilist-youranimes (youranimes.tw scraper implementation)
    ↑
anilist-iced (multi-window GUI application)
```

The project uses a trait-based abstraction. The `AnimeSource` trait defines an async `list(year, season)` method that returns all anime for a given season. Implement this trait to plug in a new data source.

Timezone conversion is handled by `AnimeTime::to_zone()` in `anilist-core`, which correctly accounts for day and weekday boundary crossings.

## 📄 License

This project is licensed under the [MIT License](LICENSE).

# anilist-rs

[中文](README.md)

Rust anime-source, Next.js parsing, and UniFFI libraries, plus a desktop seasonal timetable built
with [Iced](https://iced.rs). The desktop app fetches the current season, converts Japanese airing
times to the local timezone, and presents the schedule as a week starting from today.

## ✨ Features

- 📅 Automatically load the current year and season in the desktop app
- 🕐 Automatic timezone conversion from JST to your local timezone
- 📺 Weekly timetable starting from today's weekday
- 🖼️ Lazy image loading with viewport anticipation for smooth scrolling
- 🪟 Multi-window architecture — click an anime card to open a dedicated detail window
- 🔗 Clickable streaming platform icons (Netflix, Bahamut Anime Crazy, Prime Video, etc.) that open
  directly in your default browser

The Iced desktop UI does not currently expose season selection or search. Year/season list queries,
search, and detail operations are available through the source and FFI APIs.

A native [WinUI 3 version](crates/anilist-winui/README.md) is also available on Windows,
using Rust `windows-reactor`, Fluent controls, Mica, and local season search.
Run it with `cargo run -p anilist-winui` after installing Windows App Runtime 2.4 or later.

## 📦 Project Structure

```
anilist-rs/crates
├── anilist-core          # Core data models & timezone conversion logic
│                           Anime, AnimeTime, AnimeSeason, Minute, ScheduleDay
├── anilist-source        # Parser, source protocol, HTTP transport, and fetcher traits
├── anilist-nextjs        # Lightweight Next.js hydration / Flight parser
├── anilist-nextjs-ffi    # Independently buildable Next.js UniFFI
├── anilist-youranimes    # youranimes.tw parser/fetcher implementation
├── anilist-ffi           # Dynamic source/parser/fetcher UniFFI
├── anilist-iced          # Iced GUI multi-window desktop application
├── anilist-winui         # Native WinUI 3 / Fluent Windows application
└── uniffi-bindgen        # Binding generator for built UniFFI libraries
```

## 🛠️ Tech Stack

### Rust / Kotlin FFI

The standalone Next.js FFI can be built independently. `anilist-ffi` offers three
integration levels: `NativeAnimeSource` produces source-independent `HttpRequest`
values and parses responses for host-owned networking; `NativeAnimeParser` only
parses content supplied by the host; and `NativeAnimeFetcher` lets Rust perform
the complete HTTP flow.

Hosts such as Android can build with `--no-default-features --features all-sources`
to exclude reqwest, Tokio, and system timezone data. The default build includes all
sources, Rust HTTP fetching, and system timezone support. The only source ID currently
available is `youranimes`. See the
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
git clone https://github.com/000hen/anilist-rs.git
cd anilist-rs

# Run
cargo run -p anilist-iced

# Or build only the desktop app for release
cargo build --release -p anilist-iced
```

## 🏗️ Architecture

```
anilist-core ─────────┐
                      ├─ anilist-youranimes ─┬─ anilist-iced
anilist-source ───────┤                      └─ anilist-ffi
anilist-nextjs ───────┘
```

The project uses trait-based source abstraction. `AnimeParser` defines synchronous
parsing. `AnimeSource` extends it with `list_request`, `search_request`, and
`detail_request` while remaining independent of any HTTP client. `AnimeFetcher`
defines the complete asynchronous `list`, `search`, and `detail` operations.
`YourAnimesFetcher<C>` can use any `HttpClient`; the default Rust HTTP feature uses reqwest.

The FFI constructs `NativeAnimeSource`, `NativeAnimeParser`, or `NativeAnimeFetcher`
dynamically from a source ID. A host can execute the returned `HttpRequest` itself or
let Rust own the network flow without requiring a parallel FFI surface per source.

Timezone conversion is handled by `AnimeTime::to_zone()` in `anilist-core`, which correctly accounts
for day and weekday boundary crossings.

## 📄 License

This project is licensed under the [MIT License](LICENSE).

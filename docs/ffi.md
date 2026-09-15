# Rust FFI distributions

The Rust libraries own parsing and domain conversion. Consumers can either use
the default HTTP exports or fetch response bodies themselves and call synchronous
parsers. No foreign HTTP callback or Rust async runtime is needed for parsing.

## Build only what you use

Run these as separate package builds. Cargo unifies features when multiple
workspace members are built together; the desktop app enables HTTP explicitly.

```sh
# Next.js only: no YourAnimes, anime models, HTTP, or Tokio
cargo build -p anilist-nextjs-ffi --release

# All parsers and anime models, without HTTP, Tokio, or a timezone database
cargo build -p anilist-ffi --release --no-default-features

# Complete library, including HTTP fetching (the default)
cargo build -p anilist-ffi --release

# HTTP fetching with source schedules, without local-time conversion
cargo build -p anilist-ffi --release --no-default-features --features http

# Example Android parser library (requires the Android target and NDK linker)
cargo build -p anilist-ffi --release --no-default-features --target aarch64-linux-android
```

The standalone library is `anilist_nextjs_ffi`; the complete/parser library is
`anilist`. Platform filenames use the usual DLL, SO, or dylib conventions.
Both packages also produce static libraries and Rust rlibs.

```text
anilist-nextjs-ffi -> anilist-nextjs
         ^
anilist-ffi -> anilist-youranimes -> anilist-core + anilist-nextjs
  [http]       [http]
  UniFFI       Reqwest + futures + source trait
  Tokio
```

`anilist-youranimes` and `anilist-ffi` default to `http` and `system-timezone`.
Rust consumers that only parse should set `default-features = false` on those
dependencies. `system-timezone` owns the IANA database and system clock support;
it is independent of HTTP. Core's `timezone` and `clock` features separately
enable conversion and reading the system clock. With both disabled, core keeps
only the IANA zone name and basic calendar primitives.
Reqwest's HTTP implementation still requires Tokio; making HTTP optional does
not make its fetcher compatible with arbitrary async runtimes.

## API boundary

| Rust export | Input | Output |
| --- | --- | --- |
| `deserialize_nextjs` | HTML | JSON with `nextData` and `flight` |
| `your_animes_parse_list` | Season page HTML | `Vec<Anime>` |
| `your_animes_parse_detail` | Detail page HTML | `Anime` |
| `your_animes_parse_search` | Search API JSON | `Vec<String>` of `youranimes:<id>` |
| `anilist_list`, `anilist_detail`, `anilist_search` | Query parameters | Anime records; requires `http` |
| `week_order` | Host's current `Weekday` | Eight schedule days, unknown last; always available |
| `current_week_order` | None | Same ordering using system clock; requires `system-timezone` |

Search parsing preserves response order and returns IDs, not fabricated partial
anime records. Kotlin fetches each `/animes/<id>` page using the portion after
`youranimes:`, then calls `yourAnimesParseDetail`. The default HTTP search still
fetches those details in Rust. Kotlin owns HTTP status checks, request scheduling,
cancellation, and retries when using parser-only mode.

The Rust parser exports are also available directly as `parse_list`,
`parse_detail`, and `parse_search` from `anilist_youranimes`. They return
`YourAnimesParseError`, independent of HTTP errors. FFI maps malformed input to
`AnilistError::InvalidResponse` and preserves anime conversion errors separately.
Failure to determine the system timezone is `AnilistError::TimezoneUnavailable`,
not a malformed server response.

### Schedule ownership

Synchronous list/detail parsers **always return the original Tokyo schedule**,
regardless of Cargo features. For example, Monday 00:30 stays Monday 00:30 with
`zone = "Asia/Tokyo"`. They do not read the clock or detect the host timezone.
This changes the previous Rust parser behavior, which localized during parsing.

Kotlin should convert that weekly schedule with its own `java.time` timezone
support and a reference date. A weekday/time is recurring, not an absolute
timestamp: choose the intended airing week to handle DST correctly. Preserve
unknown minutes as unknown. The FFI `AnimeTime` shape is unchanged.

Default HTTP fetches localize schedules after parsing, retaining the desktop
behavior. With only `http` enabled, HTTP results retain the source timezone.
Rust callers using `AnimeTime::to_zone` must enable core's `timezone` feature;
`AnimeTime.zone` is now an owned IANA string and the struct is no longer `Copy`.
Invalid source zone names return a conversion error when converting a known time.

## Generate bindings for the actual artifact

Next.js is now its own UniFFI component. Rust imports through `anilist` still
work, but generated Next.js bindings move to `uniffi.anilist_nextjs_ffi`.
YourAnimes and domain bindings remain in `uniffi.anilist`. Regenerate bindings
and update Next.js imports when adopting this redesign.
For parser-only callers, use `weekOrder(today)` instead of `currentWeekOrder()`
and provide the weekday from the host clock.

Example for a Windows parser-only build:

```sh
cargo build -p anilist-ffi --no-default-features
cargo run -p uniffi-bindgen -- generate target/debug/anilist.dll --language kotlin --out-dir target/bindings/parser
```

For a standalone Next.js build, use `target/debug/anilist_nextjs_ffi.dll`.
On Linux/macOS substitute the actual library path. Generate and distribute all
component binding files emitted for the selected library. Do not pair bindings
from an HTTP-enabled build with a parser-only binary: their exported APIs differ.
Use separate output directories for variants to avoid retaining stale files.

UniFFI aggregation follows its
[component scaffolding guidance](https://mozilla.github.io/uniffi-rs/next/tutorial/Rust_scaffolding.html).

## Validation

```sh
cargo test -p anilist-ffi --no-default-features
cargo test -p anilist-ffi
cargo test -p anilist-youranimes --no-default-features
cargo test -p anilist-youranimes -- --skip tests::test_fetching
cargo tree -p anilist-ffi --no-default-features --edges normal
cargo tree -p anilist-nextjs-ffi --edges normal
```

The lean parser dependency tree must contain none of `reqwest`, `rustls`, `tokio`,
`chrono-tz`, or `iana-time-zone`. Next.js uses `html5gum` with default features
disabled; naive state switching preserves HTML script raw-text semantics.
YourAnimes's existing live fetching tests require network access; the commands
above skip those in the default-feature test run.

### Reproduce shared-library size measurements

```sh
python scripts/measure-ffi.py --offline
```

This builds five release variants for Cargo's configured target (host by default), copies each shared library under
`target/ffi-size/<variant>/`, and writes `measurements.json` with the compiler
version, exact Cargo-reported artifact paths, and byte counts. It compares Next.js-only, lean parser, parser with
timezone support, full defaults, and the same lean parser at `opt-level = "s"`.
The other variants explicitly use `"z"`. It does not clean the workspace or
change the checked-in release profile. Remove `--offline` if dependencies need
downloading. Compare the DLL/SO/dylib only; static libraries and rlibs are not
included in the size totals. Results are platform/compiler-specific.

# Rust FFI distributions

The Rust libraries own parsing and domain conversion. Consumers can either let
Rust perform HTTP requests through `NativeAnimeFetcher`, or fetch response bodies
themselves and pass them to the synchronous `NativeAnimeParser`. Parser-only
consumers do not need a Rust async runtime.

## Build only what you use

Run these as separate package builds. Cargo unifies features when multiple
workspace members are built together, so build the FFI package by itself when
you need a lean distribution.

```sh
# Next.js only: no YourAnimes, anime models, HTTP, or Tokio
cargo build -p anilist-nextjs-ffi --release

# YourAnimes parser + anime models, without HTTP, Tokio, or a timezone database
cargo build -p anilist-ffi --release --no-default-features --features youranimes

# Complete library, including YourAnimes HTTP fetching and system timezone support
cargo build -p anilist-ffi --release

# YourAnimes HTTP fetching without local-time conversion
cargo build -p anilist-ffi --release --no-default-features --features http,youranimes

# Example Android parser library (requires the Android target and NDK linker)
cargo build -p anilist-ffi --release --no-default-features --features youranimes --target aarch64-linux-android
```

The standalone library is `anilist_nextjs_ffi`; the complete/parser library is
`anilist`. Platform filenames use the usual DLL, SO, or dylib conventions.
Both packages also produce static libraries and Rust rlibs.

```text
anilist-nextjs-ffi -> anilist-nextjs
         ^
anilist-ffi -> anilist-youranimes -> anilist-core + anilist-nextjs
  UniFFI        [http]
  [http]        Reqwest + futures
  Tokio
```

### Feature contract

`anilist-ffi` has three independent feature switches:

- `youranimes` enables the YourAnimes parser/source implementation.
- `http` enables the HTTP-facing UniFFI API, Reqwest, and UniFFI's Tokio runtime.
  It intentionally does not select a source by itself.
- `system-timezone` enables system-clock/timezone helpers and forwards timezone
  support to a source when that source is enabled.

The default feature set is `http + system-timezone + youranimes`.
`anilist-youranimes` itself has no default features; its HTTP and timezone
capabilities are opt-in.

This distinction matters for parser-only consumers. `--no-default-features`
by itself builds the generic FFI/model shell but does not include any source
implementation. To construct `NativeAnimeParser("youranimes")`, enable the
`youranimes` feature explicitly.

Reqwest's HTTP implementation still requires Tokio. Making HTTP optional does
not make its fetcher compatible with arbitrary async runtimes.

## API boundary

| Rust export | Input | Output / behavior |
| --- | --- | --- |
| `deserialize_nextjs` | HTML | JSON with `nextData` and `flight` |
| `NativeAnimeParser::new` | `source_id` | Dynamic parser for the selected compiled-in source |
| `NativeAnimeParser::parse_list` | Season page HTML | `Vec<Anime>` |
| `NativeAnimeParser::parse_detail` | Detail page HTML | `Anime` |
| `NativeAnimeParser::parse_search` | Search API JSON | `Vec<String>` source IDs |
| `NativeAnimeFetcher::new` | `source_id` | Dynamic HTTP fetcher; requires `http` and the source feature |
| `NativeAnimeFetcher::list` | Year + season | `Vec<Anime>` |
| `NativeAnimeFetcher::detail` | Anime ID | `Anime` |
| `NativeAnimeFetcher::search` | Keyword | `Vec<Anime>` |
| `week_order` | Host's current `Weekday` | Eight schedule days, unknown last; always available |
| `current_week_order` | None | Same ordering using system clock; requires `system-timezone` |

The currently supported dynamic source ID is `youranimes` when the
`youranimes` feature is enabled. Unsupported or compiled-out source IDs return
`AnilistError::UnsupportedSource`.

Search parsing preserves response order and returns IDs, not fabricated partial
anime records. A parser-only client can fetch each `/animes/<id>` page using the
portion after `youranimes:`, then call `NativeAnimeParser::parse_detail`.
`NativeAnimeFetcher::search` performs those detail requests inside Rust.

Malformed parser input is mapped into `AnilistError::InvalidResponse`.
HTTP transport/status failures are `AnilistError::SourceUnavailable`.
Timezone lookup and schedule conversion failures remain distinct error variants.

### Schedule ownership

Synchronous list/detail parsers always return the source schedule in its original
timezone. For YourAnimes, Monday 00:30 remains Monday 00:30 with
`zone = "Asia/Tokyo"`. Parsing does not read the host clock or detect the host
timezone.

A platform client such as Android should convert that recurring weekly schedule
using its native timezone APIs and a reference date. A weekday/time is recurring,
not an absolute timestamp, so the reference date is required for DST-sensitive
zones. Preserve unknown minutes as unknown.

Default HTTP fetching keeps the desktop behavior: when `system-timezone` is
enabled, fetched schedules are localized after parsing. With only `http` and
`youranimes`, HTTP results retain the source timezone.

Rust callers using `AnimeTime::to_zone` must enable core's `timezone` feature.
`AnimeTime.zone` stores an IANA timezone name as an owned string, so retaining a
source timezone does not require shipping the timezone database.

## Kotlin / Android parser migration

For Android, prefer keeping HTTP in Kotlin and compiling Rust as a parser-only
library:

```sh
cargo build \
  -p anilist-ffi \
  --release \
  --no-default-features \
  --features youranimes \
  --target aarch64-linux-android
```

Generate Kotlin bindings from an artifact built with the same feature set:

```sh
cargo build -p anilist-ffi --no-default-features --features youranimes
cargo run -p uniffi-bindgen -- generate target/debug/anilist.dll --language kotlin --out-dir target/bindings/parser
```

On Linux/macOS substitute the actual library path. Do not generate bindings from
an HTTP-enabled artifact and package them with a parser-only binary: the exported
API surface differs by feature set. Use separate output directories for variants
to avoid retaining stale generated files.

The Kotlin-side parser shape is intentionally small:

```kotlin
val parser = NativeAnimeParser("youranimes")
val anime = parser.parseList(responseBody)
```

Kotlin can continue to own HTTP status handling, cancellation, retries, and
request scheduling. The returned `AnimeTime.zone` should then be converted with
`java.time` before grouping the schedule if the app wants local-time ordering.

Next.js is also a standalone UniFFI component. Rust imports through `anilist`
still work, but generated Next.js bindings belong to the
`uniffi.anilist_nextjs_ffi` component. YourAnimes and domain bindings remain in
`uniffi.anilist`. Generate and distribute all component binding files emitted
for the selected library.

For parser-only callers, use `weekOrder(today)` instead of
`currentWeekOrder()` and provide the weekday from the host clock.

UniFFI aggregation follows its
[component scaffolding guidance](https://mozilla.github.io/uniffi-rs/next/tutorial/Rust_scaffolding.html).

## Validation

```sh
# Parser-only YourAnimes FFI
cargo test -p anilist-ffi --no-default-features --features youranimes

# Full default FFI
cargo test -p anilist-ffi

# YourAnimes parser crate without HTTP/timezone extras
cargo test -p anilist-youranimes --no-default-features

# YourAnimes default test set while skipping live network tests when necessary
cargo test -p anilist-youranimes --features http,system-timezone -- --skip tests::test_fetching

# Verify the lean FFI dependency tree
cargo tree -p anilist-ffi --no-default-features --features youranimes --edges normal
cargo tree -p anilist-nextjs-ffi --edges normal
```

The lean parser dependency tree must contain none of `reqwest`, `rustls`,
`tokio`, `chrono-tz`, or `iana-time-zone`. Next.js uses `html5gum` with default
features disabled.

### Reproduce shared-library size measurements

```sh
python scripts/measure-ffi.py --offline
```

This builds five release variants for Cargo's configured target (host by default),
copies each shared library under `target/ffi-size/<variant>/`, and writes
`measurements.json` with the compiler version, exact Cargo-reported artifact
paths, and byte counts. The parser variants explicitly enable `youranimes`, so
their sizes represent a usable YourAnimes parser rather than a source-less FFI
shell. The script compares Next.js-only, lean parser, parser with timezone
support, full defaults, and the same lean parser at `opt-level = "s"`; the other
variants use `"z"`.

The script does not clean the workspace or change the checked-in release profile.
Remove `--offline` if dependencies need downloading. Compare the DLL/SO/dylib
only; static libraries and rlibs are not included in the size totals. Results are
platform/compiler-specific.

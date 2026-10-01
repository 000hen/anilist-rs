# Anilist for Windows

Native WinUI 3 version of the Iced seasonal anime timetable, written in Rust with
[windows-reactor 0.100](https://docs.rs/windows-reactor/0.100.0/windows_reactor/).

## Build and run

Use Windows with the MSVC Rust toolchain (Rust 1.95 or later), Visual Studio C++
build tools and Windows SDK, and Windows App Runtime **2.4 or later in the 2.x family**
for your architecture. Reactor 0.100 resolves `Microsoft.WindowsAppRuntime.2` itself.
See Microsoft's [Windows App SDK downloads](https://learn.microsoft.com/windows/apps/windows-app-sdk/downloads).

From the workspace root:

```powershell
python scripts/setup-windows-reactor.py
cargo run -p anilist-winui
cargo build --release -p anilist-winui
cargo test -p anilist-winui
```

This is a framework-dependent Rust executable, not an MSIX or a .NET Reactor app.
The Windows App Runtime is a separate prerequisite; it is not bundled. Non-Windows
builds provide a small unsupported-platform message and can run the pure catalog tests.

## Behavior and design

- Loads the selected season (the current season at startup) through the shared YourAnimes fetcher, including local
  timezone conversion. Refresh replaces the catalog instead of appending duplicates.
- Groups animation cards from today's weekday, sorts each day by airing time, and
  places unknown schedules last. The schedule shows only the season and weekday sections.
- The title-bar search submits with Enter and opens a compact thumbnail list with independent loading,
  error, and empty states. Back returns to the preserved seasonal catalog. Older
  requests cannot overwrite newer results; clearing a submitted query resets search.
- The season selector browses years from 2015 through next year; choosing one of four
  seasons loads it immediately. Refresh retains the selected season.
- Opens independent detail windows with centered content capped at 1,040 DIPs,
  artwork, synopsis, genres, cast, streaming
  links, and related websites. Links accept only HTTP(S) and open in the default browser.
- Uses native WinUI controls, a Mica backdrop, system theme brushes, Fluent typography,
  accessible control names, keyboard-operable buttons, a responsive weekday grid,
  and a native search ListView. The pinned Reactor API lacks
  grouped collection invocation, so schedule items use subtle native buttons in one
  scrolling surface. Cards show direct airtimes, an adult badge, up to two genres,
  and up to three small platform logos.
- Fetching runs outside the UI thread with network timeouts and loading/error/retry
  states. Failed refreshes preserve the previous successful catalog.

The workspace stores only a small Reactor patch for native `AutoSuggestBox.QuerySubmitted`
(Enter-to-search); see [patch notes](../../patches/windows-reactor/README.md).
Run setup with Python 3.12+ and Git before the first Cargo command in a fresh checkout.
It verifies the pinned crate archive and applies the patch into ignored `vendor/windows-reactor`.
It can reuse Cargo's cached archive offline and safely verifies an existing installation.
Because Cargo resolves workspace patches on every platform, this setup is required for
other workspace packages and Android builds too.

The published 0.100 API uses `Component`, `ViewContext`, `View`, and `App::run_component`.
Some examples on the windows-rs main branch describe a different hook-based API and
`windows-reactor-setup`; they do not apply to this pinned crate version.

Validation: catalog unit tests and a native Windows build. Visual layout, keyboard,
Narrator, light/dark/contrast themes, and live networking require runtime validation
on a machine with the matching Windows App Runtime installed.

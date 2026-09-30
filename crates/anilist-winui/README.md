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
cargo run -p anilist-winui
cargo build --release -p anilist-winui
cargo test -p anilist-winui
```

This is a framework-dependent Rust executable, not an MSIX or a .NET Reactor app.
The Windows App Runtime is a separate prerequisite; it is not bundled. Non-Windows
builds provide a small unsupported-platform message and can run the pure catalog tests.

## Behavior and design

- Loads the current season through the shared YourAnimes fetcher, including local
  timezone conversion. Refresh replaces the catalog instead of appending duplicates.
- Groups animation cards from today's weekday, sorts each day by airing time, and
  places unknown schedules last. Visible weekday filters and a local text filter
  narrow the displayed catalog by title, description, genre, or cast.
- The native title-bar search calls `AnimeFetcher::search()` across the source.
  Submit an empty search to return to the current season. Older requests cannot
  overwrite newer results.
- Opens independent detail windows with artwork, synopsis, genres, cast, streaming
  links, and related websites. Links accept only HTTP(S) and open in the default browser.
- Uses native WinUI controls, a Mica backdrop, system theme brushes, Fluent typography,
  accessible control names, keyboard-operable buttons, and a responsive scrolling GridView.
- Fetching runs outside the UI thread with network timeouts and loading/error/retry
  states. Failed refreshes preserve the previous successful catalog.

The published 0.100 API uses `Component`, `ViewContext`, `View`, and `App::run_component`.
Some examples on the windows-rs main branch describe a different hook-based API and
`windows-reactor-setup`; they do not apply to this pinned crate version.

Validation: catalog unit tests and a native Windows build. Visual layout, keyboard,
Narrator, light/dark/contrast themes, and live networking require runtime validation
on a machine with the matching Windows App Runtime installed.

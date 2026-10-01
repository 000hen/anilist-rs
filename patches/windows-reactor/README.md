# Local QuerySubmitted patch

The patch targets the published `windows-reactor` 0.100.0 crate from microsoft/windows-rs,
commit `a57cc321bfdbac6bb98c488ba49f399da8fcb98a`, `crates/libs/reactor`.
Only `query-submitted.patch` is stored in version control. Run
`python scripts/setup-windows-reactor.py` from the workspace root with Python 3.12+
and Git before using Cargo, including for non-Windows packages. The script downloads
the pinned crate (or reads Cargo's cached archive), checks its SHA-256, and applies
the patch into ignored `vendor/windows-reactor`. Original MIT and Apache-2.0
licenses are retained in that directory. Cargo uses it through the workspace
`[patch.crates-io]` entry; the registry cache is unchanged.

Rerunning setup verifies the existing files and refuses to overwrite local edits.
When updating the patch or crate version, move the generated directory aside and
rerun setup. Update the pinned version and archive checksum in the script together
with the workspace dependency, then regenerate the patch against the new archive.

The only source changes are in:

- `src/generated.rs`: expose `AutoSuggestBox::on_query_submitted` as a String
  callback, with event descriptors, subscription visitation, and callback dispatch.
- `src/native/winui/generated.rs`: subscribe to WinUI QuerySubmitted and enqueue
  its QueryText through the existing revision-checked event pipeline. The normal
  EventRevoker owns subscription removal.
- `src/native/winui/bindings.rs`: add the QuerySubmitted event slots and event-args
  projection. These generated-file edits are intentional until upstream exposes it.

ABI was checked using `System.Reflection.Metadata` against Microsoft.UI.Xaml.winmd
(SHA256 `7a48d821adb8ac4e1da64c69b7ecd570846aec42730b706ba973943b8bfb6692`).
IAutoSuggestBox (`3eea809e-b2db-521d-97db-e0648fb5d798`) declares add/remove
QuerySubmitted immediately after add/remove TextChanged. The event args interface
(`26da5de4-57a6-57bf-acc9-aac599c0b22b`) declares QueryText (HSTRING) followed by
ChosenSuggestion (IInspectable).

Anilist uses the submitted text directly and never installs a global Enter shortcut.
The app's component test verifies native event dispatch and ignores events from a
retired control. Remove this patch, setup script, and Cargo override once the
upstream API provides the event.

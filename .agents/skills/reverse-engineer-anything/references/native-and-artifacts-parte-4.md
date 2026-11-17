---
type: Guide
title: "REA upstream native-and-artifacts.md 4/5"
description: "Referência oficial REA v61, revisão 25d3c7237ffeffc4f9e6ab87be2f83a643cf5cbb"
---

for one selected type. For questions about a selected type, select
`detail: "summary"` with `type_names`; it returns the selection (with unmatched
names), coverage and per-collection counts. Page rows with
`view: {kind: "managed-members", collection: "call_edges", offset: 0, limit: 64}`
over `types`, `exported_types`, `fields`, `methods`, `member_refs`, `call_edges`,
`field_accesses` or `user_strings`.
Use exact `names` and `declaring_types` filters before paging supported
collections. For a compact method lookup, request `fields: ["name",
"declaring_type", "signature"]`; omit `body` to avoid anchors. Identity tokens,
original `source_index`, module/MVID and source coverage remain. Check
`unresolved_filter_rows` before interpreting an empty match as absence.

### .NET NativeAOT

NativeAOT binaries contain native machine code, so ordinary CIL decompilers
cannot recover the original method bodies. When managed metadata/CIL is absent,
classify that as a native-analysis route rather than claiming source recovery.
Open the native image with the selected deep provider and inspect functions,
strings, calls, and references; describe decompiler output as pseudocode and
inference.

NativeAOT images can be PE/COFF, ELF, or Mach-O executables and shared
libraries. Use file bytes and loader metadata rather than suffixes to identify
them. For packaged apps, inventory/extract with REA's artifact tools and retain
the exact contained image identity. PDB, ELF debug, or dSYM sidecars are
additional evidence only when the provider accepts and matches them.

REA has a built-in read-only snapshot parser for x64 PE targets with .NET 8 RTR
9.1 metadata. An optional headless Ghidra adapter adds database annotations and
broader metadata recovery on Linux; it requires an explicitly configured
NativeAOT JAR. These are separate routes. The built-in parser also runs on
Windows Ghidra P0; read frozen literals from its report, because
`search_strings` does not include them. See the
[NativeAOT guide](https://github.com/morluto/rea/blob/main/docs/providers/ghidra-nativeaot.md)
for setup, supported layouts, and host restrictions. Preserve binary identity
and distinguish derived metadata from Ghidra database observations. Recovered
types and native pseudocode are not original C# source. Windows Ghidra P0 has
no database mutation authority.

## Packages and extraction

Use `inspect_artifact` for application bundles, archives, ZIP/APK/IPA/MSIX/AppX,
ASAR, or DMG inputs when the artifact graph and findings help answer the
question. It returns the complete artifact graph inline. Cite graph manifest
IDs when using them.

For an IPA or a macOS `.app`, ZIP, or DMG, pass the inventory Evidence to
`project_apple_application_graph`. It reports application roots and nested
bundles by path convention: app extensions, XPC services, frameworks, login
items, system and driver extensions, and plug-ins. It also lists privileged
helpers, launchd plists, symlinks, and each bundle's `info_plist_path` and
executable candidates. Roles are path conventions, not parsed plists; read the
listed plists with `inspect_plist`.

After `open_binary` on a `.app` or Mach-O, use `trace_dylib_resolution` to see
which file each `@rpath`, `@loader_path` and `@executable_path` load reaches for
every executable in the bundle. It also lists missing or weak loads and earlier
`@rpath` candidates that are absent. Narrow large bundles with `roots` or
`architecture`. System paths stay undetermined because the dyld shared cache
provides them.

Use `extract_artifact` when materialized files from a supported format are needed.
It extracts the active target and accepts optional `integrity_policy`; see the [MCP contracts](https://github.com/morluto/rea/blob/main/docs/reference/mcp-contracts.md).
It materializes all regular files into a fresh temporary directory chosen by
REA. Symlinks and encrypted entries are inventory facts, not extractable files.
The result reports the materialized directory; callers do not choose the
destination. No separate permission grant is needed.

For classic iNES cartridges, `open_binary` recognizes the byte signature without
an executable architecture. `inspect_artifact` retains header facts and each
PRG/CHR file unit's offset, length and SHA-256; its nested inventory's
`ines.reconstruction` verifies the actual ordered region bytes against the
whole-file digest. Extraction includes the original header, optional trainer
and uninterpreted trailing bytes. These file units are not mapper/CPU bank
maps. NES 2.0, UNIF, dirty legacy headers and truncated payloads are explicitly
refused. No emulator or 6502 analysis is performed. See the
[iNES guide](https://github.com/morluto/rea/blob/main/docs/ines-containers.md).

Native DMG traversal automatically uses a read-only mount on supported macOS
hosts, with an owned temporary mount directory and cleanup. It needs no approval
flag. Unsupported hosts or mount failures remain explicit limitations; do not
claim child inventory when only root identity is available. DMG extraction is
unavailable through `extract_artifact`. To access DMG files on macOS, create a
read-only mount with your own tools, select files from that mount for analysis,
and detach the mount you created when finished. REA's inspection mount is
temporary, not a retained export of the image's contents.

For headerless DOS COM, explicitly open with `format: "dos-com"` (CLI
`--target-format dos-com`). Inspect the load image before trusting function
analysis: the entry is `0x10100`, file offset zero, with imposed real-mode segment
context. PSP/stack/device state remains unmodeled. See [DOS guide](https://github.com/morluto/rea/blob/main/docs/providers/ghidra-dos.md).

Use `inspect_byte_entropy` to triage high- and low-entropy ranges before extraction.

For other headerless images (embedded firmware without ELF/Mach-O/PE/MZ
headers), open with `format: "raw-binary"`, `architecture` (`x86`, `x86_64`,
`arm`, `arm64`, `6502`, or `m68k`), and `entry_point` (`0x` hex). File offset 0 is mapped at
that address. Hopper and IDA refuse this interpretation; Ghidra uses
`BinaryLoader` with a generic language: big-endian for `m68k`, little-endian
otherwise. Big-endian ARM, Thumb-only, and Cortex-M variants are unsupported.
`6502` selects NMOS 6502; the entry and complete file mapping must fit within
`0x0000..0xffff`. 65C02, 65816, cartridge containers, bank switching, and mapper
semantics are unsupported. `m68k` requires an even entry and decodes the 68040
superset of the 68000 family; Amiga/Atari containers and vector-table entry
discovery are unsupported. Windows Ghidra P0 admits raw images only for `6502`. See [raw binary guide](https://github.com/morluto/rea/blob/main/docs/providers/raw-binary.md).

Intel HEX, S-record, UF2 and DfuSe files are containers, not raw images:
`inspect_artifact` lists address regions (UF2 per familyID, DfuSe per element
with its alternate setting), and `extract_artifact` writes each region. Open an
extracted region as `raw-binary` with its start address as `entry_point`. UF2
numbering gaps leave coverage unknown; a DfuSe suffix CRC mismatch follows
`integrity_policy`. See [UF2 and DfuSe](https://github.com/morluto/rea/blob/main/docs/uf2-dfuse.md).

### Function annotations in Ghidra

Fresh-import sessions admit annotations; existing-project reuse is read-only.
Use `annotate_native_function` for one function name and/or entry comments, with at least one explicit change. Review its annotation readback and refreshed dossier inline. Changes are atomic; empty comments clear them, omitted fields preserve them. Later MCP calls observe edits. Normal close saves them when owned Program checkpoint mode is enabled; ephemeral mode discards them. CLI `annotate-native-function` follows the same setting when closing after its updated result. Select the [checkpoint workflow](#continue-editable-ghidra-analysis-across-processes) before edits that must survive a later process. Original executable bytes are unchanged; edits invalidate immutable snapshots. Windows P0 does not admit database mutations. For naming only, `set_address_name` / `set_addresses_names` also work with Ghidra: they name function entries or data addresses (primary labels) and return compact per-address success booleans instead of a dossier. Ghidra rejects batch addresses that request the same name or a name another selected address holds; `raw_result.failures` gives each failure's reason.

A provider switch or different target can retire the active Ghidra database and discard
its edits, even if the new provider fails on a later call. Same-target,
same-profile reopen keeps the active database. Retained annotation Evidence
records what happened; it does not restore the edits.

For `compare_functions`, obtain `analyze_function` Evidence before and after the
edit while the MCP session is open; annotation Evidence is not a comparison
input. Export the needed Evidence before close if comparison will happen later.
Authenticated same-artifact coordinates preserve renamed entries as
`matched / same-entry`, with the canonical `entry_address` inline; name and
exact-text changes remain separate dimensions. This requires compatible
architecture, provider-owned mapping and selected-image commitments. Equal
numeric addresses alone cannot establish correspondence; inspect retained
source Evidence and coverage when the match remains ambiguous.

## Offline PE resource inventory

Use `inspect_pe_resources` with an absolute `path`, or
`rea inspect-pe-resources ./selected.exe --json`, for portable PE32/PE32+
resource inspection without opening a native target. Preserve returned Evidence:
type/name/language IDs and UTF-16 names, original directory/data/payload ranges,
code pages, payload hashes and static icon-group references. Same-language icon
resolution is explicit; missing references and size mismatch remain visible.
Other payloads are opaque, and this operation does not decode images or simulate
Windows resource loading. Consult the budgets and supported tree profile in
[offline binary diagnostics](https://github.com/morluto/rea/blob/main/docs/guides/native/binary-diagnostics.md).

## Offline PE and ELF export tables

Use `inspect_native_exports` with an absolute `path`, or
`rea inspect-native-exports ./libexample.so --json`, to read a PE export
directory or ELF dynamic symbol table on any host without opening a native
target. Ordinal-only, forwarded and hidden-version exports stay distinct;
partial or unsupported coverage means the inventory is incomplete, not that
exports are absent. Use `inspect_macho` for Mach-O. See
[offline binary diagnostics](https://github.com/morluto/rea/blob/main/docs/guides/native/binary-diagnostics.md).

For exact core-module import/export differences, use `compare_wasm_interfaces`
with two complete inspection Evidence records (or current-session retained
references) and explicit scope. Preserve unknown types/duplicate names; static
interface equality is not runtime compatibility. CLI accepts complete inline
Evidence in `rea compare-wasm-interfaces input.json --json`, or exact references
with `--evidence-bundle /absolute/path/evidence.json`. Use the complete parent
ID when the inspection returned a summary.

## Godot PCK

For a standalone PCK, `open_binary` then `inspect_artifact` with
`detail: "summary"` retains the complete static inventory. Use the returned
`parent_evidence_id` with `inspect_analysis_view` inventory occurrences filtered
by exact `logical_paths` or `occurrence_ids`, and join `artifact_id` to nodes
filtered by `artifact_ids`. Select only needed fields; identity links remain.
Prefer `inspect-artifact --detail summary --evidence_output PATH` for CLI tasks;
the same CLI discovers embedded packs via their EOF trailer without opening a
native analysis provider. Normal native opening remains unchanged.

Distinguish declared MD5 from observed SHA-256 and graph/page completeness from

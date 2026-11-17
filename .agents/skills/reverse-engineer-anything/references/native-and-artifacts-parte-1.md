---
type: Guide
title: "REA upstream native-and-artifacts.md 1/5"
description: "Referência oficial REA v61, revisão 25d3c7237ffeffc4f9e6ab87be2f83a643cf5cbb"
---

# Native, managed, and packaged artifacts

For release comparisons on one MCP connection, retain `list_procedures` Evidence
from each target and reuse its exact IDs after target switches or `close_binary`.
For a handoff or new process, export the complete bundle and import it there,
including its source closure. `reset_investigation` explicitly clears retained
records; optional `save_path` checkpoints them before clearing.
`match_native_functions` pairs unique exact names and, when supplied,
complete observed body bytes. Reuse optional same-context `list_names`,
`procedure_info`, `read_bytes` and existing `analyze_function` dossiers as
`left_observations`/`right_observations`; an MCP `analyze_function` summary
reply contributes its `parent_evidence_id` as a `retained-evidence` reference,
not the summary itself. Avoid decompiling every function merely to match it. Its summary retains a parent for
`inspect_analysis_view` `function-matches` collections `matches` and `functions`.
Select columns, batch IDs and page individual facts; aliases remain unresolved.
Unpaired observations do not prove additions/removals. Existing matched dossier
pairs include inputs for `compare_functions`; otherwise analyze just chosen pairs.

Aggregate completed comparisons with `find_changed_behavior`, selecting each
comparison's exact `retained-evidence` ID instead of resending full records.
Inline comparison Evidence remains accepted. References belong to the current
server: after handoff, import the original bundle including its source closure.
Do not substitute dossier IDs or summary views. Static differences remain
behavior candidates, with runtime behavior unknown unless separately observed.

For a matching representation refusal, inspect `details.limit` and raise
`native.matching_representation_bytes` with `update_analysis_settings` before
retrying the unchanged MCP request. CLI launch seed:
`REA_NATIVE_MATCHING_REPRESENTATION_BYTES`. The default is 96 MiB; the policy
range is 1 byte to 1 GiB and does not bound process RSS. Keep the complete
inventories: splitting them changes global name/body uniqueness.

## Apple app resources

Use `open_bundle_resources` with the selected app path, then `inspect_plist`,
`decode_interface_builder`, `inspect_asset_catalog`, or `inspect_keyed_archive`.
The resource session reads the app without resolving `CFBundleExecutable` or
starting a deep provider. Preserve `resource_root` provenance, per-resource
hashes where reported, and null executable identity; a directory path is not
an executable digest. Export retained Evidence before closing or switching
roots. Binary analysis snapshots do not apply. Use `open_binary` to switch to
executable analysis, and `close_binary` to release either session kind.

The four matching CLI commands accept the app path directly. For an archive
inside the app, use `inspect-keyed-archive APP_PATH RELATIVE_ARCHIVE_PATH`.
Host requirements for plist and compiled asset/Interface Builder tooling still
apply; a resource-only session does not establish provider availability.

## Desktop installation directories

When an installation directory is supplied and its runtime is unknown, use
`inspect_desktop_application` with its absolute path. Default MCP summary
delivery retains complete Evidence. Read `files`, `omissions`, `candidates`, `components` or
`next_probes` with `inspect_analysis_view` kind `desktop-application`, selecting
exact paths, batched file/omission IDs or families and only needed fields and pages.
Summary `probe_issues` retains counts and bounded previews. Select files with
`has_issues: true` and fields `path`, `read_status`, `issues` to retrieve exact
failures directly; preview text is display data, while `file_id` selects full facts. Preserve source
coverage, read-range identity and unknowns; directory snapshot identity is not
a digest of every file's contents. Component path markers do not establish
runtime presence. For compact CLI output, use
`inspect-desktop-application PATH --detail summary --evidence_output BUNDLE`.
See [desktop installations](https://github.com/morluto/rea/blob/main/docs/desktop-applications.md).

## Go build metadata

Use `inspect_go_binary` for an explicit local ELF, PE or Mach-O (universal
files need `architecture`) when the
question concerns the embedded compiler version, module/dependency versions,
local replacements or build settings. No `open_binary`, Go installation or
target execution is required. The CLI equivalent is `inspect-go-binary PATH`.
Preserve artifact SHA-256, original file ranges, framed module bytes and every
unparsed module-text line. A missing record is an unknown, not proof of a
non-Go binary; observed metadata does not authenticate its claims. Function,
source and type recovery remain a separate native-analysis question.

## Model files

Use `inspect_model_artifact` (`inspect-model-artifact PATH`) for local GGUF v2/v3
and safetensors declarations without a model runtime. Header-only mode does not
read weights or establish full-file identity. `hash: "full"` explicitly streams
all bytes. Retained `model-artifact` views page `metadata`, `tensors` and derived
`types`, with exact batch filters and selected fields. Arrays retain typed counts
and eight-element previews; omitted values, unknown types and runtime behavior
stay explicit. See [model files](https://github.com/morluto/rea/blob/main/docs/model-artifacts.md).

## IL2CPP metadata

Use `inspect_il2cpp_metadata` for an explicit local Unity `global-metadata.dat`
(versions 24, 27, 29, 31 and 39) when the question concerns assembly images,
names, member owners, string literals, namespaces or index ranges. No `open_binary`, Unity installation or target
execution is required. The CLI equivalent is `inspect-il2cpp-metadata PATH`.
MCP defaults to counts with complete retained Evidence. Query `sections`,
`images`, `types`, `methods`, `fields`, `properties`, `events`, `parameters` or
`string_literals` through `inspect_analysis_view`; use exact names,
batch indices and selected fields. Row indices are scoped by artifact SHA-256
and collection; page offsets count filtered matches. Parameters support batch
`method_indices` to read one method's parameters directly. Historical reports
without member collections retain unknown counts rather than empty inventories. Preserve
artifact SHA-256 and index widths. Unrecognized magic may mean encryption and
is unsupported, not decrypted. Linking names to `GameAssembly` or
`UnityFramework` code addresses remains a separate native-analysis question.
See [IL2CPP metadata](https://github.com/morluto/rea/blob/main/docs/il2cpp-metadata.md).

## Rust dependency metadata

Use `inspect_rust_binary` for an explicit local ELF, PE, or Mach-O (universal
files need `architecture`) when
the question concerns the cargo-auditable crate list embedded in `.dep-v0`
(Mach-O: `__DATA,.dep-v0`). No `open_binary`, Rust installation, or target
execution is required. The CLI equivalent is `inspect-rust-binary PATH`.
Preserve artifact SHA-256, package order, dependency indices, and the section
file range. A missing section is null metadata, not proof the binary has no
dependencies. An omitted package kind stays null. A rustc commit hash is
inferred from a read-only `/rustc/<40-hex>/` path and does not by itself
identify the compiler. See [Rust binaries](https://github.com/morluto/rea/blob/main/docs/rust-binaries.md).

## PE Authenticode certificate tables

Use `inspect_pe_certificates` for an explicit local PE32/PE32+ image when the
question concerns the Authenticode certificate table, WIN_CERTIFICATE types, or
embedded PKCS#7 signer and X.509 identity fields. No `open_binary`, Windows SDK,
or target execution is required. The CLI equivalent is
`inspect-pe-certificates PATH`. Preserve artifact SHA-256, table file ranges,
payload hashes, original validity-time spellings, and
`subject_key_identifier_hex` when the signer uses that form. A missing
`IMAGE_DIRECTORY_ENTRY_SECURITY` directory is observed absence of that table,
not proof the image is unsigned in every catalog or overlay sense. Signatures,
chains, timestamps, and Authenticode digest coverage remain unobserved. See
[PE Authenticode certificate tables](https://github.com/morluto/rea/blob/main/docs/pe-certificates.md).

## PE import and delay-load tables

Use `inspect_pe_imports` for an explicit local PE32/PE32+ image when the
question concerns imported DLL and function names, ordinals, delay-load
descriptors, or the pefile import hash. No `open_binary`, Windows SDK, or
target execution is required. The CLI equivalent is `inspect-pe-imports PATH`.
Preserve artifact SHA-256, directory file ranges, original DLL spellings, and
ILT/IAT RVAs. A missing `IMAGE_DIRECTORY_ENTRY_IMPORT` directory is observed
absence of that table, not proof the image never imported APIs through another
table or overlay. Bound snapshots, loaded IAT contents, and export-forward
resolution remain unobserved. See
[PE import and delay-load tables](https://github.com/morluto/rea/blob/main/docs/pe-imports.md).

## PE TLS callbacks and load configuration

Use `inspect_pe_load_config` for an explicit local PE32/PE32+ image when the
question concerns code that runs before the entry point through TLS callbacks,
or the security cookie, SafeSEH, CFG, EH-continuation and XFG declarations in
the load configuration. No `open_binary`, Windows SDK, or target execution is
required. The CLI equivalent is `inspect-pe-load-config PATH`. Preserve artifact
SHA-256, directory and callback-array file ranges, and callback VAs as
hexadecimal strings. Fields outside the declared `Size` are `null`. Guard-flag
mitigations carry `evidence_kind: "inferred"`; enforcement remains unobserved.
See
[PE TLS callbacks and load configuration](https://github.com/morluto/rea/blob/main/docs/pe-load-config.md).

## Binary hardening

Use `inspect_binary_hardening` for an explicit local ELF, PE, or Mach-O file
when the question concerns PIE or ASLR, NX, RELRO, stack-protector or FORTIFY
signs, control-flow flags, or stripped state. No `open_binary`, pwntools, or
target execution is required. The CLI equivalent is
`inspect-binary-hardening PATH`. Universal Mach-O files return one image per
slice. A missing import is `unknown`, not proof the protection was omitted.
Authenticode and Mach-O code-signature rows report presence, not validity.
`hardened_runtime` stays unknown and names `inspect_signature`. See
[binary hardening](https://github.com/morluto/rea/blob/main/docs/binary-hardening.md).

## ELF note tables

Use `inspect_elf_notes` for an explicit local ELF32/ELF64 image when the
question concerns PT_NOTE/SHT_NOTE owners, GNU build-id, or GNU ABI-tag
fields. No `open_binary` or target execution is required. The CLI equivalent
is `inspect-elf-notes PATH`. Preserve artifact SHA-256, table file ranges,
descriptor hashes, and original owner spellings. A missing table is observed
absence of those declarations, not proof the image never carried notes in
another container. See
[ELF note tables](https://github.com/morluto/rea/blob/main/docs/elf-notes.md).

## Apple IPS crash reports

Use `inspect_apple_ips` for an explicit local Apple `.ips` file when the
question concerns crash metadata, exception and termination facts, threads,
usedImages, lossless registers, or documented crash classes. No `open_binary`,
Xcode, atos, or target execution is required. The CLI equivalent is
`inspect-apple-ips PATH`. Preserve artifact SHA-256, `bug_type` as a string,
uppercase image UUIDs, and integer literals that exceed IEEE-754 range. Only
`bug_type` `"309"` yields a crash body; other DiagnosticReports types return
no crash result. Opened Mach-O and dSYM UUID joins remain unobserved. See
[Apple IPS crash reports](https://github.com/morluto/rea/blob/main/docs/apple-ips.md).

## Linux kernel modules

Use `inspect_kernel_module` for an explicit local ELF32/ELF64 relocatable
object when the question concerns `.modinfo` license, vermagic, alias,

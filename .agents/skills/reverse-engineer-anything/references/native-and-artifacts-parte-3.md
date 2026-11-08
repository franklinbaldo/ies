---
type: Guide
title: "REA upstream native-and-artifacts.md 3/5"
description: "Referência oficial REA v61, revisão 25d3c7237ffeffc4f9e6ab87be2f83a643cf5cbb"
---

For a task spanning agent sessions, check `binary_session` after selecting
Ghidra: the current provider's `capabilities[].shared_limitations` states whether
Program checkpoints are enabled or the session is ephemeral. Successful native
queries distinguish initial analysis from an actually restored packed Program;
do not infer that restoration happened merely because checkpoint mode is enabled.

In ephemeral mode, choose a private absolute directory and set
`REA_GHIDRA_CHECKPOINT_DIR` in the environment starting REA. Restart the MCP server
after changing that launch configuration; no current tool call or runtime
analysis-setting update enables it. Stock ELF/PE/Mach-O imports on Linux/macOS
are supported; see the [checkpoint setup and limitations](https://github.com/morluto/rea/blob/main/docs/getting-started/installation.md#automatically-checkpoint-an-owned-ghidra-program).

With the setting enabled, normal opens restore compatible saved Programs
automatically. Initial completed analysis is saved, and `close_binary` saves
later metadata edits; no separate save/restore tool is needed. Repair a reported
save failure and retry close. Cancellation or process failure can lose edits
after the last successful save; unfinished initial analysis cannot resume.
Keep the same checkpoint configuration for later processes. Evidence/Unknown
history and immutable REA query snapshots have separate persistence workflows.

## Reuse caller-selected Ghidra analysis

When the caller supplies an expanded project, configure `REA_GHIDRA_PROJECT_PATH`
(the `.gpr` marker) and optionally `REA_GHIDRA_PROGRAM_NAME` (one root program),
then restart/reconnect before `open_binary`. CLI uses the same configuration.
REA opens only a fingerprint-bound owned copy with `-process -noanalysis -readOnly`;
original project files are not opened by the engine or saved. Database edits,
import overrides and repair scripts are unavailable in this mode. Treat original
analysis completeness/settings as unknown and prior edits as caller-project state.
The program's recorded import digest must match the target; changed project content
requires reopening for a fresh profile. See the
[setup and preparation guide](https://github.com/morluto/rea/blob/main/docs/getting-started/installation.md#reuse-an-existing-ghidra-project).

## Ghidra startup deadline recovery

`provider_timeout` during the first Ghidra query can cover import, auto-analysis,
bridge startup, and readiness. The default 330,000 ms is configurable through
`REA_GHIDRA_STARTUP_TIMEOUT_MS`; a smaller query still needs the same cold import.

When the error names `ghidra.startup_timeout_ms`, call
`update_analysis_settings` with `action: "set"` and a larger value within the
reported bounds. Resolve retained cleanup before the next startup attempt.
No REA restart is needed; an already accepted startup keeps its original
budget. Follow the error's session-specific restart guidance, and preserve useful Evidence before closing
and reopening the target when remediation requires that lifecycle step.

For a per-function timeout, update `ghidra.decompile_timeout_seconds` and retry
the same function on the existing Program. Explicit `null` removes that
per-function deadline. Inspection is optional when the error supplies the
setting and bounds. Runtime updates disappear when REA exits; environment
changes only seed future REA processes.

The client's tool-call deadline must allow startup plus the requested query;
server settings and progress notifications cannot override it. CLI analysis
avoids the MCP client deadline, but retains the provider deadline. For example:

```sh
REA_GHIDRA_STARTUP_TIMEOUT_MS=1800000 rea analyze /absolute/path/to/app.exe --provider ghidra --json
```

Thirty minutes is an example budget, not a completion guarantee. Keep host CPU
and memory controls in place. Disable specific expensive analyzers only when
their missing coverage is acceptable and retain the resulting limitations.
Exported C and embedded resources remain useful independent evidence, but a
timeout does not establish complete analysis or absence of code. See the
[deadline and client guide](https://github.com/morluto/rea/blob/main/docs/reference/mcp-contracts.md#ghidra-first-query-deadlines-and-recovery).

## Bridge request ceiling

Hopper and Cutter share `REA_BRIDGE_MAX_REQUEST_BYTES`. Unset, both sides use
16777216 bytes. The payload budget excludes the terminating LF. A decimal
integer from 7464 to 536870887 raises or lowers that ceiling. The floor fits
the largest bounded symbol-name command. A tight Cutter status still returns
its session id and up to 64 completion receipts. Its path is included when it
fits, and is unknown when it does not, so an older descriptor path is not
paired with the new generation. The maximum is the encoded payload, one byte under Node's string ceiling so
the framing LF still fits. Cutter measures, signs, and writes a fitting
response in bounded chunks. The bridge searches only the newly read bytes
for the framing LF. The client unescapes each socket chunk into one response
string before parsing it, so this ceiling does not keep the paged wire frame
and another concatenated copy. Invalid values are configuration errors and do not fall back to 16777216. Set the variable in the environment that starts REA, restart REA,
and start Hopper or Cutter from that same environment. An out-of-band bridge
without the setting stays at 16777216 bytes. The client reports that mismatch
during discovery only after the hello proves a live authenticated bridge, and
does not send a status request or the command. A stale descriptor is ignored.
Discovery fails if any such live ceiling differs, even when another live
session matches. A command response discarded for that ceiling stays an
unknown execution with `output_truncated`. A hello
that never sends its authenticated frame does not hold the accept loop.
At most 8 connection threads run; a further accept is closed, and a failure
to start a thread does not stop the listener. `completed_connection` is the
longest prefix of connections that have terminated. Dense scalar replies are
encoded in bounded batches.
The 25000 ms Cutter request deadline and the 256-candidate discovery budget
still have no runtime override.

## Selected native facts and call graphs

For a large `get_call_graph`, or repeated `procedure_address`,
`procedure_callees`, `procedure_callers` and `resolve_containing_procedure`
queries, select `detail: "summary"` when the first 64 recorded observations
suffice. Read further `native-observation` pages from the returned parent Evidence
ID. Graph message IDs index that page's exact-text dictionary; keep level,
source index, failed nodes, and partial/unknown relationship coverage.
Read referenced source context through `native-context` facets `overview`,
`analysis-profile`, `limitations` and `result-limitations`. Reuse inspected
context/profile values only for the same digest; until caveats are read,
interpretation beyond the returned facts remains unknown. Complete Evidence
remains retained and exportable. Small complete replies can cost fewer tokens
than additional context/page calls; count the completed task and discovery.

## Static memory and original file bytes

For Ghidra, `read_bytes` reads initialized analysis memory, including loader
fixups; it does not read a running process. Check `returned_bytes` and `complete`
before decoding a table. An incomplete read is missing coverage, not a zero-filled
tail. Use `list_segments` to inspect the selected block's initialization and address-space
facts, and retain unavailable facts as unknown.

Use `address_to_file_offset` to anchor one observed address to original file
bytes. A mapping for the first address does not establish a contiguous mapping
for an entire table, and loader fixups can make analysis bytes differ from source
bytes. Keep source ranges and analysis-memory observations separate.

A pointer table can refer to writable slots populated by a constructor rather
than directly to immutable resources. An uninitialized block or an observed zero
does not establish the slot's later runtime value. Follow `xrefs` and the relevant
writer's assembly or pseudocode; distinguish recorded assignments from inferred
execution order. Do not replace unreadable bytes with zeros or conclude that a
resource is absent without the necessary initialization evidence.

## DOS MZ with Ghidra

For an admitted DOS MZ target, use `inspect_native_load_image` to check measured
loaded bytes, file mappings, relocations and the entry against the immutable
snapshot. Keep `mismatch` and `unsupported` explicit. Verified import covers the
reported static image, not runtime DOS or PC-98 hardware behavior.

For a large load-image report, select `detail: "summary"` and query its
`parent_evidence_id` with `inspect_analysis_view` kind `native-load-image`.
Choose a collection such as `relocations` or `mappings`, select only needed
fields, and page with offset/limit. Original indices preserve same-address
alternatives. Null collections remain unrecorded or inapplicable; paging does
not upgrade unsupported verification. CLI requires `--evidence_output` for
summary delivery so complete source Evidence remains accessible.

For Ghidra decompilation, `procedure_pseudo_code` also accepts `procedure: "file-offset:0x..."`; CLI `rea decompile <path> --file-offset=0x... --provider ghidra` resolves the source byte offset internally. An unresolved 0x-prefixed numeric procedure selector also falls back to a unique file mapping. Existing symbols and analysis addresses win; multiple mappings fail as ambiguous. Preserve the returned file-offset and address locations and raw selection observations. ELF virtual addresses are distinct from file offsets; never assume an arbitrary offset plus image base is an analysis address.

Complete function body ranges use inclusive ends and can contain gaps. Do not
treat the enclosing span as code.
For packed targets, retain the original and separately derived artifact identities;
decompiling an unpacking stub does not recover the unpacked program.

When reading Objective-C dispatch metadata, distinguish class implementation
methods from protocol requirements: class methods report null requirement
flags, while decoded protocol declarations retain required and optional facts.
Property attributes remain raw name/value tokens; use the property's
`atomicity` field for the decoded `N` marker or the default atomic behavior.
Null class/meta flags, ivar counts, atomicity, or read-only values mean the
selected provider did not establish that fact. Symbol names alone do not prove
that a class is a root class or has no ivars.

## Managed PE/CLI

Start with `inspect_managed_artifact`. REA's canonical managed inspection is
execution-free: do not claim it loaded, reflected, executed, or resolved the
assembly. Keep managed/native boundaries and unavailable reconstruction facts
explicit. A bring-your-own reconstruction oracle is separate from the canonical
parser and must not become an implicit setup dependency.

For identity/classification questions, `inspect_managed_artifact` can explicitly
select `detail: "summary"` on the first MCP call. It retains complete metadata
and returns a linked overview with full classification/evidence, nullable
module/MVID, frameworks, metadata coverage/issues/limitations and returned-row
counts. Default complete requests remain unchanged. Read original rows with
`inspect_analysis_view`, source `normalized_result.parent_evidence_id`, and
`view: {kind: "managed", collection: "attributes", offset: 0, limit: 16}`.
Collections are `references`, `resources` and `attributes`. Paging does not
repeat parsing. Returned-row exhaustion
does not prove complete metadata or absence under partial/unavailable coverage;
preserve build-local tokens and their artifact/MVID scope.

`inspect_managed_members` always returns the assembly-wide `member_refs`, even

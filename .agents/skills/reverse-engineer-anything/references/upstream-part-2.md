---
type: Guide
title: "REA upstream — parte 2/4"
description: "Trecho fiel da skill REA v61 upstream 25d3c7237ffeffc4f9e6ab87be2f83a643cf5cbb"
---

  limit for complete matching output. Preserve the source digest across pages.
  This does not perform live capture.
  For address/DNS/SNI/protocol summaries use `inspect_pcap_capture` with MCP
  detail summary, then retained kind pcap pages. Requires Wireshark 4.x tshark
  and capinfos, with Linux prlimit or Windows Job Objects; see
  [packet summaries](https://github.com/morluto/rea/blob/main/docs/guides/data/pcap-analysis.md).
  Select an exact DNS question for address pages; page exhaustion does not prove
  network completeness. This reads recordings and does not acquire traffic.
- Decoder discovery: `inspect_signal_decoders` searches installed decoder IDs
  or selected decoder help, including channels and options. Query and page the
  relevant details before choosing settings; this does not scan hardware.
- Recorded logic/signal traces: `decode_signal_recording` with explicit decoder,
  channel and protocol settings. On Linux with util-linux prlimit, supply sigrok-cli 0.7.2 through
  `REA_SIGROK_COMMAND`; see [recording guide](https://github.com/morluto/rea/blob/main/docs/guides/signals/signal-recordings.md).
  Output defaults to 50 literal lines; select annotation classes and page with
  `line_offset` / `line_limit`, or explicit null for all remaining output.
  Window counts are display lines, not annotations or samples. Preserve source digests across pages.
  This reads existing files; it does not acquire hardware signals.
- Explicit local WebAssembly artifact: `inspect_wasm_artifact`. Supply WABT 1.0.42
  through `REA_WABT_BIN_DIRECTORY`; retain exact byte/tool identity and decoded WAT.
  Select JavaScript glue and candidate paths explicitly and preserve URL/path
  ambiguity. No implicit fetch or target execution; WAT is not original source.
  See the [WASM guide](https://github.com/morluto/rea/blob/main/docs/guides/wasm/wasm-artifacts.md).
- Explicit local Android APK (or ZIP/AAB) signing: `inspect_apk_signature`.
  Prefer its summary or selected signer/digest/lineage page; retain complete
  Evidence and use `inspect_analysis_view` kind `apk-signature` for follow-ups.
  Reports which schemes are present (v1 JAR, v2, v3, v3.1), each signer's
  certificate SHA-256 fingerprint (v1 included, from its PKCS#7 block), v3 key-rotation lineage, source stamp, and
  whether each signed digest still matches the bytes (self-consistency). This
  is identity and integrity, not a trust or authenticity verdict.
  See the [APK signature guide](https://github.com/morluto/rea/blob/main/docs/apk-signature.md).
- Explicit local EVM bytecode carrier: `inspect_evm_interface` with caller-selected
  `raw` or `hex` encoding. The owned worker deadline is `timeout_ms` (default
  30000 ms, range 1..2147483647, next call). It does not raise the fixed byte,
  address-space, or CPU allowances. Preserve carrier/decoded digests and treat selectors,
  argument strings and mutability as inferred candidates; this performs no chain
  lookup or target execution. For the complete inferred interface without hex or
  duplicate stdout, discover support and select `detail: "summary"`; complete
  Evidence stays retained under its original identity. Use the returned
  `parent_evidence_id` for exact function or `bytecode_hex`/stdout/stderr pages.
  Candidate coverage does not establish interface completeness. See the
  [EVM bytecode guide](https://github.com/morluto/rea/blob/main/docs/guides/evm/evm-bytecode.md).
- Embedded Go compiler, module or build-setting metadata: `inspect_go_binary`
  with the explicit local file. This built-in target-free inspection requires no
  Go installation or disassembler. Preserve raw module bytes and source ranges;
  absent metadata does not prove a binary is not Go, and unparsed text records
  remain explicit. See the
  [Go binary guide](https://github.com/morluto/rea/blob/main/docs/guides/native/go-binaries.md).
- Embedded CUDA/HIP code objects: `inspect_gpu_code_objects`; inventory targets
  first, then decode selected payloads and query retained GPU Evidence.
- Bundled GGUF or safetensors model declarations: `inspect_model_artifact`
  with the explicit local file. Default header hashing leaves full-file identity
  unknown; `hash: "full"` additionally reads all weights for SHA-256. MCP returns
  retained summaries; optional `view` selects one page in the initial call. Use
  `inspect_analysis_view` kind `model-artifact` for
  batched metadata keys, tensor names or type totals, selected fields and pages.
  Preserve header-scoped indices and array preview omissions; do not infer model
  behavior from declared architecture or tensor types. See the
  [model file guide](https://github.com/morluto/rea/blob/main/docs/model-artifacts.md).
- Unity IL2CPP names, member owners and string literals from `global-metadata.dat`:
  `inspect_il2cpp_metadata` with the explicit local file; MCP defaults to compact
  counts. Query sections, images, types, methods, fields, properties, events,
  parameters or string_literals through `inspect_analysis_view`
  with the returned `parent_evidence_id`, using exact filters, batch indices and
  selected fields. Parameters accept `method_indices`, avoiding a scan of all
  parameter pages. Missing collections in historical Evidence remain unknown.
  Versions 24, 27, 29, 31 and 39; no Unity, IL2CPP
  binary or disassembler is run. Unrecognized magic (possibly encrypted) and
  other versions are unsupported. Optional matching `binary_path` enables static native addresses and type references; query `method_addresses` with indices or addresses/states, preserving every shared-body method and both artifact identities. Without a linked binary native counts remain unknown. Byte-pattern fallback is inferred; no runtime behavior or same-build pairing is proved. See the
  [IL2CPP metadata guide](https://github.com/morluto/rea/blob/main/docs/il2cpp-metadata.md).
- Embedded cargo-auditable Rust dependency metadata: `inspect_rust_binary`
  with the explicit local ELF, PE, or thin Mach-O file. This built-in
  target-free inspection requires no Rust installation or disassembler. Preserve
  package order, dependency indices, and the section file range. A missing
  `.dep-v0` section is null metadata, not proof the binary has no dependencies.
  A rustc commit hash is an inference from a read-only `/rustc/<40-hex>/` path.
  For selected crates, use `view: {collection: "packages", names: [...],
  fields: ["name", "version", "source"]}` on the first call; explicitly include
  `dependencies` for direct index references. Use `detail: "summary"` for counts
  and ranges, or retained `inspect_analysis_view` Rust package/edge pages for
  subsequent filters. Keep the complete parent Evidence accessible; CLI compact
  delivery requires `--evidence_output`.
  See the
  [Rust binary guide](https://github.com/morluto/rea/blob/main/docs/rust-binaries.md).
- PE Authenticode certificate table: `inspect_pe_certificates` with the explicit
  local PE32/PE32+ file. This built-in target-free inspection requires no
  Windows SDK. Preserve WIN_CERTIFICATE ranges, types, payload SHA-256, and
  PKCS#7 signer/X.509 identity fields, including `subject_key_identifier_hex`
  when the signer uses that form. A missing security directory is observed
  absence of that table, not proof the image is unsigned in every catalog sense.
  Signatures are not verified. See the
  [PE certificate guide](https://github.com/morluto/rea/blob/main/docs/pe-certificates.md).
- PE import and delay-load tables: `inspect_pe_imports` with the explicit
  local PE32/PE32+ file. This built-in target-free inspection requires no
  Windows SDK. Preserve DLL names, named and ordinal functions, ILT/IAT RVAs,
  delay-load descriptors, and the pefile import hash of the standard import
  table. A missing import directory is observed absence of that table, not
  proof the image never imported APIs through another table. Bound snapshots
  and loaded IAT contents remain unobserved. See the
  [PE import guide](https://github.com/morluto/rea/blob/main/docs/pe-imports.md).
- PE TLS callbacks and load configuration: `inspect_pe_load_config` with the
  explicit local PE32/PE32+ file. This built-in target-free inspection requires
  no Windows SDK. Preserve callback VAs as strings with their RVA, file offset
  and section, and the size-covered security cookie, SafeSEH, CFG,
  EH-continuation and XFG fields. Guard-flag mitigation claims are inferred
  linker declarations, not proof of enforcement. A missing directory is observed
  absence of that declaration. See the
  [PE load configuration guide](https://github.com/morluto/rea/blob/main/docs/pe-load-config.md).
- ELF, PE, and Mach-O hardening: `inspect_binary_hardening` with the explicit
  local file. This built-in target-free inspection requires no pwntools or other
  host tool. One image per Mach-O slice. Each property is `enabled`, `disabled`,
  `partial`, `unknown`, or `not_applicable`, with the header field or import
  behind it. A missing stack-protector or fortify import is `unknown`, not
  proof the protection was omitted. Authenticode and Mach-O code signatures
  report presence, not validity. Hardened runtime stays unknown and names
  `inspect_signature`. The image is not executed. See the
  [binary hardening guide](https://github.com/morluto/rea/blob/main/docs/binary-hardening.md).
- ELF note tables: `inspect_elf_notes` with the explicit local ELF32/ELF64
  file. This built-in target-free inspection requires no disassembler. Preserve
  PT_NOTE and SHT_NOTE ranges, owners, types, descriptor SHA-256, and GNU
  build-id or ABI-tag fields. A missing table is observed absence of those
  declarations, not proof the image never carried notes. See the
  [ELF notes guide](https://github.com/morluto/rea/blob/main/docs/elf-notes.md).
- Apple `.ips` crash reports: `inspect_apple_ips` with the explicit local
  file. This built-in target-free inspection requires no Xcode, atos, or
  pwntools. Preserve first-line metadata, `bug_type` as a string, process and
  exception facts, lossless integer registers, usedImages UUIDs, and
  documented classification matches. Only `bug_type` `"309"` yields a crash
  body; other types, including `"288"` stackshots, return no crash result.
  Opened Mach-O and dSYM UUID joins remain unobserved. See the
  [Apple IPS guide](https://github.com/morluto/rea/blob/main/docs/apple-ips.md).
- Linux kernel module metadata: `inspect_kernel_module` with the explicit local
  ELF32/ELF64 relocatable file. This built-in target-free inspection requires
  no Linux host or `modinfo`. Preserve `.modinfo` key order and duplicate keys,
  `__versions` CRC hex, section file ranges, and the appended signature
  trailer's PKCS#7 SHA-256. A missing section is observed absence when section
  names are known, not proof the module never carried that metadata.
  `struct module` is not decoded. The signature is not validated. gzip, xz, and
  zstd containers are unsupported; decompress into a separate file with an
  external codec utility, then inspect the decompressed `ET_REL` image. See the
  [kernel module guide](https://github.com/morluto/rea/blob/main/docs/kernel-modules.md).
- CPython 3.7–3.13 `.pyc` header, nested marshal code objects and bytecode
  identity: `inspect_python_pyc` with the explicit local file. This built-in
  target-free inspection requires no Python installation. Preserve magic,
  PEP 552 hash or mtime/size, nested names, and bytecode SHA-256. Bytecode is
  not decompiled, and a decoded header does not prove the matching `.py` is
  present. See the
  [CPython pyc guide](https://github.com/morluto/rea/blob/main/docs/python-pyc.md).
- Erlang BEAM IFF container: `inspect_erlang_beam` with the explicit local
  `.beam` file. This built-in target-free inspection requires no Erlang
  installation. Preserve chunk names, atom order, export/import/lambda tables,
  and the Code header. For selected facts, prefer `detail: "summary"` plus a

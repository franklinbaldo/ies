---
type: Guide
title: "REA upstream native-and-artifacts.md 5/5"
description: "Referência oficial REA v61, revisão 25d3c7237ffeffc4f9e6ab87be2f83a643cf5cbb"
---

`source_coverage`. Encrypted directories and sparse bundles have unknown entry
coverage; encrypted members have unknown content identity. An empty filtered
page cannot prove absence in unknown coverage. Retained views are historical,
not fresh filesystem checks. No decryption, Godot resource decoding, GDScript
decompilation or target execution is provided. See
[Godot PCK](https://github.com/morluto/rea/blob/main/docs/godot-pck.md).

UnityFS AssetBundle containers (`.bundle`, `.ab`, YooAsset packages and other
byte-identified files) list `CAB-*` SerializedFiles and `.resS` members with
header and node facts; none/LZ4/LZ4HC member blocks stream on demand. Bundles
declaring asset-bundle encryption keep header facts with unknown directory
coverage; LZMA member bytes retain named occurrences with unavailable hashes.
Virtual member offsets are not physical carrier locations. Format 6–8 readers
cache one decoded block and support retained filtered/paged views. No
decryption or SerializedFile object decoding is provided. See
[UnityFS](https://github.com/morluto/rea/blob/main/docs/unityfs.md).

WeChat mini-program packages (`.wxapkg` files with `0xBE`/`0xED` marks) list
members with declared names and absolute carrier offsets; member bytes are
stored verbatim. `V1MMWX` PC-client packages report `encrypted` directory
status with unknown coverage and are refused on extraction; no decryption is
attempted. See
[wxapkg](https://github.com/morluto/rea/blob/main/docs/wxapkg.md).

U-Boot legacy uImages use the same inspection/extraction workflows. Prefer a
retained summary and selected inventory views for header facts or component
IDs. Header addresses/CPU/OS are declarations; payloads remain as stored.
Multi-file components retain their aligned physical ranges; the full payload
is also materialized. Unknown name text retains its original 32 bytes. See
[U-Boot uImage](https://github.com/morluto/rea/blob/main/docs/uimage.md).

## ESP images

For an ESP32-family app or bootloader image, `open_binary` then
`inspect_artifact` with `detail: "summary"`. `root_format_metadata` holds the
observed `header` and `descriptor` (its `kind` is the app or bootloader role)
and the derived `chip`, `checksum`, `appended_sha256` and
`undescribed_trailing` ranges. Segment rows (`segment-NN.bin`) carry
`load_addr`, `classification` (`mapped`, `loaded`, `not-loaded` or `unknown`,
per ESP-IDF `should_map`/`should_load`; RAM ranges are not checked) and
`source_location`; select them with `inspect_analysis_view` and
`logical_paths`. Signature blocks are `unverified`; `public_key_sha256`
matches eFuse key digests.

Unknown `source_coverage` means truncation or undescribed trailing bytes, such
as the flash segments of a `--ram-only-header` image; do not infer absence
from it. Erased flash is described. Integrity mismatches are facts, not
refusals. ESP8266 images are refused, and flash-encrypted images are not
recognized. Segments are not disassembled, and RISC-V or Xtensa segments
cannot be opened as raw-binary. See
[ESP images](https://github.com/morluto/rea/blob/main/docs/esp-image.md).

## ESP flash images

A whole-flash dump opens as `esp-flash-image`. `root_format_metadata` holds
`partition_table` (its `method`: `default`, `caller` or a derived
`scan`), `bootloader` and `apps` ESP image facts, the `otadata` records,
`partitions_outside_dump` and `boot_selection`. Every table has a
`boot_selection`: its `basis` names the bootloader's branch (`no-otadata`
without otadata, `none` when the bootloader rejects the table, `unknown`
when a record is missing), and it lists its `assumptions`;
`running_partition` is always `unknown`. Each image names its entry
`path`. With `detail: "summary"` the root kind is `esp-flash-image-summary`
and each image is a compact `summary` (chip, entry point, segment count,
classified load ranges, signature block count, descriptor identity,
integrity); complete detail keeps every segment. Partition rows
(`partitions/<label>.bin`) carry the declared `type`, `subtype`, names,
`flash_address`, `size` and flags; `encrypted` is a storage flag, not
proof of ciphertext. `written_offset`/`written_end` bound the bytes that
are not erased: an app slot whose facts say `erased` can still hold an older
image, and a limitation says so. For a dump read from another flash address, call
`open_binary` with `format: "esp-flash-image"` and `flash_offset`; an
`esp-image` whose `flash_dump` is set names that offset in a next probe; a
relocated table can be named with `partition_table_offset`. No table found
means unknown, often flash encryption. Extraction writes `nvs_keys` as
ordinary data. See
[ESP flash images](https://github.com/morluto/rea/blob/main/docs/esp-flash-image.md).

## Bun standalone executables

A `bun build --compile` executable stores its module graph in Mach-O
`__BUN,__bun`, a PE or ELF `.bun` section, or an appended trailer.
`open_binary` then `inspect_artifact` reports root format `bun-standalone` with
the carrier, structural layout, module count, entry point, raw flags and
compile exec-argv; each module keeps its `/$bunfs/...` source name, loader,
encoding, sizes and SHA-256, with sourcemap, bytecode and module-info regions as
separate children. Use summary root metadata `entry_point_logical_path` as the
exact inventory `logical_paths` selector, then read the selected content node by
its artifact ID; the reader owns virtual-name normalization and indexed fallback. `extract-artifact` writes stored contents through the safe
output tree; route extracted JavaScript to `analyze_javascript_application`.
Layout labels describe record shapes, not a detected compiler version; unknown
or ambiguous layouts are unsupported. Bytecode is retained, not decoded, and the
executable is never run. See
[Bun standalone executables](https://github.com/morluto/rea/blob/main/docs/bun-standalone.md).

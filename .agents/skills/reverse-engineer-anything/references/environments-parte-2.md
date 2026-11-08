---
type: Guide
title: "REA upstream environments.md 2/2"
description: "Referência oficial REA v61, revisão 25d3c7237ffeffc4f9e6ab87be2f83a643cf5cbb"
---

once when the reason is application behaviour, and report the write as
withheld with its hint:

| Reason                | What to do                                                                                                                  |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `no-state-target`     | Check the path template and recorded views; record missing reads. The route form may not be supported as a mutation target. |
| `no-observation`      | Re-record: read the entity back by id after the write; read it first for an update or delete.                               |
| `unconfirmed`         | Re-record: read the entity and its list after the write.                                                                    |
| `incomplete-entities` | Re-record: read the entity through every view that lists it.                                                                |
| `unexplained-change`  | Re-record once with a second, different call of the same write.                                                             |
| `no-call`             | Re-record once with a write that succeeds.                                                                                  |
| `side-effect`         | Stop: the application changes other entities.                                                                               |
| `contradicted`        | Stop: a later read disagrees with the modelled effect.                                                                      |
| `body-unusable`       | Re-record a UI write once with `request` steps (its body was not exposed); else stop.                                       |

Use new scratch names on each attempt, and delete what an earlier attempt
created before recording again (through the UI, or a `request` step with the
id that capture shows): the environment starts from what the recording first
reads, so a leftover object becomes seeded data that write tasks then find.
New browser scenario captures include request times, so a fully timed capture
can learn a write clock and serve updates that stamp modification times. Older
captures or ones missing a request start keep record order and learn no clock.
When coverage reports that limitation, record again with the current version
before concluding that the timestamp change cannot be served.

## Export and hand off

`export_environment_bundle` with `source: {kind: "retained-evidence",
evidence_id}` (or `{kind: "evidence-file", path}`), an absolute
`output_directory` that does not exist yet and whose parent does, and a `name`
(required when the application runs on `localhost` or an IP address). It returns
the coverage again, `tool_count`, the written files and warnings; `dry_run:
true` reports without writing. The bundle installs `rea-agents` of this REA's
version from the registry; when the user runs an unreleased REA build, pass the
`package_tarball` they give you (a packed `rea-agents` tarball) instead.

Tell the user how many tools answer from the state, which writes are served and
which are withheld and why, and the run command (`agent-env run <folder>`). Say
that the bundle ships the recorded responses and the capture's local paths
(for a browser scenario, the browser executable's path), and
list any `credential-named-members` warning pointers for review. Never publish
or share the bundle yourself.

See
[agent environments](https://github.com/morluto/rea/blob/main/docs/agent-environments.md)
for what the state, replay and the bundle establish, and
[evidence workflows](evidence-workflows.md) for the manifest and state views.

## Tasks for a desktop application environment

Use this when the user has a folder from `rea capture-desktop-application` and
wants a task for the environment `rea export-desktop-application` makes from
it: a prompt for a computer-use agent and checks that grade what the agent
left in the app's files. You write the prompt and the checks; REA validates
the task against the capture and exports it. These commands are CLI only. To
build the environment and its task from a request instead, follow
[desktop builds](desktop-build.md).

### Describe the change

Run `rea describe-desktop-application-task CAPTURE --format json`. Read:

- `owned_paths`: the only paths whose files the app serves, so the only files
  a check can read.
- `files`: each file there whose state differs between the capture's first
  and last version, with its `role`, the `baseline` and `final` versions and
  the net `change`. Text changes list each added and removed line once, and
  `line_counts` gives lines found at both ends whose count changed.
- `task`: a draft whose `checks` are suggestions from the net change of the
  documents the user worked on, or of the app's own state when no document
  changed.

### Write the prompt

Say what the user wants done, in their words and the app's terms: "Add buy
milk to my to-do list and save it". Never say how the file stores it: no
paths the user would not name, line formats, XML, keys or SQL, and never the
checks. Replace the draft's placeholder; the export refuses it.

### Pick the checks

Checks must express only the observed change:

- Keep a suggestion only if it says what the user did. Drop those that follow
  from how the app saves: a format rewrite, renumbered lines, counters. A
  LibreCAD drawing gaining four holes was rewritten from DXF R12 to 2007, and
  the draft suggests counts of `SECTION`, `ENDSEC` and `CIRCLE`; only
  `CIRCLE` equals 9 is the user's change.
- Never check unchanged content, whole files, or values the app stamps:
  times, ids it assigns, window sizes. REA refuses ISO 8601 and `asctime`
  date-times (`Thu Nov  5 09:20:31 2026`); the
  rest is your judgment.
- Prefer a few robust checks: whole lines over positions, rows by meaningful
  columns over generated ids, `at_least` when the user may add more.

Kinds: `file_present` and `file_absent`; `lines_present` and `lines_absent`
(`lines`); `line_count` (`exact` or `contains`, and `equals`, `at_least` or
`at_most`); `value_equals` and `value_absent` (a JSON `pointer` into the view,
such as `/groups/GROUP/KEY` for a key file); `text_equals` and
`text_contains` (an XML element's whole text at a `pointer`, optional
`normalize_whitespace`); `row_present` and `row_absent` (`table`, `where` by
column) and `row_count`. Each names its `file` as a path under the home. A
document saved as a zip archive, such as FreeCAD's `.FCStd`, reads as its
entries: point into an entry's view, and name array members by selector
rather than position, as
`/entries/[name=Document.xml]/view/root/children/[name=Object][@name=Hole]`,
so the check survives an app version that writes them in another order. Give
`value_equals` a `tolerance` to compare a number the app writes as text, such
as `"5.0000000000000000"`, with a number. Check what the user wrote in a
document with `text_equals` on its paragraph element, not `value_equals` on a
text node: apps such as AbiWord and Writer split a paragraph into runs as they
save, so its first text node holds only part of it; use `text_contains` for a
value within a sentence, such as a time. The
[desktop application tasks](https://github.com/morluto/rea/blob/main/docs/desktop-application-tasks.md)
guide has the exact rules.

### Export the task

Write the task as JSON with a `name`, the `prompt` and the `checks`, then run
`rea export-desktop-application CAPTURE DIRECTORY --task TASK.json`. REA keeps
the task only when its checks fail on the seed and all pass on the captured
end state. A refusal names the check: one that fails on the end state does not
match what the user left; "every check already passes on the seed" means no
check captures the change.

When the export refuses an application that no apt repository installs, ask
the user for the `.deb` file they installed it from and capture with
`--package-file`, then ask where the computer may download that file and pass
it as `--package-url`. Never fetch a vendor's download page yourself: such
sites often refuse automated downloads, and the copy and its licence are the
user's to choose.

Tell the user the task's name, prompt and checks, and how to measure a solve
rate: run `agent-env run DIRECTORY --task NAME --agent claude-code` with
`--model MODEL` three times and count the runs reported `passed`. Do not run
agents unless asked. The bundle's seed is the user's real data: keep it local.

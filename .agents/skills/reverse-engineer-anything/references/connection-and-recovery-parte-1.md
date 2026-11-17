---
type: Guide
title: "REA upstream connection-and-recovery.md 1/2"
description: "Referência oficial REA v61, revisão 25d3c7237ffeffc4f9e6ab87be2f83a643cf5cbb"
---

# Connection and task recovery

Read this guide only for an observed connection, command, approval or provider
blocker. Keep the user's target and question in view. Explain what prevented the
operation and the next useful step; after repair, continue the investigation.

Commands below use an installed `rea`. If none is available, use the package
invocation in `SKILL.md` when acquiring REA is authorized. Follow the connected
server's schemas and the installed CLI's help rather than assuming repository
main and an older release expose the same arguments. Tool-name prefixes belong
to the client: find the canonical REA operation in its actual tool listing and
server association before concluding that it is absent. A shared prefix alone
does not establish which server an approval wildcard covers. See
[client tool names and discovery](https://github.com/morluto/rea/blob/main/docs/getting-started/client-configuration.md#client-tool-names-and-discovery).

## Choose the remedy from the observed failure

| Observation                                                     | Meaning and next step                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| --------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| No REA tools in the active session                              | If REA was installed as a client plugin, follow the plugin section below. Otherwise inspect registration with `rea doctor --client codex --json`. A skill installation alone does not register MCP.                                                                                                                                                                                                                                                                                                                                                                                     |
| Missing, malformed or stale registration                        | Prepare a scoped setup plan; apply only authorized changes with backups.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| Aligned registration, but no connected tools                    | Restart/reconnect the client. Doctor checks registration; it cannot establish the current agent's connection. Inspect the client's launch error if reconnection fails.                                                                                                                                                                                                                                                                                                                                                                                                                  |
| MCP connection dropped or REA could not start                   | If the connection drops, retry that call once only when it is safe to repeat or known not to have started. If it may already have changed the target, or the same failure returns, tell the user REA's connection failed and ask them to restart the MCP client. If REA could not start its connection, retry once, then tell the user and ask them to restart the MCP client. Do not retry a startup failure that returns the same unchanged result. Run `rea doctor` when REA could not start or tools stay missing after restart. Doctor cannot restore a live connection by itself. |
| `MCP tool call requires approval, but approval policy is never` | Codex rejected the call before REA executed it. Do not install an engine or repeat setup to solve this client-policy decision. See the approval section below.                                                                                                                                                                                                                                                                                                                                                                                                                          |
| REA returns `provider_unavailable` or an unavailable tool entry | Read that operation's provider diagnostics or `binary_session.result.tool_availability`. Expand each group's tool names and inherit the shared client requirements unless overridden. Use the operation's availability, reason, and remediation; a complete tool catalog does not imply every operation is available. Request `{"detail": "complete"}` only for provider diagnostics.                                                                                                                                                                                                   |
| `rea` is missing in a child shell, or Node is incompatible      | Inspect the actual shell PATH and launcher/runtime. A prefix installation may be absent from a login shell's PATH; this does not establish a broken MCP registration.                                                                                                                                                                                                                                                                                                                                                                                                                   |

If the client cannot launch its shell/sandbox helper or read the installed skill,
inspect its launch diagnostic. For example, Codex can refuse helper aliases when
CODEX_HOME is under `/tmp`; use an authorized profile outside the OS temporary
directory. REA setup cannot repair the client's executor. Preserve the sandbox
instead of silently disabling it to make an analysis command work.

Static JavaScript and ordinary ZIP inventory need neither Hopper nor Ghidra.
HAR inspection is offline and does not require a live browser. Native
decompilation, Android analysis and firmware extraction have their own
bring-your-own prerequisites. Use supported inventory/static evidence when it
answers part of the question, and name the unavailable evidence separately.
Do not substitute another target. Label observations from ordinary shell tools
as such; only actual REA output supplies REA Evidence.

## Distinguish client and server deadlines

A host message such as `timed out awaiting tools/call` with no REA result means
the client's deadline expired before a result arrived. It does not establish
that REA finished, canceled successfully, or released all resources. Do not
repeat the unchanged analysis. Inspect available session/provider state and
retained Evidence only when useful; an idle `binary_session` is not proof that
every target-free operation stopped. Never invent a missing Evidence reference.

Allow the host tool deadline to cover execution, cancellation/cleanup, and
response delivery. In Codex, the setting is
`mcp_servers.<server-id>.tool_timeout_sec` in **seconds**, not
`startup_timeout_sec`. Select a value for the actual operation budget and
headroom, not a universal timeout. See the
[official MCP configuration reference](https://learn.chatgpt.com/docs/extend/mcp?surface=cli).
Apply configuration changes only within the authorized scope, preserving
unrelated entries and backups; reconnect/restart as required by the host.
When configuration is not editable, report the setting needed or use an
authorized CLI fallback. Do not silently broaden permissions or switch targets.

A returned REA `provider_timeout` instead means the **server/provider** reached
its deadline. Follow its `remediation.action`, reuse accessible partial Evidence,
and resolve reported incomplete cleanup before retrying. Change a request
field or a `REA_*` setting only when the returned remediation explicitly
recommends that change. Restart REA when that text says the value is read at
startup. A mention that a control does not raise the deadline is not a
recommendation to change it. Do not invent a setting the remediation does not
name. Raising only the client deadline does not change
this server budget. For static JavaScript,
`analysis_timeout_ms` (CLI `rea analyze-javascript-application <path>
--analysis-timeout-ms <milliseconds>`) controls each source's
startup, analysis and transfer deadline in **milliseconds**. Increase that
budget or reduce semantic work / select exact inventoried `source_paths` when
the task permits. Omit `source_paths` for all sources; `[]` is invalid.
`detail: "summary"` reduces delivery, not analysis work. A per-source budget
does not bound the elapsed time for an entire multi-source application.
Partial-Evidence inspection can avoid recomputation; an unavailable reader is
not itself a reason to block a corrected static-analysis retry.
The shorthand `rea analyze <path>` does not expose this timeout flag; use the
explicit JavaScript command when changing that budget.

For invalid arguments, use the connected tool's current input schema, or the
installed CLI's command help. Refresh the catalog if stale; do not guess fields
from an older skill or return the entire catalog to repair one field. Correct
all reported fields before retrying.

## Recovery ladder

Follow the returned remediation. Change a request field or a `REA_*` setting
only when that text explicitly recommends the change. Restart REA when it says
the value is read at startup. A mention that a control does not raise a limit
is not a recommendation to change it. Do not invent a setting. Host memory and
`NODE_OPTIONS` do not raise a REA admission unless that remediation says so.

- MCP connection dropped: retry that call once only when it is safe to repeat
  or known not to have started. If it may already have changed the target, or
  it fails again, tell the user REA's connection failed and ask them to restart
  the MCP client. Do not run `rea doctor` for the drop itself.
- REA could not start its connection: retry once, then tell the user and ask
  them to restart the MCP client. Run `rea doctor` when REA could not start or
  tools stay missing after that restart. Do not retry a startup failure that
  returns the same unchanged result.
- `cleanup_incomplete`, or retained cleanup owners in the error details: retry
  that exact cleanup owner. Do not start new work that needs those resources
  until that owner confirms release. Repeating the original call is not that
  retry.
- Capacity, when the result is `resource_constraint` and `retryable` is true:
  wait for the active admission to finish, then retry within the reported
  limits. Do not restart REA to free the slot.

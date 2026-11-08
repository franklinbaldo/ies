---
type: Guide
title: "REA upstream connection-and-recovery.md 2/2"
description: "Referência oficial REA v61, revisão 25d3c7237ffeffc4f9e6ab87be2f83a643cf5cbb"
---

- `configuration_invalid`: correct the named setting in the environment that
  starts REA (the shell or the MCP registration), then restart REA. Run
  `rea doctor` to see the `runtime-configuration:` check. Re-run `rea setup`
  for that client when the registration must carry the new setting, then
  reconnect.
- `invalid_request`: correct the reported fields from the current schema. Do
  not repeat the unchanged call.

## REA installed as a client plugin

A Claude Code or Codex plugin owns its `rea` server and this skill. Doctor's
`--client` and `--skill` checks and `rea setup` read and write only
setup-owned state, so for a plugin-only install they report `missing`, and
applying setup would add a second `rea` server. If this skill was loaded from
a plugin, or the client lists a `rea` plugin, do not prepare a setup plan for
missing tools. Instead:

- Check that the plugin is installed and enabled with the client's own
  listing: `claude plugin list` and `/mcp` in Claude Code, `codex plugin list`
  in Codex.
- Restart or reconnect the client, then read its launch error for the `rea`
  server if tools are still absent. The plugin starts its bundled runtime; a missing build or version mismatch
  requires a reinstall, or a fresh build when developing from source.
- Check analysis engines through the connected server's
  `binary_session.result.tool_availability`. `rea doctor --provider <id> --json`
  from a shell reports that shell's environment, which can differ from the
  environment the client gave the plugin server. If you still run it through
  the package runner, use the version pinned in the plugin manifest's server
  entry (`rea-agents@<version>`), not `@latest`, so it matches the server the
  plugin starts.

Use the setup repair below only when the user wants a setup-owned registration
instead of the plugin, and remove or disable the plugin first.

## Repair registration when needed

For Codex:

```bash
rea doctor --client codex --json
rea setup --client codex --dry-run --json
```

Use the current supported client's ID: `claude_code`, `claude_desktop`, `codex`,
`cursor`, `gemini_cli`, `windsurf`, `devin`, `opencode`, `antigravity`,
`copilot_cli`, `commandcode`, `qwen_code`, `vscode`, `trae`, `grok_build`, `omp`, `pi`,
`hermes`, `qoder`, `droid`, `zcode`, `dsh`, `freebuff`, or `grok_bot`. If unknown, inspect `doctor --json` before choosing a
scope. `grok_bot` uses a chat registration step rather than local `mcp.json`.
For unsupported clients, use manual stdio registration with the selected
installation’s absolute runtime and entry script, or continue through the CLI.
Acquire packages from a trusted directory outside the target project. Keep a
package-runner installation and its runtime available while registrations use it.

Show the plan's exact paths, backups, changes and bundled-skill replacement.
Obtain approval for writes when that concrete scope is not already authorized,
then apply the same scope with `rea setup --client codex --yes`. Add
`--install-hopper` only when separately needed and explicitly approved. Do not
install or upgrade unrelated prerequisites. Restart/reconnect, verify tools
actually appear, and resume the original task. If they remain absent, inspect
the launch failure rather than looping through setup.

## Codex tool approval is separate from REA readiness

REA tools have no permission-grant handshake. Codex can still require approval
because an operation creates owned resources, launches a helper or has cleanup
effects. An advertised tool or successful doctor result does not override that
client policy. `workspace-write` alone does not resolve an MCP approval
rejection, and `approval_policy = "never"` does not mean every MCP tool is
preapproved.

When interactive approval is available, use the client's approval flow for the
requested operation. In an unattended run, continue through an authorized CLI
workflow if the execution environment permits it, or report the blocked
operation and the approval needed to proceed. Do not retry the same rejected
call, silently broaden approvals, or rewrite MCP effects as read-only.

For an operator who explicitly wants to preapprove named operations in a
disposable Codex profile, current Codex supports per-tool configuration. For
example, opening and closing an analysis session can be scoped independently:

```toml
[mcp_servers.rea.tools.open_binary]
approval_mode = "approve"

[mcp_servers.rea.tools.close_binary]
approval_mode = "approve"
```

Use the actual server ID and only the operations whose effects were approved;
this example does not approve inspection, extraction or runtime execution.
Show the proposed config edit and preserve unrelated entries and backups before
writing. Managed policy can constrain client settings. Consult the
[Codex configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference)
for the installed client's supported settings. This is an optional recovery
choice, not a prerequisite for every REA task.

## Repair an editable REA limit

If `details.limit` names an editable setting and its bounds, call
`update_analysis_settings` directly with a `changes` entry containing `setting`,
`action: "set"` and `value`. `inspect_analysis_settings` is optional when those
facts are already present. Retry the same input after work and cleanup settle.
Runtime defaults affect new work only; explicit per-call arguments take
precedence and must be corrected directly when selected. Reset restores launch
values. Updates survive target close/open but disappear on REA restart. They do
not edit host deadlines, response buffers, registrations or fixed limits.

## Continue through an installed CLI

Check the command environment only if the launcher fails:

```bash
command -v rea
command -v node
node --version
rea --version
```

Use the public launcher at its known installation path with a supported existing
Node on PATH when the child shell loses a prefix installation. Avoid internal
`dist/cli.js` entrypoints or global toolchain changes. MCP may have its own valid
launcher even when a shell cannot resolve `rea`.

For a user-supplied JavaScript tree or ASAR, select output files in the authorized
workspace and save complete Evidence before reading a view:

```bash
rea analyze-javascript-application /absolute/path/to/app --json > app-evidence.json
jq -c '{source: {kind: "inline", evidence: .}, view: {kind: "summary"}}' app-evidence.json > app-view.json
rea inspect-analysis-view app-view.json
```

These commands do not register MCP or install an analysis engine. CLI JavaScript
JSON is complete and can be very large; redirect it rather than loading it into
context. Use an existing JSON processor if `jq` is absent, without installing
one just for this recipe. Inspect a module page or item using the same inline
source and the [JavaScript recipes](javascript-applications.md). Preserve graph
coverage, limitations and unknowns. Native CLI analysis still requires its
selected engine, and all CLI operations retain the host's execution permissions.

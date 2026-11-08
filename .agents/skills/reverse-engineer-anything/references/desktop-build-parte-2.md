---
type: Guide
title: "REA upstream desktop-build.md 2/2"
description: "Referência oficial REA v61, revisão 25d3c7237ffeffc4f9e6ab87be2f83a643cf5cbb"
---

unless at least one check fails on the seed and every check passes at the
end, and says why, such as a pointer that names nothing at the end. A
check that the rest is untouched passes on the seed too, which is fine. On
a refusal, fix the task and export again, into another folder if that one
is no longer empty. Keep
`seed_files`, `scrubbed_files`, `unscrubbed_files` and `limitations` for the
report.

Keep the build computer open until the witness holds: a task fix exports
again from it.

## Witness

Witness in a fresh computer of the bundle, from the prompt, as a solving
agent would. Do not replay the build's actions or coordinates, and do not
check between stages to steer the edit: a solving agent never sees the
checks. Give every `check_build_task` the task exactly as exported.

1. `start_build_computer` with
   `{"source": {"kind": "bundle", "path": "<bundle>"}}`. The app starts
   with the seed's files in place but need not open the document: open it
   from the prompt's path, as a solving agent would. Take a `wait`
   screenshot first, since the desktop may not have drawn yet.
2. `check_build_task` with `stage: "seed"` and `directory`, the absolute
   path of a new or empty folder for the record. A check must fail.
   Recording starts. Later stages take no `directory`.
3. Do the task from the prompt with `act_on_build_computer` and save.
   `check_build_task` with `stage: "after_edit"`: every check must pass.
4. Make a near miss on top of it: a plausible wrong edit, such as a typo or
   a wrong number, and save. `check_build_task` with `stage: "near_miss"`:
   a check must fail.
5. `act_on_build_computer` with `action: "reset"`, then `check_build_task`
   with `stage: "after_reset"`: a check must fail again.
6. The last stage ends the witness: `witness.record` has the verdict, the
   path of `witness.json` and the recording's segments in `directory`.
   `close_build_computer` on the witness computer.

A stage that does not hold ends the witness as `failed`. Read each check's
result and decide:

- `after_edit` failed though the saved file is right: the check is brittle.
  Fix it, export again and witness in a new computer.
- `after_edit` failed because your edit was wrong: witness again in a new
  computer; a computer is witnessed once.
- `near_miss` passed: the checks are too loose. Tighten them, export again
  and witness again.
- `after_reset` passed: the reset did not restore the seed. Report it as a
  limitation.

Stop after three witnesses and report what still fails.

## Solver runs

Only if the requester asked for them. If they wanted to review the task
first, show them its prompt and checks and wait for their approval. Then run
the bundle within the budget, one run at a time, with the runner
`REA_AGENT_ENV_COMMAND` names:
`agent-env run <bundle> --task <name> --agent claude-code --model <model>`.
Count runs reported `passed`. A run `scored below 1` is a miss; one reported
`failed` did not finish, so run it again if the budget allows instead of
counting it. Never run past the budget.

## Close and report

`close_build_computer` on every computer still open, also after a failure.
If a close fails, call it again; if it still fails, tell the requester what
is left.

Report:

- the bundle path and the environment name;
- the task's name and prompt;
- each check, and why it proves the edit and fails the near miss;
- each witness stage with its result, the verdict, and the paths of
  `witness.json` and the recording;
- the seed's contents: `seed_files` and what each holds;
- `scrubbed_files` and `unscrubbed_files`; an unscrubbed file still names
  the host or user, so it needs review before sharing;
- limitations: the capture's, the export's and anything you could not do;
- solver runs, if any: the model, the runs and the passes;
- the build time and the questions you asked.

Never publish or share the bundle yourself.

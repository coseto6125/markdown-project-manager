---
name: mpm
description: Read/update the cross-session follow-ups log (deferred work) via the mpm CLI instead of reading the whole FOLLOWUPS.md. Use at four moments — before starting a task (check for related/blocking work), when you start fixing an entry (mark it in progress by its branch, then its PR), during (file what you defer), and after the PR merges (mark done). Reach for mpm whenever a task involves opening a PR, deferring work, or asking "did some PR already close a follow-up".
---

# mpm — follow-ups, without reading the whole file

`mpm` is structured CRUD + graph queries over the two-file follow-ups log
(`FOLLOWUPS.md` = open work, `FOLLOWUPS_DONE.md` = archive). It returns exactly
the slice you need so you never read the 25 KB+ file. Markdown stays the source
of truth — every change re-renders both files.

## The one rule: query the log at four moments

- **Before** a task → `mpm list --status open`, then `mpm list --status in-progress` (and `mpm query pr:<N>` if a PR is involved). Is this already filed? Blocked? In progress? Already done?
- **Start** → before your first edit on a fix for an entry, run `mpm start <id> --branch <branch>`. When its PR opens, run `mpm start <id> --pr <N>`. When the PR fixes only part of the entry, first `mpm add` the rest as a new entry and `mpm link` it to the original.
- **During** → before you `mpm add` an issue you found in the task, fix it or ask the user. File it only after the user says to leave it, or when the user froze the scope. File a blocker: `mpm block <id> --on <other>`.
- **After** the PR merges → `mpm done <id>`. It resolves with the PR or branch that `start` recorded. Work dropped: `mpm stop <id>`. Won't do it: `mpm wontfix <id> --reason ...`.

If you open a PR without checking the log first, you have skipped the one rule.

## Which log mpm uses (you rarely need --dir)

`mpm` finds the log automatically by walking up from the current directory to the
nearest `.claude/FOLLOWUPS.md` — like git finding `.git`. So **inside any repo,
just run `mpm <cmd>`** and it targets that repo's log. Pass `--dir <path-to-.claude>`
only to point at another repo's log:

```
mpm list --status open                          # this repo's log (auto-found)
mpm --dir /other/repo/.claude list --status open  # explicitly another repo's log
```

## Read (never mutates)

```
mpm --help            # every subcommand, always current
mpm <sub> --help      # that subcommand's flags

Reach for these first, and let `--help` supply the flags:
  read      list · show · stub · query · graph
  write     add · start · stop · done · wontfix · supersede · block · unblock · reopen · set · move · link
  check     validate
```

`query` filter keys (combine with spaces, all must match): `status`, `category`,
`size`, `pr`, `blocked-on:<id>`, `links-to:<id>`, `owner`. `status` values:
`open`, `in-progress`, `done`, `wontfix`, `blocked`, `superseded`. `size` values: `S`, `M`, `L`.
`pr:<N>` also matches an entry that `start --pr <N>` marked in progress.

**graph direction — ignore the words "upstream/downstream" in the request, decide by meaning:** "what depends on / is blocked on / waits for X" → `--direction up`. "what X itself needs / depends on" → `--direction down`.

## Write: four constraints `--help` does not tell you

Every write re-renders both markdown files and prints the affected id.

1. **`done` needs one PR or branch, never both.** With no flag it takes the one `start` recorded. Without a `start`, pass `--pr <N>` (the normal close) or `--branch <name> --commit <sha>` (work that never became a PR). `start` takes exactly one of `--pr` or `--branch`, and refuses a resolved entry.
2. **`add` requires `--category` and `--scope`.** Everything else is optional. `--scope` is one sentence: what's deferred *and why it matters*.
3. **Targets of `supersede --by`, `block --on`, `link` must be existing ids.** Run `mpm list`/`mpm show` first if unsure the id exists.
4. **`ambiguous id <id>: N entries share it`** means the log has a duplicate id (a real data bug). Run `mpm validate` to see all duplicates, then resolve by editing the markdown — mpm can't disambiguate for you.

## Mint a new id only when adding

`mpm add` mints the id for you and prints it — do not pre-compute ids. Only when
you must reference the id *before* the entry exists: reserve it with `mpm next-id`,
then create the entry with that exact id via `mpm add --id <id> ...`. (Plain
`mpm add` without `--id` mints its own, so a bare `next-id` you don't pass back is
never stored.)

## When NOT to use mpm

Editing the markdown body prose by hand, or anything that isn't an entry / edge /
field — open the file directly. `mpm` owns structure; freehand text is yours.

---
name: mpm
description: Read/update the cross-session follow-ups log (deferred work) via the mpm CLI instead of reading the whole FOLLOWUPS.md. Use at three moments — before starting a task (check for related/blocking work), during (file what you defer), and after (mark done). Reach for mpm whenever a task involves opening a PR, deferring work, or asking "did some PR already close a follow-up".
---

# mpm — follow-ups, without reading the whole file

`mpm` is structured CRUD + graph queries over the two-file follow-ups log
(`FOLLOWUPS.md` = open work, `FOLLOWUPS_DONE.md` = archive). It returns exactly
the slice you need so you never read the 25 KB+ file. Markdown stays the source
of truth — every change re-renders both files.

## The one rule: query the log at three moments

- **Before** a task → `mpm list --status open` (and `mpm query pr:<N>` if a PR is involved). Is this already filed? Blocked? Already done?
- **During** → file what you defer: `mpm add ...`. File a blocker: `mpm block <id> --on <other>`.
- **After** → `mpm done <id> --pr <N>` (or `--branch`). Won't do it: `mpm wontfix <id> --reason ...`.

If you open a PR without checking the log first, you have skipped the one rule.

## Which log mpm uses (you rarely need --dir)

`mpm` finds the log automatically by walking up from the current directory to the
nearest `.claude/FOLLOWUPS.md` — like git finding `.git`. So **inside any repo,
just run `mpm <cmd>`** and it targets that repo's log. Pass `--dir <path-to-.claude>`
only to point at a DIFFERENT repo's log than the one you're standing in:

```
mpm list --status open                          # this repo's log (auto-found)
mpm --dir /other/repo/.claude list --status open  # explicitly another repo's log
```

## Read (never mutates)

```
mpm list --status open                 # triage open work (default if no filter)
mpm list --status open --json          # machine-readable (compact JSON)
mpm show <id>                          # one full entry
mpm stub <id>                          # just the one-line resolution stub
mpm query pr:498                       # did PR #498 surface or close a follow-up?
mpm query status:open size:L           # flat key:value AND filter (space-separated)
mpm graph <id> --direction up          # what DEPENDS ON <id> (who is blocked-on / links-to it)
mpm graph <id> --direction down        # what <id> ITSELF depends on (its own blocked-on / links-to / supersedes)
mpm validate                           # dangling links, duplicate ids
mpm next-id                            # the next free id for today (no entry created)
```

`query` filter keys (combine with spaces, all must match): `status`, `category`,
`size`, `pr`, `blocked-on:<id>`, `links-to:<id>`, `owner`. `status` values:
`open`, `done`, `wontfix`, `blocked`, `superseded`. `size` values: `S`, `M`, `L`.

**graph direction — ignore the words "upstream/downstream" in the request, decide by meaning:** "what depends on / is blocked on / waits for X" → `--direction up`. "what X itself needs / depends on" → `--direction down`.

## Write (re-renders both markdown files; prints the affected id)

```
mpm add --category "<C>" --scope "<what & why it matters>" [--why <out-of-scope|blocked-by|size-too-large|other>] [--size S|M|L] [--surfaced "PR #<N>"]
mpm done <id> --pr <N> [--commit <sha>] [--note "..."]    # resolved by a PR
mpm done <id> --branch <branch> --commit <sha>            # resolved on a branch, not a PR
mpm wontfix <id> --reason "..."
mpm supersede <id> --by <other-id>                        # <other-id> must exist
mpm block <id> --on <other-id>      ·     mpm unblock <id> [--on <other-id>]
mpm reopen <id>                                           # pull a resolved entry back to open
mpm set <id> --field <name> --value "..." [--append]      # edit one field; --append for links/lists
mpm move <id> --category "<C>"
mpm link <from-id> <to-id>                                # records a links-to edge
```

## Three hard constraints (these are where calls go wrong)

1. **`done` needs exactly one of `--pr` or `--branch`** — never zero, never both. PR resolution → `--pr <N>`. Branch-only → `--branch <name> --commit <sha>`.
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

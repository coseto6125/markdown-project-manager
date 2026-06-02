# mpm — Markdown Project Manager

Structured CRUD + graph queries over a markdown deferred-work log, built for
LLM coding agents. The markdown stays the human-readable source of truth; `mpm`
gives an agent **exactly the slice it needs without reading the whole file** —
one entry, one stub, one relationship hop, one PR lookup.

It targets the two-file "follow-ups" protocol (`FOLLOWUPS.md` for open work,
`FOLLOWUPS_DONE.md` for the resolved archive), but the model is general: ordered
entries with a small set of core fields, free-form extra fields, and typed
relationships (`blocked-on`, `links-to`, `superseded-by`, `done-in-PR`,
`carried-over-from`).

## Why

The protocol says *"read the log before opening any PR."* For an agent that is a
27 KB+ file read on every task — pure context-window tax. `mpm` replaces the grep
with structured queries that return signal-dense [toon](https://crates.io/crates/etoon)
(or `--json`):

```
mpm list --status open               # triage, no full-file read
mpm show FU-2026-05-23-042            # one entry
mpm query pr:498                      # did this PR already close a follow-up?
mpm graph FU-2026-05-23-048 --direction down   # what's blocked on this?
mpm stub FU-2026-05-23-042           # the one-line resolution stub
```

Every mutation writes a structured store and **re-renders both markdown files**,
so the human-facing log stays in lockstep and git-diffable. A disposable msgpack
cache (guarded by file mtime + schema version) makes reads sub-millisecond;
manual markdown edits are absorbed by `mpm import`.

## Install

```
cargo install --git https://github.com/coseto6125/markdown-project-manager --bin mpm --locked
```

Or grab a prebuilt binary from [Releases](https://github.com/coseto6125/markdown-project-manager/releases).

## Usage

`mpm` resolves the log from `--dir <.claude dir>` (defaults to the canonical
code-graph-nexus path). All read commands accept `--json`; mutations print the
affected id on stdout so an agent can capture it.

### Create / read

```
mpm add --category "Parser & Schema" --scope "..." --why "..." --size S --surfaced "PR #520"
mpm show <id> [--json]
mpm stub <id>
mpm list [--status open|done|wontfix] [--category C] [--size S|M|L] [--blocked] [--pr N] [--json]
mpm next-id
```

### Mutate

```
mpm set <id> --field scope --value "..." [--append]
mpm move <id> --category "CLI / Commands"
mpm done <id> --pr 520 [--commit abc1234] [--note "..."]
mpm done <id> --branch feat/x --commit abc1234
mpm wontfix <id> --reason "..."
mpm supersede <id> --by <id>
mpm block <id> --on <id>     ·     mpm unblock <id> [--on <id>]
mpm reopen <id>
mpm link <from> <to>
```

### Graph / query / maintenance

```
mpm graph <id> [--direction up|down|both] [--depth N] [--json]
mpm query status:open size:L          # flat key:value AND filters
mpm query pr:385                       # filters: status, category, size, pr, blocked-on, links-to, owner
mpm render [--check]                   # re-render markdown; --check exits 1 on drift
mpm import [--dry-run]                 # parse markdown into the store
mpm validate                           # dangling links, duplicate ids
```

## Data model

| Concept | Notes |
|---|---|
| Entry | `id` + `category` + `status` + 6 core fields (owner/scope/why-deferred/next-action/size/links) + ordered extra fields |
| Field order | preserved verbatim per entry (`original-scope` round-trips as `original-scope`, one-off keys keep their spot) |
| Status | `open` · `done` · `wontfix` · `blocked` (stays open) · `superseded` |
| Edges | `blocked-on`, `links-to` (from `[[FU-id]]`), `superseded-by`, `carried-over-from`, `done-in-PR`, `surfaced-in-PR` |
| Render | canonical form; the first `import` + render is a one-time normalization, then byte-stable |

## License

MIT

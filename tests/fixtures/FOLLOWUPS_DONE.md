# Follow-ups · Done archive

<!-- Resolved (`✅ done in PR #N`) and wontfix (`🚫 wontfix: <reason>`)
     entries live here so the active `.claude/FOLLOWUPS.md` stays
     token-light. Append-only; never delete. -->

### FU-2026-05-27-001  ·  surfaced + resolved in PR #473 (load_ensured CI race)  ·  ✅ done in PR #473
- **owner**: session (fix-ci-473)
- **original-scope**: `write_builder_fingerprint_sidecar` detached its ~20-byte write to `std::thread::spawn`, mirroring the head-SHA / compatible-version sidecars. But those two are pure read-side perf hints (miss → mtime walk / `header_compatible`), whereas a stale/missing fingerprint sidecar makes `fingerprint_drifted` report drift and forces a full `build_l2` — the most expensive fallback. A CLI process that drift-rebuilt via the attach fast-path then exited could terminate before the detached flush (Rust does not join detached threads at exit), so the next invocation rebuilt an already-current graph. Surfaced because the PR #473 test `load_ensured_rebuilds_on_fingerprint_drift` read the sidecar synchronously and raced the flush — passed on the faster macOS runner, failed on ubuntu/windows.
- **resolution**: made `write_builder_fingerprint_sidecar` synchronous (dropped the `thread::spawn`). The write is sub-ms; thread creation cost more than it deferred. This corrects the false symmetry with the other two sidecars (whose miss is cheap) and closes the detach-survives-exit gap — on return the sidecar reflects the running binary, so drift is cleared the moment `build_l2` / the attach fast-path returns. Test reverted from the bounded poll (its first-pass fix) back to a direct synchronous assertion, which now also guards against anyone re-detaching the write.
- **links**: PR #473; `crates/ecp-cli/src/auto_ensure.rs` (`write_builder_fingerprint_sidecar`); `crates/ecp-cli/src/build/orchestrator.rs` (`attach_if_fingerprint_matches`, `build_inside_locked` callers); test `crates/ecp-cli/tests/load_ensured_version_check.rs::load_ensured_rebuilds_on_fingerprint_drift`

### FU-2026-05-22-001  ·  surfaced in PR #345  ·  ✅ done in PR #370 (merged as dbb71278, 2026-05-23)
- **owner**: session 780ca2a6 (B-takeover) → continued by session fdae19af (post-/clear). 17 commits shipped on the umbrella branch
- **original-scope**: 12 langs missing Type-1 BlindSpot emitter; only `python/parser.rs:719` pushed BlindSpot pre-FU-001. P0 had landed in PR #351 (verdict layer consuming CallMeta flags from indirect_dispatch.rs, raising coverage 1/14 → 7/14 without parser work). P1–P7 was the per-lang BlindSpot emitter rollout.
- **resolution**: full P1–P7 + spec finalisation shipped on `feat/blindspot-rollout`:
  - **P1 (TS+JS)** `6a2d58c0` — eval / Function-ctor / dynamic-import / dynamic-require with literal-vs-variable check
  - **P4 (Rust)** `7c42652a` — transmute-to-fn / libloading::Library::get
  - **P3 (Go)** `ee683727` — reflect.MethodByName / plugin.Open
  - **P2a (Java)** `6d96631c` — Class.forName / Method.invoke (chain anchor)
  - **P2b (Kotlin)** `3cce6bc9` — Java reflection bridge
  - **P2c (C#)** `5e7c8622` — Activator.CreateInstance / MethodInfo.Invoke
  - **P5a (PHP)** `c8d3927b` — eval / call_user_func / variable-function-call
  - **P5b (Ruby)** `535b69e8` + clippy fix `3291d099` — eval / send(var) / instance_eval
  - **P6a (Swift)** `79a249e9` — NSClassFromString / perform(Selector)
  - **P6b (Dart)** `25efa472` — Function.apply / dart:mirrors import
  - **P7a (C)** `e8443599` — dlsym load-site anchor
  - **P7b (C++)** `8f95ecbb` — same anchor
  - **schema-cmd** `be806258` + extension `3c58dc08` — `ecp schema blindspots` + `reltypes` / `node-kinds` / `graph-version` (Constraint 5)
  - **is_test-field** `e8d3ac0b` — `BlindSpotRecord.is_test` schema bump + diff-region filter (Constraint 4)
  - **dispatcher-skel** `de6bbf84` — extracted `push_blind_spot` helper; 31 dispatch sites consolidated (Constraint 6)
  - **docs(spec)** `eebcf8fb` — Constraint 5 marked SHIPPED, MCP follow-up logged
  - **merge origin/main** — `EnumVariant` (NodeKind) + `Decorates` (RelType) added to schema-cmd inventories; 2 textual conflicts (c_sharp / php parser imports) resolved
- **size-actual**: 17 atomic commits on the umbrella branch + 1 merge commit; ~1100 LOC src + ~70 new tests across analyzer & CLI. Full suite: analyzer 2077 / CLI 1084 / schema 10 / clippy clean.
- **deferred-into-followup**: MCP exposure (`ecp_schema` tool needs hand-rolled subcmd discriminator like ecp_group) — tracked as FU-2026-05-23-020
- **links**: PR #345 commit `5e7cc4dd`; PR #351 (P0 + design spec); `docs/specs/2026-05-23-blindspot-cross-lang-design.md`; memory `project_fu_001_b_takeover.md`

### FU-2026-05-22-002  ·  surfaced in PR #345  ·  ✅ done in PR #350
- **owner**: dx-3in1 session
- **scope**: `dev::uid_audit::parse_hint` 用 `rsplit_once(':')` 切 name；若任一 parser 開始 emit 含 `:` 的 name（例如 Swift selector `init(foo:bar:)`），rsplit 會誤把 name 結尾的 `:` 當邊界
- **resolution**: 把 hint 欄位分隔符從 `:` 改成 ASCII Unit Separator (`\u{1f}`)。`/simplify` review 後再進一步抽出 `HINT_FIELD_SEP` const + `HintFields` struct + `format_hint` helper 到 `ecp-core`，emit 端與 parse 端共用。三條新 regression test（Swift selector / Rust `::` / Windows path with `:`）+ 5 條既有 test fixture 同步切換到新 delimiter
- **size**: S（實際 ~150 LOC，含 fixture 切換）
- **links**: PR #350 commits `fix(parse_hint): use ASCII US (\x1F) delimiter` + `refactor(hint): extract HINT_FIELD_SEP/HintFields/format_hint to ecp-core`

[59 more lines]
### FU-2026-05-22-003  ·  surfaced in PR #345  ·  ✅ done in PR #372 (merged as fdfdd6ea, 2026-05-23)
- **owner**: PR #372 session
- **original-scope**: `ecp coverage` 與 `ecp group coverage` 別名（為一 release 向後相容）— 一 release 後拔掉
- **resolution**: PR #372 commit `ad2913c chore(cli)` 拔掉 `#[command(alias = "coverage")]` × 2 處（top-level Summary variant + group/mod.rs::Summary variant）+ 移除 `coverage_alias_still_routes_to_summary` 和 `group_coverage_alias_help_exits_zero` 兩個 back-compat test
- **size**: S（~25 LOC delete）
- **links**: PR #372 commit `ad2913c`；PR #345（alias 引入處）

### FU-2026-05-22-004  ·  surfaced in PR #345  ·  ✅ done in PR #372 (merged as fdfdd6ea, 2026-05-23)
- **owner**: PR #372 session
- **original-scope**: `ecp admin verify-resolver` 別名（為一 release 向後相容）— 一 release 後拔掉
- **resolution**: PR #372 commit `ad2913c chore(cli)` 從 `commands/admin/mod.rs::AdminCommands` 移除 `VerifyResolver` variant + dispatch arm；保留 `ecp dev verify-resolver` 為唯一路徑。`cli_surface_invariants::ADMIN_SUBCMDS` + `cli_help_surface_test` 同步更新；新增 `admin_verify_resolver_alias_removed` regression lock
- **size**: S（~25 LOC delete）
- **links**: PR #372 commit `ad2913c`；PR #345 commit `feat(cli-dev): hidden ecp dev namespace`

### FU-2026-05-22-005  ·  surfaced in PR #345  ·  ✅ done in PR #345
- **owner**: PR #345 session (body lost in 2026-05-23 housekeeping mishap; reconstructed from jsonl history)
- **original-scope**: `dev::uid_audit::build_report` 用 `sort_by_key + take(top)` 是 O(N log N) — 對 N=450k 級別大圖譜會慢；改用 `BinaryHeap<Reverse<_>>` 維持 size K 的 min-heap，達 O(N log K)
- **resolution**: PR #345 (per stub) — exact commit hash not recovered. Pattern matches eywa hint "[tooling][algorithm] use heapq for top-K problems". `crates/ecp-cli/src/commands/dev/uid_audit.rs::build_report` is the impl site
- **size**: S（~20 LOC）
- **links**: PR #345; `crates/ecp-cli/src/commands/dev/uid_audit.rs::build_report`

### FU-2026-05-22-006  ·  surfaced in PR #345  ·  ✅ done in PR #334
- **owner**: cold-ingest session (PR #334)
- **scope**: `.sample_repo/C` 索引失敗，最初診斷為 submodule 缺檔；dx-3in1 session 重新調查後判定為 `ecp admin index` orchestrator 在 rename `_src` → `_src.dead.*` 後仍 walk dead tree 的 race condition（也涉及 tantivy background writer 的 segment churn）
- **resolution**: PR #334 CI-M-followup commit 已修 — `crates/ecp-cli/src/build/orchestrator.rs::dir_size` 改成 tolerant metadata fetch (`if let Ok(m) = e.metadata()`)，遇 ENOENT 跳過該 entry。dx-3in1 session 在 origin/main tip 驗證 fix 已合進 main
- **size**: S（已被 #334 解）
- **links**: PR #334 commit `fix(build): dir_size tolerant of tantivy background race`；`crates/ecp-cli/src/build/orchestrator.rs::dir_size`

### FU-2026-05-23-022  ·  surfaced in PR #367 (feat/pathliteral-full)  ·  ✅ done in 2026-05-23 worktree cleanup batch
- **owner**: cross-session cleanup (was: 🚫 wontfix per CLAUDE.md "NEVER delete `.claude/worktrees/`" rule; user explicitly overrode 2026-05-23)
- **original-scope**: PathLiteral P0 spike artifacts (`feat/pathliteral-spike` worktree) — 199 LOC standalone scanner + 920-file corpus RESULT.md
- **resolution**: 2026-05-23 worktree cleanup batch ran `git worktree remove --force .claude/worktrees/pathliteral-spike && git branch -D feat/pathliteral-spike`. Spike rationale already internalised into PR #367 C1-C3 commit messages so no docs lost
- **size**: S（cleanup only）
- **links**: PR #367 (merged 2026-05-23); CLAUDE.md "NEVER delete `.claude/worktrees/`" rule (user-overridden for this batch)

### FU-2026-05-23-029  ·  surfaced in feat/blindspot-rollout (/simplify pre-pass)  ·  ✅ done in feat/blindspot-rollout commit cb897c15
- **owner**: feat/blindspot-rollout session
- **original-scope**: `ecp impact --baseline <ref> --repo .` panic 在 `impact.rs:809` 因 `node.file_idx = u32::MAX`. Bisect 確認根因是 PR #365 (`feat(schema): RelType::Decorates`) 在 `post_process/decorates_edges.rs:143` emit synthetic Annotation node 用 u32::MAX 當「沒 owning file」sentinel；10 個 consumer (impact ×4、routes ×6) 沒 guard
- **resolution**: commit `cb897c15` 三件套修：(1) `crates/ecp-core/src/graph.rs` 加 `pub const SYNTHETIC_FILE_IDX: u32 = u32::MAX` + `Node::has_owning_file()` + `ArchivedNode::has_owning_file()` helper；(2) `decorates_edges.rs:143` 改用 named const + 完整 producer 註解；(3) `impact.rs` 兩個 site 加 `has_owning_file()` guard
- **deferred-into-followup**: `routes.rs` 6 個同款 site 沒修 — 已在 PR #372 一併解決 (19 sites + 5 bounds-check refactor)
- **size**: S（3 files, +53 LOC，含 const+helper+guard）
- **links**: feat/blindspot-rollout commit `cb897c15`；PR #365 (producer)；PR #372 (downstream completer)

### FU-2026-05-23-013  ·  surfaced in promotion-readiness review (2026-05-23)  ·  ✅ done in PR #374 (merged as c094f9ee, 2026-05-23)
- **owner**: this session
- **renumber-note**: 原號 FU-2026-05-23-004（撞 PR fix/reindex-head-sha-drift 那條），2026-05-23 housekeeping 改為 013
- **original-scope**: `NodeKind::Process` + Leiden community detection + `process_trace::detect_processes` 已實作但 CLI 無 top-level 入口；ecp's only true differentiator 對外部讀者透明
- **resolution**: PR #374 commit `7fe4412f feat(cli): ecp processes + processes trace (FU-013)` — `Commands::Processes` variant + dispatch path + `ProcessesCommands::Trace(<pattern>)` subcommand; new `processes_cmd.rs` integration test (list / cross-community / trace match / not-found) + invariant table entry + `cli_surface_invariants::every_processes_subcommand_has_help`
- **size-actual**: +467 LOC (5 files: main.rs + commands/mod.rs + commands/processes.rs + cli_surface_invariants.rs + tests/processes_cmd.rs)
- **links**: PR #374; `crates/ecp-cli/src/commands/processes.rs`

### FU-2026-05-23-012  ·  surfaced in PR #365 (feat/annotation-decorates-edges)  ·  ✅ done 2026-05-25 (feat/decorates-niche-langs)
- **owner**: fresh session 2026-05-25 (verified the three deferred items still real before working)
- **original-scope**: Go / Ruby / C / C++ Decorates emission unimplemented; JS `parse_js` helper dead-code; FU lumped all four as one "10-of-14 OO-language coverage gap"
- **resolution**: re-bounded scope on triage to keep `Decorates` semantically clean (symbol-decoration only — never file-scope conditions / language constructs without true annotation token):
  - **(1) JavaScript** — added `(decorator)*` capture to `class_declaration` + `export_statement` in `javascript/queries.scm` (mirrors TypeScript pattern; tree-sitter-javascript exposes `decorator` nodes on classes but rejects them on `class_body` — `(decorator) @decorator . (method_definition…)` raised `Impossible pattern`). Layer-1 parser test added (`javascript_decorator_captured_in_raw_node`); `parse_js` helper now reachable, `#[allow(dead_code)]` removed.
  - **(2) Go** — symbol-level pragmas collected via `collect_go_pragmas` walking `prev_sibling` comment chain from `function_declaration` / `method_declaration` / `var_declaration`, gated by an explicit **allowlist** `GO_SYMBOL_PRAGMAS` (noinline / nosplit / noescape / linkname / norace / notinheap / nointerface / nowritebarrier / nowritebarrierrec / yeswritebarrierrec / registerparams / wasmimport / wasmexport / embed). Initial denylist (`build` only) was too permissive — caught in review for letting `//go:generate` (build-pipeline directive, not symbol property) and any future package-scope directives bleed in. Allowlist rejected `//go:build`, `//go:generate`, `//go:debug`, `//go:binary-only-package`, and unknown third-party directives. Two var-emit sites both walk up from `var_spec` to `var_declaration` before collecting. Regression tests pin `//go:build`, `//go:generate`, and unknown directives as NOT producing decorators.
  - **(3) C / C++** — C23 `[[attr]]` (`attribute_declaration` node) + GNU `__attribute__((attr))` (`attribute_specifier` node) collected via shared `framework_helpers::collect_cpp_attributes` walking the declaration's direct children (verified via tree-sitter probe). C++ path preserves the existing `__override__` sentinel by `extend`ing, not replacing.
  - **normalize_decorator** extended with three new branches: `//go:<directive>` (lookup=directive bare name, full_name=`go:<dir>` keeping namespace so `noinline` does not collide with user `@noinline`), `[[attr]]` / `[[ns::attr]]` (lookup=last segment, full=raw), `__attribute__((attr))` (lookup=attr, full=attr).
  - **Ruby** marked permanent wontfix in `docs/language-matrix.md` `[^rb-dec]` — no annotation system; `do…end`/`include`/`prepend` carry different semantics modelled by `Calls` / `Implements` already.
- **size-actual**: 5 files: `javascript/queries.scm` (+5 LOC), `go/parser.rs` (+45 LOC helper + emit sites), `c/parser.rs` (+8 LOC), `cpp/parser.rs` (+5 LOC), `framework_helpers.rs` (+collect helper +3 normalize branches +7 unit tests), `tests/decorates_emission.rs` (+10 tests / +180 LOC), `docs/language-matrix.md` (4 row updates + 4 new footnotes). Suite: 2249 passed / 7 ignored.
- **links**: this entry resolves the FOLLOWUPS.md "Open" stub; rationale-for-//go:build-skip surfaced as design question before implementation (avoided misleading edge for compile-condition); JS class_body method-decorator query attempted and reverted on `Impossible pattern` error — tree-sitter-javascript does not expose method-position decorator currently.

### FU-2026-05-23-014  ·  surfaced in promotion-readiness review (2026-05-23)  ·  ✅ done in PR #376 (merged as b2d27e48, 2026-05-23)
- **owner**: this session (haiku sub-agent dispatch — Bundle A of enum-visibility rollout)
- **renumber-note**: 原號 FU-2026-05-23-005（撞 PR #350 dx-3in1 那條），2026-05-23 housekeeping 改為 014
- **original-scope**: cypher `Process` / `StepInProcess` 範例文件缺；blocked-by FU-013 (now done)
- **resolution**: PR #376 commit `226c0653 docs(cypher)` — 8 Cypher MATCH examples appended to `docs/skills/ecp/_shared/cli/cypher.md` covering: list / find-by-name-substring / step members in order / cross-community processes / upstream callers / per-file density / co-occurring processes / long-tail length-ranking
- **size-actual**: +85 LOC docs
- **links**: PR #376 commit `226c0653`; FU-2026-05-23-013

### FU-2026-05-23-015  ·  surfaced in promotion-readiness review (2026-05-23)  ·  ✅ done in PR #377 (merged as 38012c2b, 2026-05-23)
- **owner**: this session (sonnet sub-agent + post-/simplify HIGH fixes)
- **renumber-note**: 原號 FU-2026-05-23-006（撞 PR #352 decorator-alloc 那條），2026-05-23 housekeeping 改為 015
- **original-scope**: observability gap — codescope's `insight` mode lets enterprise users see per-tool p50/p99 + error rate; ecp had zero telemetry surface
- **resolution**: PR #377 — new `crates/ecp-mcp/src/telemetry.rs` (jsonl append best-effort, `OnceLock<Mutex<BufWriter<File>>>` cached writer per /simplify H3 fix); new `crates/ecp-cli/src/commands/insight.rs` (read side: BufReader window read + p50/p99 + hourly buckets); `crates/ecp-mcp/src/server.rs::call_tool` instrumented with `Instant::now()` + ok/err capture; `Commands::Insight` variant; `ecp-cli::commands::mcp.rs` wires `telemetry::init_repo_id(repo_dir_name_for_cwd(&cwd)?)` at MCP boot. /simplify caught 3 cross-confirmed HIGH issues all fixed pre-merge: (1) bare `file_name()` vs `repo_dir_name_for_cwd` runtime bug (telemetry would be orphaned from rest of `~/.ecp/`); (2) `unix_secs_to_rfc3339` / `days_to_ymd` duplicated 3× (telemetry.rs + insight.rs + insight_cmd.rs) — centralised in new `crates/ecp-core/src/time.rs`; (3) 3 syscalls per MCP call (open+write+close) replaced with cached BufWriter (1 writeln + flush)
- **size-actual**: ~880 LOC across 11 files including new ecp-core time module + telemetry + insight + 2 test files + simplify fix commit
- **deferred-into-followup**: M1 `--telemetry-path` hidden flag in API surface (refactor `build_payload(&Path)`); M2 silent `--hours > 168` clamping; M3 streaming aggregator for huge telemetry files (OOM risk at >168h on multi-month logs)
- **links**: PR #377; `crates/ecp-mcp/src/telemetry.rs`; `crates/ecp-cli/src/commands/insight.rs`; `crates/ecp-core/src/time.rs`

### FU-2026-05-23-020  ·  surfaced in feat/blindspot-rollout (FU-001 schema-cmd)  ·  ✅ done in PR #374 (merged as c094f9ee, 2026-05-23)
- **owner**: this session
- **original-scope**: `ecp_schema` MCP tool auto-derived by `enumerate_tools` had no path for nested `blindspots`/`reltypes`/`node-kinds`/`graph-version` subcommands → MCP clients invoking ecp_schema got empty arg surface
- **resolution**: PR #374 commit `5b5e5b81 feat(mcp): hand-rolled ecp_schema tool with subcmd discriminator` — `#[command(hide = true)]` on Schema variant + new `crates/ecp-mcp/src/schema_mcp.rs` mirror of group.rs / peers.rs pattern + `server.rs` retain-and-extend block. Invariant tests: `every_schema_subcommand_has_help` + `mcp_ecp_schema_subcmds_are_real_cli_paths` + `ecp-mcp/tests/schema_tools.rs` (6 tests for tool registration + argv shape per subcmd)
- **size-actual**: +279 LOC across 6 files
- **links**: PR #374 commit `5b5e5b81`; `crates/ecp-mcp/src/schema_mcp.rs`

### FU-2026-05-23-027  ·  surfaced in PR #368 (inspect-extensions doc audit)  ·  ✅ done in PR #374 (merged as c094f9ee, 2026-05-23)
- **owner**: this session
- **renumber-note**: 原號 FU-020（撞 feat/blindspot-rollout schema-cmd 那條），改 027
- **original-scope**: `docs/language-matrix.md` 沒有「per-language schema emission 矩陣」— 各語言覆蓋度差異大，LLM/maintainer 要 grep test file 才知道哪語言支援哪邊
- **resolution**: PR #374 commit `b59758a0 docs(language-matrix)` — new "Schema emission coverage" section with 14 mainstream langs × 6 recently-added edge types (Implements / EnumVariant / Decorates / TransactionScope / PathLiteral / Fetches) using ✓ / partial / — / n/a cells, footnoted to per-lang gap rationale + tracked follow-up IDs (FU-009/011/012/017/018). Uniformly-emitted edges listed below the table for completeness; cross-link to runtime equivalents `ecp schema reltypes` / `ecp schema blindspots`
- **size-actual**: +79 LOC pure markdown
- **links**: PR #374 commit `b59758a0`; `docs/language-matrix.md` "Schema emission coverage" section

### FU-2026-05-23-011  ·  surfaced in PR #364 (feat/enum-variant-nodes)  ·  ✅ done in PR #376 (merged as b2d27e48, 2026-05-23) — expanded scope
- **owner**: this session (5 parallel sonnet sub-agents)
- **original-scope**: Python + PHP 8.1+ EnumVariant; Go/Ruby/JS 留無 emission
- **expansion**: user feedback inverted the "保留無 emission" decision — imitation patterns should surface as BlindSpot (not pollute EnumVariant schema), so the LLM querying `(n:EnumVariant)` doesn't conclude "no enums" when the codebase uses pre-Enum / class-const / Object.freeze / module-constant / iota patterns
- **resolution**: PR #376 — 5-lang EnumVariant + 5-lang imitation BlindSpot rollout (Python true Enum + class-as-enum imitation; PHP 8.1+ true enum + class-const imitation; Ruby module-as-enum BlindSpot; JS Object.freeze BlindSpot; TS Object.freeze + as-const BlindSpot; Go iota-const-block BlindSpot). 5 new BlindSpot kinds via existing `push_blind_spot` mechanism; all heuristics conservative (frozen-string skipped, function-valued objects rejected, < 2-entry rejected). 8 commits from 7 parallel sub-agents integrated via cherry-pick onto one branch.
- **size-actual**: +1819 LOC across 17 files; full ecp-analyzer suite 2183 passed, 6 ignored
- **deferred-into-followup**: FU-2026-05-23-032 (perf-not-yet-measured 2nd full-tree DFS in Python + TS); FU-2026-05-23-033 (4 mechanical cleanups — Go subtree dedup, PHP returns Vec vs &mut, JS/TS SCALAR_VALUE_KINDS sharing, Ruby _source unused)
- **links**: PR #376; FU-2026-05-23-011 → FU-2026-05-23-032 / FU-2026-05-23-033

### FU-2026-05-23-009  ·  surfaced in PR #363  ·  ✅ done in PR #380 (merged as bc596769, 2026-05-23)
- **owner**: this session (6 parallel sub-agents + integration + /simplify fixes)
- **original-scope**: T10 TransactionScope only covers 5 langs/frameworks (Python/Django, Java/Spring, Kotlin/Spring, C#/.NET, PHP/Symfony); expand to TS TypeORM, Rust `#[transaction]`, Dart, Go, Ruby, Swift
- **resolution**: PR #380 — 9 commits: 1 setup (FrameworkId 6-variant pre-allocation + 2 annotation helpers + design notes for non-annotation patterns) + 1 from_u8 fix (caught by 4 of 6 agents independently) + 5 lang implementations + 1 lang audit + 1 integration-recovery commit (recovered 14 lost tests + /simplify fixes). Detector patterns: TS TypeORM (annotation, mirror of Spring), Rust #[transaction] (annotation, mirror of Symfony), Go db.Begin() (NEW call-site pattern, per-fn dedup), Ruby Model.transaction do (NEW block-form pattern, per-fn dedup with HashSet — /simplify caught Vec::contains O(K²) bug pre-merge), Dart Drift/Firestore .transaction(closure) (call-site closure-arg pattern), Swift audit-only (no consensus framework: Core Data / GRDB / Realm / SQLite.swift — slot reserved per future demand). Bumps 5/14 → 10/14 main-lang × framework. Consolidates FU-2026-05-23-018 (Ruby block-form was carved out as separate; rolled into this PR per session directive).
- **size-actual**: ~1100 LOC across 11 files; tx_scope_emission 40 tests pass, 2207 ecp-analyzer suite
- **deferred-into-followup**: FU-2026-05-23-034 (span-area formula divergence between Go / Dart / Ruby detectors), FU-2026-05-23-035 (Go nested-if pyramid not extracted to predicate fn like Ruby/Dart), FU-2026-05-23-036 (Swift 31-LOC audit doc block belongs in docs/language-matrix.md not source)
- **integration-lesson**: `-X theirs` cherry-pick strategy silently drops earlier agents' tx_scope_emission.rs trailing-append tests when later agents conflict-resolve in append-only order. 14 tests lost across TS/Rust/Go agents; recovered by extracting test functions from each agent's worktree and manually appending. Future multi-agent integrations should `grep -c '^#[test]'` to verify test count matches expected sum.
- **links**: PR #380; FU-2026-05-23-018 (consolidated); FU-2026-05-23-034 / 035 / 036 (deferred)

### FU-2026-05-23-018  ·  surfaced in PR #361+ (tx-scope-emission)  ·  ✅ done in PR #380 (merged as bc596769, 2026-05-23) — consolidated into FU-009
- **owner**: this session (Ruby sub-agent, FU-009 rollout)
- **renumber-note**: 原號 FU-2026-05-23-009（撞 PR #363 T10 5-langs 那條），2026-05-23 housekeeping 改為 018
- **original-scope**: TransactionScope SQL-block detection (Kotlin Exposed `transaction { ... }`, Ruby `Model.transaction do ... end`, raw `BEGIN; ... COMMIT;` SQL) carved out as separate from annotation-form FU-009
- **resolution**: consolidated into PR #380 commit `6b4d4a23 feat(ruby): Model.transaction do...end block-form`. Ruby block-form implemented inline (NEW block-form detector pattern: walk AST for `call` node with method=`transaction` + `do_block` child, recover enclosing function via point_in_span, per-fn HashSet<u32> dedup). Kotlin Exposed `transaction { ... }` + raw SQL block forms NOT covered in this PR — slot still open if user demand surfaces (Kotlin uses Spring `@Transactional` which is already covered; Exposed lambda-block is a separate Kotlin idiom).
- **deferred**: Kotlin Exposed lambda-form (no FU filed — file fresh if needed); raw SQL `BEGIN; COMMIT;` (unlikely worth modelling separately given parser doesn't AST-parse SQL inside strings)
- **size-actual**: included in FU-009 resolution above
- **links**: PR #380 commit `6b4d4a23`; FU-2026-05-23-009 (the parent FU it consolidated into)

### FU-2026-05-22-003  ·  surfaced in PR #345  ·  ✅ done in PR #372 (merged as fdfdd6ea, 2026-05-23)
- **owner**: unassigned
- **original-scope**: `ecp coverage` 與 `ecp group coverage` 別名（為一 release 向後相容）→ 一 release 後拔掉
- **resolution**: PR #372 commit `ad2913c chore(cli)` 拔 `coverage` 別名 + retire 兩個 back-compat test (`coverage_alias_still_routes_to_summary` + `group_coverage_alias_help_exits_zero`)
- **size-actual**: S
- **links**: PR #345; PR #372 commit `ad2913c`; `crates/ecp-cli/src/main.rs` Summary variant; `crates/ecp-cli/src/commands/group/mod.rs` Summary variant

### FU-2026-05-22-004  ·  surfaced in PR #345  ·  ✅ done in PR #372 (merged as fdfdd6ea, 2026-05-23)
- **owner**: unassigned
- **original-scope**: `ecp admin verify-resolver` 別名（為一 release 向後相容）→ 一 release 後拔掉
- **resolution**: PR #372 commit `ad2913c chore(cli)` 拔 `verify-resolver` admin variant + dispatch arm；保留 `ecp dev verify-resolver` 為唯一路徑
- **size-actual**: S
- **links**: PR #345 commit `feat(cli-dev): hidden ecp dev namespace`; PR #372 commit `ad2913c`

### FU-2026-05-23-005  ·  surfaced in PR #350 (dx-3in1)  ·  ✅ done in PR #372 (merged as fdfdd6ea, 2026-05-23)
- **owner**: unassigned
- **original-scope**: PR #350 新增 `CommitBuildMeta.binary_commit_sha` 後，`ecp summary` 仍未讀取它 → 圖建立時 binary commit ≠ 現在跑的 binary commit 時無法 surface warning
- **resolution**: PR #372 commit `fbe2aaa feat(summary)` 加 `graph_builder_sha` / `current_binary_sha` 欄 + stderr warn；若兩者皆 Some 且不同會引導跑 `ecp admin index --force`
- **size-actual**: S（~30 LOC + 1 test）
- **links**: PR #350 commit `feat(cli): embed git short-SHA…`; PR #372 commit `fbe2aaa`; `crates/ecp-cli/src/commands/summary.rs`; `crates/ecp-core/src/registry/commit_meta.rs::CommitBuildMeta::binary_commit_sha`

### FU-2026-05-23-016  ·  surfaced in PR #359  ·  ✅ done in PR #372 (merged as fdfdd6ea, 2026-05-23)
- **owner**: session a59bfc41-5e8c-4bd3-974e-a8a0215ab73b (sub-projects-1-5-spec worktree)
- **renumber-note**: 原號 FU-2026-05-23-007（撞 promotion-readiness review 競品 benchmark 那條），2026-05-23 housekeeping 改為 016
- **original-scope**: `stamp_owner_class_by_span` 只把 class-type containers 的 children 標 `owner_class`，Namespace / Module containers (C# `namespace`, PHP `namespace`, Rust `mod`, Python `module`, C++ `namespace`) 的 children 全部帶 `owner_class=None`。結果 PR #359 新增的 `scope_defines::Pass2` (Namespace/Module → child 邊) 對所有現有 parser 都不會 fire — children 全部走 Pass1 (File→top-level) 代償，邏輯正確但 graph 表達失真
- **resolution**: PR #372 commit `57a36c4 feat(scope-defines)` 把 Namespace + Module 折進 container pool (tightest-span wins, 保留既有 owner_class 避免 clobber Rust `enclosing_impl_type`)；C# / PHP / C++ / Rust 都會 emit `Namespace/Module → child` Defines 邊；`defines_emission.rs` 新增 assertion 確認 Pass2 真的 fire
- **size-actual**: M
- **links**: PR #359; PR #372 commit `57a36c4`; `crates/ecp-analyzer/src/post_process/scope_defines.rs`

### FU-2026-05-23-017  ·  surfaced in PR #358  ·  ✅ done in PR #372 (merged as fdfdd6ea, 2026-05-23)
- **owner**: session a59bfc41-5e8c-4bd3-974e-a8a0215ab73b (sub-projects-1-5-spec worktree)
- **renumber-note**: 原號 FU-2026-05-23-008（撞 promotion-readiness review agent-distribution 那條），2026-05-23 housekeeping 改為 017
- **original-scope**: Kotlin parser 把 `interface` 跟 `class` 視為同一種。`crates/ecp-analyzer/src/kotlin/queries.scm` 只有 `(class_declaration ...)` 跟 `(object_declaration ...)`，完全沒有 `(interface_declaration ...)` 的處理 → Kotlin `interface Foo {}` 跟 `class Foo {}` 一樣都進 `NodeKind::Class` 路徑。PR #358 (Implements kind-dispatch) 在 production 會誤把 Kotlin `class Bar : Foo`（Foo 是 interface）emit 為 `Extends` 而非 `Implements`
- **resolution**: PR #372 commit `9d21a91 feat(kotlin)` 加 `is_interface_class()` demotion arm (mirror `is_enum_class()`)，並把 PR #358 的 raw-LocalGraph workaround test 換成真 parser test
- **size-actual**: S
- **links**: PR #358; PR #372 commit `9d21a91`; `crates/ecp-analyzer/src/kotlin/queries.scm`; `crates/ecp-analyzer/src/kotlin/parser.rs`

### FU-2026-05-23-019  ·  surfaced in PR #360 (Fetches)  ·  ✅ done in PR #372 (merged as fdfdd6ea, 2026-05-23)
- **owner**: takeover-session-fetches (foreground)
- **renumber-note**: 原號 FU-2026-05-23-013（撞 PR feat/pathliteral-full 的 spike-cleanup 那條），2026-05-23 housekeeping 改為 019
- **original-scope**: 兩個 CLI-level integration tests 在 #360 fixtures 上 fail：(1) `shape_check_ab::ab_upstream_fixtures_three_clean_one_drift` 期望 ≥4 Fetches edges、實際 emit 3；(2) `review_verdicts_cross_lang_test::cross_lang_ripple_escalates_modified_route_to_risk` 期望 modified route + literal-URL consumer 升 RISK、實際停在 WARN
- **resolution**: PR #372 commit `00f55dd fix(fetch-urls)` — 真兇是 `FETCH_WITH_METHOD` regex 要 quoted key，但 `fetch('/x', { method: 'POST' })`（JS object-shorthand，TS/TSX 主流寫法）的 bare `method` 沒被 match，silent fall through 到 bare-fetch GET fallback。改 `[\"']?method[\"']?` + 新 unit test，兩個 ignored fixture 都 re-enabled
- **size-actual**: S
- **links**: PR #360; PR #372 commit `00f55dd`; `crates/ecp-analyzer/src/fetch_shape/fetch_urls.rs` FETCH_WITH_METHOD regex; `crates/ecp-cli/tests/shape_check_ab.rs:205`; `crates/ecp-cli/tests/review_verdicts_cross_lang_test.rs:141`
### FU-2026-05-23-023  ·  surfaced in PR #367 (feat/pathliteral-full)  ·  ✅ done in PR #385 (merged as ffd1643f, 2026-05-23)
- **owner**: unassigned
- **original-scope**: PathLiteral sink classification leaves chained-call patterns mis-classified. `File("x.json").readText()` (Kotlin) / `File("x.json").write_text(...)` (Python) / `std::ifstream("x")` (C++) — the immediate parent `call_expression` of the literal is the constructor (`File`/`ifstream`), so `classify_sink` returns `sink:join|medium` rather than `sink:read|high` / `sink:write|high`. Literal value extraction + enclosing-fn resolution are correct; only the sink tag is conservative.
- **why-deferred**: requires flow analysis (which chained `.readText()` is the terminal operation?). Out of PR #367 scope — the split-brain detection use case (LLM finding readers/writers by value match) still works, just with `sink:join` instead of `sink:read` on Kotlin/Swift idioms. Documented in PR #367 description + `tests/kotlin_path_literals.rs` test comments.
- **next-action**: extend `path_literals.rs` extractors to walk chained `member_expression` / `field_expression` outward from the call_expression and re-classify when the terminal call is in the read/write whitelist. Per-lang work (Kotlin `navigation_expression`, Swift `navigation_expression`, Python `attribute`, etc.).
- **size-actual**: M (per-lang ~30-50 LOC × 5-7 langs that use this idiom)
- **links**: `crates/ecp-analyzer/src/path_literal.rs::classify_sink`; `crates/ecp-analyzer/tests/kotlin_path_literals.rs` (commented test waiver); PR #367 description "Known scope / follow-ups" section


### FU-2026-05-23-024  ·  surfaced in PR #367 (feat/pathliteral-full)  ·  ✅ done in PR #385 (merged as ffd1643f, 2026-05-23)
- **owner**: unassigned
- **original-scope**: `with_extension("json")` / `with_extension("toml")` etc. don't surface as PathLiteral because the literal value `"json"` doesn't pass `is_path_shaped` (no `/`, no suffix that ends the string). Only `with_file_name("session_meta.json")` works because the value matches the suffix predicate. Limits split-brain detection in repos that compose paths via extension swaps.
- **why-deferred**: the predicate is intentionally conservative — adding bare extensions would FP on every error code / option name that happens to match (`"json"` could be a serialisation-format token). Proper fix needs sink-name carve-out: when callee is `with_extension` / `set_extension`, accept any short non-empty value as a PathLiteral with `sink:ext-change`.
- **next-action**: in `path_literal.rs::classify_sink`, when the callee resolves to a HIGH-confidence ext-change name, return Some((ExtChange, High)) and have the per-lang extractor accept the literal regardless of `is_path_shaped` verdict. Equivalent to a "sink-override" path that runs before the predicate filter.
- **size-actual**: S (~30 LOC core + 14 lang extractor tweak)
- **links**: `crates/ecp-analyzer/src/path_literal.rs::is_path_shaped`; PR #367 description "Known scope / follow-ups" §2


### FU-2026-05-23-037  ·  surfaced in PR #378 /simplify pre-merge review  ·  ✅ done in PR #385 (merged as ffd1643f, 2026-05-23)
- **owner**: unassigned
- **original-scope**: 10+ integration-test fixtures (incl. the 6 new `silent_drop_*.rs` added in PR #378 plus existing `class_membership_inspect.rs`, `inspect_module_qualified_calls.rs`, `imports_edge_inspect.rs`, `inspect_cpp_namespace_qualified_calls.rs`, ...) each redefine the same three helpers verbatim: `ecp_bin() -> &'static str`, `write(repo, rel, body)`, `init_and_analyze(repo)`. `tests/common/mod.rs` already exports `ecp_bin()` + `write_graph()` + `run_git()` but the duplicates predate it. The module's doc comment explicitly says "wrapping `setup_repo*` functions stay per-file because fixtures vary" — but our 6 new fixtures all use the *identical* layout (single git init + add + commit + admin index), so the variance argument doesn't apply here.
- **why-deferred**: cross-cutting cleanup that touches ~10 files; coherent migration is its own chore PR, not bundled with the silent-drop audit. Reuse reviewer (HIGH) explicitly noted "older files can be migrated opportunistically (minor, don't bundle)".
- **next-action**: chore PR `chore/promote-fixture-helpers-to-common`: (1) add `pub fn write(repo: &Path, rel: &str, body: &str)` + `pub fn init_and_analyze(repo: &Path)` to `crates/ecp-cli/tests/common/mod.rs`; (2) migrate all 10+ fixtures that currently inline these helpers to `mod common; use common::{ecp_bin, write, init_and_analyze}`; (3) drop the per-file copies.
- **size-actual**: M (~200 LOC mechanical, 10+ files)
- **links**: PR #378 /simplify reuse review (HIGH); `crates/ecp-cli/tests/common/mod.rs`; affected fixtures via `rg -l "fn init_and_analyze" crates/ecp-cli/tests/`



### FU-2026-05-23-038  ·  surfaced in PR #378 /simplify pre-merge review  ·  ✅ done in PR #385 (merged as ffd1643f, 2026-05-23)
- **owner**: unassigned
- **original-scope**: `shape_check::render_text` (`crates/ecp-cli/src/commands/shape_check.rs:223`) re-reads `unparseable_fetches`, `unknown_target_shapes_total`, `unknown_target_shapes_truncated` from `value: &serde_json::Value` after the typed counters already existed in `build_payload_with_hints`. The JSON round-trip via `.as_u64() / .as_bool() / unwrap_or(0)` is a leaky abstraction — `render_text` is reaching back into the just-serialized JSON.
- **why-deferred**: fixable by widening `render_text` signature to take typed `(unparseable_fetches: u64, unknown_total: u64, unknown_truncated: bool)` directly, but quality reviewer self-flagged as MEDIUM / non-blocker; not worth a separate PR by itself.
- **next-action**: next time `shape_check.rs` is touched, refactor `render_text(value: &Value)` → `render_text(value: &Value, counters: &ShapeCheckCounters)` or pass the 3 fields directly. Update both call sites in `pub fn run`.
- **size-actual**: S (~20 LOC; widen signature + threading)
- **links**: PR #378 /simplify quality review (MEDIUM); `crates/ecp-cli/src/commands/shape_check.rs:223-243`


### FU-2026-05-23-040  ·  surfaced in PR #378 /simplify pre-merge review  ·  ✅ done in PR #385 (merged as ffd1643f, 2026-05-23)
- **owner**: unassigned
- **original-scope**: `cypher::build_payload` (`crates/ecp-cli/src/commands/cypher.rs:122`) carries an empty-row fallback `match row.pop() { Some(v) => v, None => { eprintln!; Null } }`. The fallback path is theoretically unreachable (executor invariant: `row.len() == columns.len()`), and the existing unit test `build_payload_single_column_empty_row_yields_null` pins the null-fallback contract on the public-API side. But there's no test ON THE EXECUTOR SIDE asserting the invariant holds — if a future projection refactor produces empty rows, the silent null surfaces with only a stderr warning.
- **why-deferred**: PR #378 audit added the stderr warning + kept null fallback for backwards compatibility; pinning the invariant in the executor is a separate cypher correctness concern, not part of the silent-drop audit.
- **next-action**: in `crates/ecp-core/src/cypher/executor.rs`, add unit test asserting `result.rows.iter().all(|row| row.len() == result.columns.len())` for representative single-column / multi-column / aggregation queries. If the invariant is documented but not enforced, also add `debug_assert!` at the executor's row-emission site.
- **size-actual**: S (~30 LOC unit tests + optional debug_assert)
- **links**: PR #378 M3 audit; `crates/ecp-cli/src/commands/cypher.rs:113-145`; `crates/ecp-core/src/cypher/executor.rs`



### FU-2026-05-23-043  ·  surfaced in PR #379 /simplify pre-merge review  ·  ✅ done in PR #385 (merged as ffd1643f, 2026-05-23)
- **owner**: unassigned
- **renumber-note**: 原號 FU-2026-05-23-036（撞 PR #380 /simplify Swift audit-relocation 那條），2026-05-23 housekeeping 改為 043
- **original-scope**: 3 個 minor cleanup 在 `pr_analyze.rs`：
  (1) `changed_files_pb: Vec<PathBuf>` (line 210-211) 是 redundant derived state — `classify_area` 只 call `p.to_string_lossy()`，簽名改成 `&[impl AsRef<Path>]` 或 `&[String]` 可消掉中間 allocation；
  (2) `match args.format` catch-all 把 `OutputFormat::{Llm, Toon, Text}` 路由到一個 bare `eprintln!`，靜默誤導 — 應顯式拒絕（"pr-analyze only supports --format json"）或顯式 match 4 個 variant；
  (3) `ImpactJson::changed_files` / `impact_set_names` (line 126-144) 每個 element clone 兩次（HashSet insert + map collect），改 `filter_map(|s| seen.insert(&s.file_path).then(|| s.file_path.clone()))` 省一次 clone
- **why-deferred**: 全 Minor severity，不阻 PR #379；下次動 pr_analyze 時順手修
- **next-action**: 下次 touch pr_analyze.rs（如 FU-2026-05-23-042 動工時）順手修這 3 條
- **size-actual**: S（~30 LoC mechanical）
- **links**: PR #379 /simplify quality review findings 6/7/9; `crates/ecp-cli/src/commands/dev/pr_analyze.rs:114-155,210-211,293-311`



### FU-2026-05-23-030  ·  surfaced in PR #368 CI smoke timeout (parity_gate_smoke)  ·  ✅ done in PR #373 (merged as 67a92fe1, 2026-05-23)
- **renumber-note**: 原號 FU-029（撞 feat/blindspot-rollout `ecp impact --baseline` panic 那條），改 030
- **owner**: session on `perf/parity-smoke-pipeline-cache` worktree
- **original-scope**: `crates/ecp-cli/tests/incremental_full_parity.rs::parity_gate_smoke` doc 寫「~5s on dev machine」，2026-05-23 實測本機 31.58s（6x regression），CI 4-shard 競 CPU 再 4x → SLOW [>120s] 警示。**不是 PR #368 引入**，是近期 main 多個 post-process 累積成本。本 FU 最初推測 #365 Decorates 的 `Resolver::new(&symbol_table).with_path_aliases(...)` 每 LocalGraph 重建是元兇。
- **resolution**: PR #373 root-cause: `parse_direct` 每次都 call `make_pipeline()` 重建 21 providers + capture queries（cold 0.640s × ~60 parses = 38s）。`pipeline.analyze()` 完全不走 `ResolutionBuilder.build()`，所以 post-process pass 不可能是元兇 — 本 FU 的元兇推測錯了。Fix 是改用 cached `pipeline()` accessor。實測 32.11s → 0.83s（39×）。附 diagnostic bench `examples/bench_parse_pipeline.rs` 給未來同類 regression 用
- **size-actual**: S（~50 LOC 改用 cached pipeline + diagnostic bench example）
- **links**: PR #368 CI run；PR #373 merge commit `67a92fe1`；`crates/ecp-cli/tests/incremental_full_parity.rs:305`；`crates/ecp-cli/examples/bench_parse_pipeline.rs`

### FU-2026-05-23-045  ·  surfaced during PR #395  ·  ✅ done in PR #396 (merged as 4a9c40b2, 2026-05-23)
- **owner**: this session
- **original-scope**: `wait_for_completion_prefers_latest_generation_for_same_sha` flaked intermittently because `CommitIndex::scan` used raw mtime as tie-breaker — when same-SHA `base` and `gen_dir` are written back-to-back, ext4 mtime can tie (~10ms / 1ns resolution per `discard_inode` flag + kernel version) and **read_dir insertion order** (filesystem-dependent) decides the winner, NOT alphabetical order. Not introduced by any specific PR.
- **resolution**: 3 commits on `fix/commit-index-generation-tiebreaker`:
  - `eef4c7d8 fix(commit-index): generation-tuple tie-breaker beats mtime race` — `CommitDirName` now carries `generation: Option<Generation>`, `CommitIndex::scan` sorts by `(Option<Generation>, mtime)` so `None < Some(_)` + 3-tuple lex order dominate mtime. Deterministic regression test forces `base.mtime` 1h AHEAD of `gen_dir.mtime` and pins gen_dir winning.
  - `22ff4705 refactor(commit-index): /simplify cleanup — single SoT for .gen suffix` — `Generation::format_suffix()` extracted as single SoT for `.gen.X.Y.Z` on-disk shape (used by both `CommitDirName::format` and `publish_dir_for`); `commit_dir_freshness` → `commit_dir_mtime` rename (mtime is now secondary tie-breaker).
  - `a737ab7c fix(test): open meta.json writable so Windows SetFileTime accepts set_modified` — Windows CI surfaced `Os { code: 5, kind: PermissionDenied }` at the new regression test because `File::open` returns a GENERIC_READ-only handle and Windows `SetFileTime` requires `FILE_WRITE_ATTRIBUTES` (rust-lang/rust#95558). Switched both `set_modified` callsites to `OpenOptions::new().write(true).open(...)` via a local `open_writable` closure. Pure stdlib, no new dep, content preserved (no truncate), Linux futimens path unaffected.
- **size-actual**: S (~40 LOC core fix + simplify cleanup + Windows compat fix)
- **links**: PR #396 merge commit `4a9c40b2`; `crates/ecp-cli/src/build/orchestrator.rs:644-752`; `crates/ecp-cli/src/commit_lookup.rs`; `crates/ecp-core/src/registry/dirname.rs`; rust-lang/rust#95558

### FU-2026-05-23-047  ·  surfaced during PR #395 + PR #396 full-suite runs  ·  ✅ done in PR #399 (merged as 9d87bac1, 2026-05-23)
- **owner**: this session
- **original-scope**: Two CLI integration tests (`review_verdicts_intra_caller_marks_warn`, `indirect_dispatch_verdict_fires_on_rust_dyn_trait_in_diff`) flaked under full-suite parallel `cargo test` with `git stash push failed: warning: failed to remove .ecp/tmpXXX/commits/branch_main__SHA/tantivy: Directory not empty`. FU originally hypothesised TempDir Drop racing the background tantivy writer.
- **resolution-with-revised-diagnosis**: The actual race isn't TempDir Drop — it's that `auto_ensure::ensure_fresh` (called by `main.rs:259` on every CLI command) discards `BuildResult` via `?` so `tantivy_handle` is silently dropped; the spawned writer keeps appending to `tantivy/` while the same subprocess's `GitGuard::enter` runs `git stash push -u`, which enumerates+removes untracked `.ecp/.../tantivy/` and trips "Directory not empty". The FU's original `cfg(test)` newtype suggestion is unviable — integration tests spawn the **release** `ecp` binary as a subprocess, `cfg(test)` doesn't propagate. In normal prod `~/.ecp/` is sibling to the repo, not nested, so the race is unreachable; only fixtures that overload `HOME=$REPO` (and rare edge cases like `git init` inside `$HOME`, or `ECP_HOME=$REPO/.cache`) put the cache inside the worktree. Two-part fix on `fix/tantivy-drain-on-ensure-fresh`:
  - Fixture isolation in `review_verdicts_test.rs` + `review_verdicts_indirect_dispatch_test.rs` — separate `TempDir` for `HOME` so `.ecp/` lives outside the worktree (zero perf cost, root-cause fix for the two known flakes).
  - `auto_ensure::drain_tantivy_if_inside_worktree` defensive check — when `resolve_home_ecp().starts_with(worktree_root)`, sync-join the tantivy handle. ~10μs prefix-check probe in common prod path; the ~270ms join penalty only fires when the cache root genuinely nests under the worktree.
  - Regression test `tests/ensure_fresh_tantivy_drain.rs` asserts both branches via `test_counters::TANTIVY_JOIN_COUNT` (1 for nested, 0 for sibling).
- **verification**: 5 sequential runs of both originally-flaky tests passed; full `cargo test -p egent-code-plexus --tests` 1145 passed / 10 ignored; `cargo clippy -p egent-code-plexus --tests` clean.
- **size-actual**: S (~30 LOC prod + ~50 LOC regression test + 2-line fixture isolation each × 2)
- **links**: PR #399 merge commit `9d87bac1`; `crates/ecp-cli/src/auto_ensure.rs` (drain_tantivy_if_inside_worktree, TANTIVY_JOIN_COUNT); `crates/ecp-cli/src/build/orchestrator.rs:218-240` (background spawn unchanged); `crates/ecp-cli/tests/ensure_fresh_tantivy_drain.rs` (regression coverage)

### FU-2026-05-23-034  ·  surfaced in PR #380 /simplify pre-merge review  ·  ✅ done in PR #394 (merged as 6fab1368, 2026-05-23)
- **owner**: this session
- **original-scope**: Span-area formula divergence across 3 new tx-scope detectors. Go's `min_by_key` in `collect_go_sql_tx_scopes` (`crates/ecp-analyzer/src/go/parser.rs:167-173`) duplicated `framework_helpers::span_area()`; Dart's version (`dart/parser.rs:494-497`) used a different `<< 16` shift that ignored start_col; Ruby used `find_map` first-match (no min-area selection at all). Nested-function scenarios would resolve different "smallest enclosing fn" choices across the three detectors. Go's `seen` was `HashSet<usize>` (inferred), Dart's `HashSet<u32>` (explicit) — cosmetic but inconsistent.
- **resolution**: Bundled with FU-044 as the "correctness bundle" PR #394. Extracted shared `enclosing_fn_idx_by_span(nodes, row, col) -> Option<u32>` helper in `framework_helpers.rs` using `span_area` for consistent smallest-fn selection; Go / Ruby / Dart detectors all routed through it with `HashSet<u32>` dedup. 4 unit tests cover the smallest-area invariant. 40-test `tx_scope_emission` suite passes unchanged.
- **size-actual**: M (~80 LOC core + per-lang regression test)
- **links**: PR #394 merge commit `6fab1368`; PR #380 simplify reviewer reports; `crates/ecp-analyzer/src/framework_helpers.rs::enclosing_fn_idx_by_span`; FU-2026-05-23-009 (parent tx-scope feature)

### FU-2026-05-23-044  ·  surfaced in PR #385 rebase review  ·  ✅ done in PR #394 (merged as 6fab1368, 2026-05-23)
- **owner**: this session
- **original-scope**: `ecp impact --baseline --format json` emitted `changed_symbols[*].filePath` (semantic-derived) but no flat git-diff-derived `changed_paths` array. `pr-analyze::run()` therefore needed a SECOND subprocess (`git diff --name-only`) to get whitespace-only / comment-only file changes for area classification — two subprocesses + two sources of truth, with `ecp impact` internally already running git diff. Risked divergence if git state shifted between the two invocations.
- **resolution**: Bundled with FU-034 as the "correctness bundle" PR #394. `ImpactJson` now carries `changed_paths: Vec<String>` (un-filtered git diff list); `pr_analyze::run()` consumes it directly and the `git_diff_files` second-subprocess shim is gone. 3 end-to-end integration tests cover docs-only / mixed code+docs / pure code diffs. Forward-compat serde default keeps older subprocess outputs parseable.
- **size-actual**: S (~50 LOC: impact.rs payload add + pr_analyze refactor + 3 integration tests)
- **links**: PR #394 merge commit `6fab1368`; PR #385 review thread on `pr_analyze.rs` rebase; FU-2026-05-23-043 (sibling); `crates/ecp-cli/src/commands/impact.rs::build_baseline_payload` (envelope extended); `crates/ecp-cli/src/commands/dev/pr_analyze.rs` (subprocess removed)

### FU-2026-05-23-008  ·  surfaced in promotion-readiness review (2026-05-23)  ·  ✅ done in PR #395 (merged as f763d222, 2026-05-23) — partial, sub-part (c) → FU-2026-05-23-048
- **owner**: this session
- **original-scope**: Agent integration distribution gap, three parts — (a) Cursor / Windsurf / Cline 缺 turn-key MCP snippet（README:186 only showed Claude Code）；(b) Codex CLI 要 `git apply patch` 到 fork（高門檻、流程冗長）；(c) 缺 `npx @ecp/install` 或 `brew install ecp` — `curl install.sh` 流程不會被廣傳。
- **resolution**: Sub-parts (a) + (b) shipped in PR #395: README now has copy-paste MCP snippets for Cursor / Windsurf / Cline; `ecp admin codex install native-tools --auto-fork` runs `gh repo fork openai/codex --clone` + `git apply` in one shot (default fork dir `~/.config/ecp/host-integration/codex-fork/`, overridable via `--fork-dir` / `$ECP_CODEX_FORK_DIR`). 3 unit tests pin the fork-dir precedence ladder. Sub-part (c) — Homebrew tap + npm wrapper — carried out as **FU-2026-05-23-048** (still Open) because each is its own M-sized chunk needing its own packaging + release-pipeline work; bundling would have ballooned PR #395 scope.
- **size-actual**: M (a+b combined); sub-part (c) carried as FU-048
- **links**: PR #395 merge commit `f763d222`; README.md MCP snippets section; `crates/ecp-cli/src/commands/admin/codex.rs` (--auto-fork); FU-2026-05-23-048 (carried sub-part c)

### FU-2026-05-23-031  ·  surfaced in PR #368 (inspect-extensions doc audit)  ·  ✅ done in PR #395 (merged as f763d222, 2026-05-23)
- **owner**: this session
- **renumber-note**: 原本想用 FU-028（被 feat/blindspot-rollout commit 7c13046c 佔），改 031
- **original-scope**: Skill 內容雙頭：`docs/skills/ecp/SKILL.md`（45 行 repo-tracked，layer-1 entry-point summary）與 `~/.claude/skills/ecp/SKILL.md`（170+ 行 global，含完整 NodeKind/RelType 清單跟 cypher 範例）內容 diverge。Schema 補完只發生在 repo 版的 `_shared/refs/cypher-subset.md`（PR #368 加），global 版 NodeKind/RelType 列表仍漏 Annotation / EnumVariant / TransactionScope / Decorates / OpensTxScope。
- **resolution**: 走 CLI subcommand 路線而非 install script — `ClaudeSkillTarget` 新增 `Ecp` variant；`ecp admin claude install skills ecp` 從 canonical `docs/skills/ecp/` 來源（Codex pattern 留在 `skill_sample/claude/`）；`Skills::All` 現在同時展開 `Ecp` 與 `Simplify`。`docs/skills/README.md` 更新成 surface CLI path 為 official machine-driven re-sync，LLM-driven 預設，手動 `cp` fallback。"no install script by design" 原則保留 — 不引入獨立 shell / Makefile / rsync hook。
- **size-actual**: S (Skill target enum 擴充 + CLI dispatch + docs)
- **links**: PR #395 merge commit `f763d222`; PR #368 (`docs/skills/ecp/_shared/refs/cypher-subset.md`)；`crates/ecp-cli/src/commands/admin/claude_code.rs` (ClaudeSkillTarget::Ecp)；`docs/skills/README.md`

### FU-2026-05-23-032  ·  surfaced in PR #376 /simplify pre-merge review  ·  🚫 wontfix (2026-05-24, measured by orchestrator session c7cba51f)
- **original-scope**: Python `parse_file` 跑完 tree-sitter query loop 後又起第二次完整 AST DFS 找 `class_definition`（`crates/ecp-analyzer/src/python/parser.rs:1348-1381`），且每個 class 命中時 `process_class_for_enum` 再做 `nodes.iter().find(|n| n.span == class_span && n.kind == NodeKind::Class)` 線性掃。TS 同款獨立 DFS 在 `crates/ecp-analyzer/src/typescript/parser.rs:658` 的 `collect_freeze_enum_spots`。PR #376 /simplify reviewer 估算 +14% (Py) + ~10% (TS) cold-ingest，但 **unmeasured**。
- **measurement** (2026-05-24): `python scripts/benchmark/benchmark_ecp.py --runs 1` against `.sample_repo` (22858 files; 1704 TS, 136 Python). Methodology: patch both DFS sites to no-op, rebuild release, run cold-ingest 3×; then `git checkout -- ...` to restore + rebuild + run 3× more.
  - Baseline (DFS on):  2.22s, 2.22s, 2.24s → **median 2.22s**, tight spread (~20ms)
  - Patched (DFS off): 2.05s, 2.16s, 2.57s → **median 2.16s**, noisy spread (~520ms; 2.57s outlier indicates system jitter, not DFS work)
  - Diff: **~60ms median = ~2.7% of cold-ingest wall**
- **verdict**: Below the 5% threshold the FU specified. Reviewer's 14% (Py) + 10% (TS) estimate was substantially overestimated, likely because (a) `.sample_repo`'s TS files don't average many `call_expression` nodes so the `Object.freeze` finder terminates fast on most files, and (b) Python's `nodes.iter().find()` is O(K·N) but K (classes per file) is small in typical Python and N is per-file, not corpus-wide. Cost of churn (HashMap pre-compute on Python side, frameworks.scm capture addition on TS side, 14-lang parity re-verification) far exceeds the 60ms total saving on a 22k-file corpus.
- **re-open criteria**: (i) a class-heavy Python corpus (≥30 classes/file × ≥1000 nodes/file) shows >5% regression, or (ii) a TS corpus with many `Object.freeze({...})` literals (config-heavy codebases) shows >5%.
- **size-actual**: S (spike: ~30 min wall for patch + 6 builds + 6 runs + restore + verify clean)
- **links**: PR #376 simplify reviewer report (efficiency review); `crates/ecp-analyzer/src/python/parser.rs:1348-1381`; `crates/ecp-analyzer/src/typescript/parser.rs:658`; PR #371 (precedent: merge calls + path-literals into single DFS, valid pattern when needed)

### FU-2026-05-23-035  ·  surfaced in PR #380 /simplify pre-merge review  ·  ✅ done in PR #408
- **owner**: unassigned → fu-035-036 chore session 2026-05-24
- **original-scope**: Go `collect_go_sql_tx_scopes` (`crates/ecp-analyzer/src/go/parser.rs`) had a 5-layer nested `if` pyramid for `call_expression → function selector_expression → field field_identifier → method_name` match. Ruby (`is_transaction_do_block_call`) + Dart (`is_tx_call`) had already extracted flat `is_*_call(node, source) -> bool` predicates with the same depth refactored to flat guard-clauses; Go was the inconsistent outlier (`-X theirs` cherry-pick artefact from PR #380).
- **resolution**: Extracted `is_db_begin_call(call, source) -> bool` using `let-else` guards (matches Dart's template — both Go and Dart dispatch on a `selector_expression`'s `field` child, so the structures align 1-to-1). Call site collapsed to `node.kind() == "call_expression" && is_db_begin_call(node, source)`. Predicate body character-for-character identical to original conditions, just flattened — behaviour unchanged. Net Go: +30/-22 LOC = +8 LOC (extra helper signature + 4-line doc-comment outweighed by 5-level → 0-level nesting elimination).
- **size-actual**: S (~50 LOC delta on one file; ~5 min wall after rebase)
- **links**: PR #408; `crates/ecp-analyzer/src/go/parser.rs::is_db_begin_call`; Ruby template `crates/ecp-analyzer/src/ruby/parser.rs::is_transaction_do_block_call`; Dart template `crates/ecp-analyzer/src/dart/parser.rs::is_tx_call`; PR #380 /simplify quality reviewer report

### FU-2026-05-23-036  ·  surfaced in PR #380 /simplify pre-merge review  ·  ✅ done in PR #408
- **owner**: unassigned → fu-035-036 chore session 2026-05-24
- **original-scope**: Swift `swift/parser.rs:68-96` carried 31 LOC of design-doc commentary (Core Data / GRDB / Realm / SQLite.swift audit findings + cost estimates + future-path notes). CLAUDE.md prefers WHY-only comments in source — audit decisions are project-policy documentation that belongs in `docs/language-matrix.md` (extended in PR #374 with per-lang schema emission table), not a hot parser file.
- **resolution**: Collapsed source comment to 3 lines pointing at the docs footnote. Full audit content relocated to a new `[^sw-tx]` footnote on the Swift TransactionScope cell of the schema-emission matrix in `docs/language-matrix.md`. Swift row now displays `—[^sw-tx]` so readers see immediately that the dash is annotated rather than unexplained. Footnote captures: (a) the 4-pattern audit (Core Data / GRDB / Realm / SQLite.swift), (b) the receiver-type-inference + import-tracking requirement that makes the detector cost-prohibitive, (c) the 120-150 LOC parser + 10-15 LOC query estimate, (d) the "zero scopes is correct outcome" framing. Net Swift: -27 LOC source; docs: +20 LOC.
- **size-actual**: S (1 source file truncation + 1 docs footnote append; ~5 min wall)
- **links**: PR #408; `crates/ecp-analyzer/src/swift/parser.rs:68-70` (3-line pointer); `docs/language-matrix.md` Swift TransactionScope footnote `[^sw-tx]`; FU-2026-05-23-009 (parent transaction-scope feature, ✅ done in PR #380); PR #374 (docs/language-matrix.md schema emission table source)

### FU-2026-05-23-039  ·  surfaced in PR #378 /simplify pre-merge review  ·  ✅ done in PR #409 (merged as e27622e7, 2026-05-23T19:34Z)
- **owner**: aa9f1819194c398c8 sub-agent (FU-039 implementer session 2026-05-24)
- **original-scope**: `tantivy_hits` returned `(hits, 0)` as truncate-total — LLM consumers couldn't distinguish exact-N hits from truncated-at-MULTI_CAP. Substring fallback path was correct; tantivy path was the audit gap. Violated CLAUDE.md "Never fabricate · honest 'no data' beats a guess" since BM25 callers got a phantom 0 for total.
- **resolution**: Extended `TantivyEngine::search` return type to `Option<(Vec<(f32, String)>, u64)>` via a `(Count, TopDocs)` fused tantivy collector (single index scan, no extra latency). Threaded the total through `tantivy_hits` → `bm25_hits_from_graph` → `compute_single` → `run_single`. Reused existing `bm25_pre_truncate_total` payload field so consumers see the same metadata shape on both paths. Also corrected the `run_single` truncation message from "substring fallback truncated" to "search truncated" (the old wording was wrong when the tantivy cap triggered). 2 unit tests cover below-cap (total == hits) and above-cap (total > hits) scenarios.
- **size-actual**: 3 files / +112 / -24
- **links**: PR #409 (squash e27622e7); MULTI_CAP truncate site at `crates/ecp-cli/src/commands/find.rs:687-720`; tantivy `Count` collector pattern in `crates/ecp-cli/src/search.rs` `TantivyEngine::search`

### FU-2026-05-23-004  ·  surfaced in PR fix/reindex-head-sha-drift  ·  ✅ done in PR #410 (merged as 12a4ca69, 2026-05-23T19:40Z); /simplify cleanups follow-up in `chore/simplify-fu004-followup`
- **owner**: ab3bf766abef5b9af sub-agent (FU-004 implementer session 2026-05-24); orchestrator session c7cba51f for /simplify cleanup
- **original-scope**: Out-of-band branch switch — IDE / GUI / external terminal `git checkout` bypasses the PostToolUse hook, so the next `ecp` read command blocks 1-2s on a synchronous cold rebuild at `main.rs:231`. Violated CLAUDE.md priority #1 (<30ms per-query latency).
- **resolution**: PR #410 added `attach_latest_sibling_sha` warm-attach fallback in `auto_ensure::ensure_fresh` Missing branch + new `EnsureFreshOutcome::WarmAttach { sibling_graph_path }` variant + `Engine::load_warm` with in-memory `is_stale_for_sha` flag (Option B from the schema-risk audit — plain Rust field on `Engine`, NOT rkyv-serialized, zero on-disk impact). Background rebuild fires via the existing `crate::background::spawn_bg` + `flock -n` pattern. 4 integration tests cover warm-attach pickup / no-sibling-falls-back-to-sync-build / stale-flag-propagation / after-rebuild-ensure-fresh-returns-ready.
- **simplify follow-up**: PR #410 auto-merged 6 minutes before /simplify finished. Review surfaced 5 cleanup items: (Q1+Q2) drop dead `is_stale_for_sha()` getter — field is already pub, getter was `#[allow(dead_code)]` with a misleading "private field" docstring; (Q4) `vec!["admin", ...]` → array literal in `spawn_background_rebuild` (no heap alloc, slice coercion); (E1) `OnceLock<Mutex<HashMap>>` cache for `attach_latest_sibling_sha` keyed by `worktree_root` (no-op for CLI, MCP-friendly); (E2) new `graph.bin.compatible_version` 4-byte sidecar written by orchestrator after publish_dir rename, read on warm-attach to skip the 10-50ms mmap + rkyv::access full validation. Rejected: Q3 (tuple→struct, violates surgical changes). Shipped as `chore/simplify-fu004-followup` against current main.
- **size-actual**: original 4 files / +445 / -15; cleanup 3 files / +72 / -14
- **links**: PR #410 (squash 12a4ca69); follow-up PR `chore/simplify-fu004-followup`; existing head_sha sidecar pattern at `auto_ensure::write_head_sha_sidecar_with_sha`; existing `commit_lookup::find_latest_by_mtime` reuse

### FU-2026-05-23-001  ·  surfaced in PR #334  ·  🚫 wontfix 2026-05-24
- **owner**: unassigned (never claimed)
- **original-scope**: Apply the `<Lang>CaptureIndices` pre-resolve template (PHP / Java / Kotlin / C# / Swift / Dart / Crystal / Python / TS / Rust got it in PR #334) to the remaining 16 parsers: cpp / c / go / ruby / solidity / move / hcl / cairo / verilog / lua / zig / svelte / astro / sql / bash / vue / dockerfile / markdown / yaml / vyper / javascript. The template caches `capture_index_for_name` lookups at provider startup so the parse hot loop uses u32 indices instead of string comparisons.
- **wontfix-reason**: The FU's own `why-deferred` line said `capture_index_for_name` is not the bottleneck on these parsers, and ~800 LOC of mechanical refactor across 16 parsers buys no measurable user-visible improvement. The remaining langs split into three groups, none of which justifies the work:
  - **Niche / low-traffic**: Solidity / Move / Cairo / Verilog / Vyper — LLM agents rarely query these.
  - **Non-code formats**: SQL / Bash / Markdown / YAML / Dockerfile — limited node-kind diversity in the first place, so capture lookups are O(small).
  - **SFC composite**: Vue / Svelte / Astro — embedded sub-grammar dominates wall time; capture indexing isn't where it'd help.
  - JavaScript was the only mainstream survivor; it's been fine on the slow-path lookup and not flagged in any profile since.
  Reverse if a future profile actually attributes >5% wall-clock to capture name resolution on any of these parsers.
- **size-spared**: ~800 LOC across 3-4 PRs that nobody wanted to review.
- **links**: PR #334 commits `4d3a6217` (CaptureIndices Kotlin/C#/Swift/Dart/Crystal), `f20a360e` (PHP CI-L #1); template at `crates/ecp-analyzer/src/php/parser.rs:142-260`

### FU-2026-05-23-037  ·  surfaced in PR #384  ·  🚫 wontfix 2026-05-24
- **owner**: unassigned (never claimed)
- **original-scope**: Restore risk-based queue priority that was stripped from `.mergify.yml` during the PR #384 workflow fix bundle. The correct Mergify API path was `queue_rules.priority_rules[].priority: high|low` nested inside each queue rule — 5 queues × 3 priority bands = duplicated config across 5 sites (no YAML anchor possible because Mergify rejects nested-list anchor splices). Without it, area queues were FIFO regardless of `ecp:risk-low`/`-high` labels.
- **wontfix-reason**: PR #401 (`refactor(ci): replace Mergify with GitHub-native auto-merge + auto-update`, merged 2026-05-23) removed `.mergify.yml` entirely. The "Auto-merge ready PRs" workflow now enables `gh pr merge --auto --squash` on PR open — observed live during the FU-039 / FU-004 / FU-416 round, both auto-enabled by `app/github-actions` bot within seconds of PR creation. There is no queue, no priority_rules, no Mergify config file. The entire premise of this FU is gone.
  Reverse only if a future PR reintroduces a Mergify-style queueing layer.
- **links**: PR #401 (Mergify removal); PR #384 (Mergify config validation, removed the priority lines that this FU was trying to restore); Mergify priority_rules docs at https://docs.mergify.com/configuration/file-format/#priority-rules (no longer relevant to this repo)

### FU-2026-05-23-002  ·  surfaced in PR #334  ·  ✅ done in working tree 2026-05-25
- **owner**: current session
- **original-scope**: step3a `parse_only` remained the largest cold-ingest residual. One concrete hotspot was `crates/ecp-analyzer/src/calls.rs::extract_calls`: every call site invoked `attach_to_enclosing`, which linearly scanned all Function/Method/Constructor nodes, making attachment O(C · K) per file.
- **resolution**: `extract_calls` now batches pending calls, builds a sorted list of enclosing containers once, and uses a sweep over call lines with an active min-heap keyed by container width. This preserves smallest-span containment and original call order while avoiding per-call full-node scans. The public `attach_to_enclosing` helper remains for receiver-type modules that still attach one call at a time.
- **verification**: `cargo test -p ecp-analyzer calls::tests`; `cargo check -p ecp-core -p ecp-analyzer --tests`; `cargo check -p egent-code-plexus --tests`; `git diff --check`.
- **links**: `crates/ecp-analyzer/src/calls.rs`; PR #334 per-provider profile; original open entry in `.claude/FOLLOWUPS.md`.

### FU-2026-05-24-003  ·  surfaced in FU-006 follow-up spike  ·  ✅ done in working tree 2026-05-25
- **owner**: current session
- **original-scope**: `crates/ecp-core/src/cypher/executor.rs::archived_fm_flag` used `function_metas.binary_search_by_key(...)` per node. Flag-filter Cypher queries therefore paid log(N) side-table lookup cost for each candidate node.
- **resolution**: Bumped graph schema to v11 and added dense `node_flags: Vec<u8>` to `ZeroCopyGraph`, indexed by `node_idx`. `GraphBuilder` populates the low-byte FunctionMeta flags during pass 1.8. `archived_fm_flag` now reads `node_flags[node_idx]` in O(1) for boolean flags, with the old binary-search path kept as fallback for empty synthetic fixtures / legacy-shaped data.
- **verification**: `cargo test -p ecp-core cypher::executor::tests::fm_`; `cargo test -p ecp-analyzer --test functionmeta_python python_async_function_has_async_flag`; `cargo test -p egent-code-plexus graph_version_history_includes_current_version`; `cargo check -p egent-code-plexus --tests`; `git diff --check`.
- **links**: `crates/ecp-core/src/graph.rs`; `crates/ecp-analyzer/src/resolution/builder.rs`; `crates/ecp-core/src/cypher/executor.rs`; FU-2026-05-23-006.

### FU-2026-05-23-025  ·  surfaced in PR #367 (feat/pathliteral-full)  ·  ✅ done in working tree 2026-05-25
- **owner**: current session
- **original-scope**: Auto-detect filename split-brain pairs among PathLiteral nodes instead of requiring humans / agents to manually guess candidate pairs such as `meta.json` ↔ `session_meta.json`.
- **resolution**: Added `ecp impact --literal-coherence`, which scans all PathLiteral nodes and emits conservative candidate pairs only when they share an extension, have similar basenames, appear in nearby source directories, and have separated access patterns (read-only vs write-only). The existing `ecp impact --literal <V>` path remains exact-match only.
- **remaining-split**: Review verdict integration is tracked separately as FU-2026-05-25-001; the automation gap for candidate-pair discovery is closed at the CLI layer.
- **verification**: `cargo test -p egent-code-plexus literal_coherence_tests`; `cargo check -p egent-code-plexus --tests`; `git diff --check`.
- **links**: `crates/ecp-cli/src/commands/impact.rs`; `docs/skills/ecp/SKILL.md`; `docs/skills/ecp/_shared/cli/impact.md`; FU-2026-05-25-001.

### FU-2026-05-23-006  ·  surfaced in PR #352  ·  ✅ done in PR #426 (merged as 2cbeee2c, 2026-05-24)
- **owner**: session a59bfc41-5e8c-4bd3-974e-a8a0215ab73b (sub-projects-1-5-spec worktree)
- **original-scope**: `archived_fm_decorators` 在 cypher WHERE eval 路徑上每 row 配一個全新的 `Vec<Value::Str>`。Cypher 引擎的 `Value` 型別目前只支援 owned variants（沒有 borrowed-slice / cow 版本），所以 `m.decorators` 屬性無法 zero-allocation 表達。Decorator-dense 的查詢（Spring `@Injectable` / Hilt DI / Django `@route`）若同時跑 `IN m.decorators` 過濾，per-row alloc 會出現在 WHERE eval hot path。
- **resolution**: PR #426 implemented a specialized predicate pushdown for `WHERE 'X' IN node.decorators`. Instead of materializing the `Vec<Value>` for the full list, the executor now uses a borrowed-variant aware check that directly iterates over the `rkyv` archived slice. This avoids the per-row `Vec` allocation entirely for the most common decorator filter pattern.
- **size-actual**: M (~150 LOC in executor and value evaluation logic)
- **links**: PR #426 (merged as 2cbeee2c); FU-2026-05-24-001 (sibling optimization in PR #426).

### FU-2026-05-23-007  ·  surfaced in promotion-readiness review (2026-05-23)  ·  ✅ done in PR #406 (merged as 13b53db7, 2026-05-24)
- **owner**: c7cba51f (orchestrator)
- **original-scope**: receipts 只 vs GitNexus（60× cold index）但 GitNexus 是 Node.js 不會被當競品；真正空間競品 codescope (SurrealDB)、coraline (SQLite) 沒同 corpus benchmark；無法支撐「我們比同類 Rust 競品更快」的廣宣 claim
- **resolution**: PR #406 landed the `scripts/benchmark/benchmark_vs_competitors.py` scaffold and an initial `ecp`-only baseline snapshot. The script is now ready to incorporate codescope and coraline numbers once those binaries are installed and verified. README's "Performance" section was updated to point to the new benchmark document.
- **size-actual**: M (~200 LOC python scaffold + docs updates)
- **links**: PR #406 (merged as 13b53db7); `docs/benchmark-vs-competitors.md`.

### FU-2026-05-23-026  ·  surfaced in PR #367 /simplify review  ·  ✅ done in PR #417 (merged as c42cb03a, 2026-05-24)
- **owner**: unassigned → orchestrator session
- **original-scope**: `python/path_literals.rs::strip_quotes` (PR #367) and `route_detector.rs::strip_string_quotes` overlap on Python prefix-quote handling (`r/b/u/f/rb/br/rf/fr` + single/double quotes). Deduping was deferred to avoid scope creep in #367.
- **resolution**: PR #417 unified the logic into `pub fn strip_python_string_quotes(raw: &str) -> Option<&str>` inside `crates/ecp-analyzer/src/framework_helpers.rs`. Both `path_literals.rs` and `route_detector.rs` were updated to use the shared helper. Triple-quote support was preserved and tested.
- **size-actual**: M (~60 LOC unified helper + tests)
- **links**: PR #417 (merged as c42cb03a).

### FU-2026-05-23-041  ·  surfaced in PR #379 /simplify review  ·  ✅ done in PR #403 (merged as 7474cc9f, 2026-05-24)
- **owner**: c7cba51f (this orchestrator session)
- **original-scope**: 5 個 site 共享同一 self-exe-subprocess pattern (`std::env::current_exe()?` + `Command::new(&self_exe).args(...).output()?`). Every command duplicated this boilerplate.
- **resolution**: PR #403 promoted the `self_exe()` helper and added `run_self(args: &[&str]) -> Result<Vec<u8>, EcpError>` to a shared module. Updated `pr_analyze`, `bindings::dump`, and `admin::claude_code` to use the unified helper.
- **size-actual**: S (~80 LOC refactor)
- **links**: PR #403 (merged as 7474cc9f).

### FU-2026-05-23-046  ·  surfaced during local main-cleanup  ·  ✅ done in PR #403 (merged as 7474cc9f, 2026-05-24)
- **owner**: c7cba51f (this orchestrator session)
- **original-scope**: `examples/dump_uid_collisions.rs` was a redundant ad-hoc tool superseded by the more complete `ecp dev uid-audit`.
- **resolution**: PR #403 deleted the orphan `examples/dump_uid_collisions.rs` (-143 LOC) as part of the subprocess helper cleanup bundle.
- **size-actual**: 0 LOC new work; orphan deletion.
- **links**: PR #403 (merged as 7474cc9f).

### FU-2026-05-24-006  ·  surfaced in `ecp processes` skill audit  ·  ✅ done in PR #431 (merged as a9566069)
- **owner**: unassigned
- **original-scope**: Two gaps in the main `ecp` skill — (a) `docs/skills/ecp/_shared/cli/processes.md` missing while every other subcommand had a `_shared/cli/` card; (b) Quick Reference's `processes` row didn't surface the `ecp processes trace <pattern>` invocation shape.
- **resolution**: PR #431 added the `processes.md` reference card and a `SKILL.md` row (`ecp processes trace <pat>` | Dump full Function/Method step sequence) — both sub-items closed.
- **size-actual**: S (~50 LOC across one new file + one row edit)
- **links**: PR #431 (merged as a9566069); `docs/skills/ecp/_shared/cli/processes.md`; `docs/skills/ecp/SKILL.md:75`.

### FU-2026-05-22-007  ·  surfaced in PR #345  ·  ✅ resolved 2026-05-25 — orthogonal third path shipped
- **owner**: another session (dispatch indirection roadmap)
- **original-scope**: Naming-decision conflict — another session's Phase 4 planned a top-level `ecp blind-spots --kind dispatch-...` command, vs this session's `ecp summary.blind_spots` being a section not a command.
- **resolution**: FU-001 shipped `ecp schema blindspots` (inventory-mode, no graph load) as a third orthogonal path; `ecp summary` keeps its per-repo `blind_spots.by_kind` section. The ecp side no longer occupies the `blind-spots` command name, so the conflict that deferred this FU is gone — the other session is free to claim `blind-spots` or adopt the recommended `ecp summary --filter blind_spots.kind=*` flag form. No further action owned by this side.
- **size-actual**: 0 LOC new work (resolved by FU-001's `schema blindspots` already landing).
- **links**: PR #345 description tail; `ecp schema blindspots`; feat/blindspot-rollout commit `be806258`.

### ✅ FU-2026-05-25-003 — resolved 2026-05-25 (NOT a code bug: stale index)
- **was**: 3738 nodes stored `src/`-relative (e.g. `src/admin/diagnostics.rs`) vs 14105 `crates/`-relative; suspected ecp-cli path-rooting bug, also blocked `factor_base_path` on `ecp inspect` (mixed-root → empty LCP).
- **root cause**: stale L2/overlay data from an older index build that used a crate-relative root. Incremental reindex (the auto-ensure path) only re-walks changed files, so the old mis-rooted nodes persisted across ordinary reindexes.
- **resolution**: `ecp admin index --repo /home/enor/code-graph-nexus --force` (drops L2 dir + per-file parse_cache, then rebuilds from the `~/.ecp/.../commits/branch_*.building/_src` snapshot). Result: `src/`-rooted nodes 3738 → 0; `crates/`-rooted 14105 → 21016. No code change. Current indexing (`admin/index.rs:110 strip_prefix(src_root_ref)` against the `_src` snapshot) is correct.
- **verified**: `ecp inspect parse_with_budget` now emits `base_path: crates/` (factoring kicked in once paths were consistent). Detection invariant for future: `MATCH (n) WHERE n.filePath STARTS WITH 'src/' RETURN count(*)` == 0.
- **lesson**: analyzer/path-convention changes need `--force` (the documented purpose); a plain reindex won't purge stale-rooted nodes.

### FU-2026-05-23-003  ·  surfaced in PR #334  ·  ✅ done in PR #447 — pass16 fetch-shape par_iter (A/B: 22ms→6ms wall, flat CPU, Fetches count identical)
- **owner**: unassigned
- **scope**: pass16_fetch_shape (0.075s = 4% wall) 與 update_repo_meta 之 dir_size walkdir 都是序列 — 前者 per-file scan 各語言獨立可平行；後者目前已用 `if let Ok(m) = e.metadata()` 容錯但仍是序列 walk
- **why-deferred**: pass16 par_iter 與 CI-H 模板同形但 ROI 小；dir_size 即使平行化也只省 ms 級別（advisory stats）
- **next-action**: 若未來總時長要進一步壓到 1.5s 以下再啟動；目前 1.87s median 已達 -40% 目標
- **size**: S each
- **links**: PR #334 CI-M-followup commit `fix(build): dir_size tolerant of tantivy background race`；`crates/ecp-analyzer/src/resolution/builder.rs:827-882`

<!-- FU-2026-05-23-004 → ✅ done in PR #410 (squash 12a4ca69, 2026-05-23); /simplify cleanups follow-up in `chore/simplify-fu004-followup` -->

### FU-2026-05-24-002  ·  surfaced in PR #430 + PR #432 perf measurement  ·  ✅ done in PR #447 — +4 bench cypher patterns (COLLECT / IN-list / multi-hop / GROUP BY, all >60ms)
- **owner**: unassigned
- **scope**: `benchmark_ecp.py` only covers `count(*) ungrouped` and `decorator IN` for cypher hot patterns. Future cypher perf work (e.g., `COLLECT()`, `WHERE name IN [literal,...]` literal-list, multi-hop edge `(a)-[:R*1..3]->(b)`, multi-aggregate GROUP BY) will land without bench coverage and the regression detector won't catch it. The realigned bench (#430) shipped only the two patterns that PRs #422 + #426 exercised.
- **why-deferred**: each new query needs verification that it exercises a distinct hot path; bundling more than 2 into PR #430 would have diluted its scope (canonical CLI realign vs bench coverage expansion).
- **next-action**: extend `benchmark_ecp.py` queries[] with `COLLECT()` projection, `IN [literal,...]` filter, `(a)-[:Calls*1..3]->(b)` multi-hop, `GROUP BY n.kind, count(*), sum(...)` multi-aggregate. Verify each shows non-trivial cost (>5 ms) on `.sample_repo`. Optionally add a regression-detection mode comparing to a committed baseline JSON checkpoint.
- **size**: S (~30 LOC bench script + optional baseline JSON capture)
- **links**: PR #430 (realign); PR #432 (kind-CSR + walk_rel); PR #433 (Binding SmallVec); FU-2026-05-23-006

<!-- FU-2026-05-24-004 → ✅ done in PR #447 — Value::write_dedup_key structural key (A/B: ~12% on DISTINCT, no regression on control) (see FOLLOWUPS_DONE.md) -->

### FU-2026-05-24-004  ·  surfaced in FU-006 follow-up spike  ·  ✅ done in PR #447 — Value::write_dedup_key structural key (A/B: ~12% on DISTINCT, no regression on control)
- **owner**: unassigned
- **scope**: `dedup_rows()` (cypher executor, ~line 253) uses `format!("{row:?}")` as the dedup key for DISTINCT. O(n) string formatting per row in a path that's only meant to filter duplicates. Not in the spike's measured hot path (no DISTINCT queries), but flagged as latent — any `RETURN DISTINCT ...` query pays it.
- **why-deferred**: low-priority — DISTINCT queries are infrequent in agent-fired cypher. Optimizing without a real workload showing impact would be guess-driven.
- **next-action**: replace `format!("{row:?}")` with either (a) `xxh3_64` over the row's value_key serialization, or (b) a structural key `Vec<u8>` built by a new `Value::write_dedup_key(&mut buf)` method. Add a bench query exercising DISTINCT to validate before/after.
- **size**: S (~30 LOC including the new write_dedup_key trait method)
- **links**: FU-006 follow-up spike "code-level findings" section; `crates/ecp-core/src/cypher/executor.rs::dedup_rows`

<!-- FU-2026-05-24-005 → ✅ done in PR #447 — reinterpreted as FU-004 before/after A/B in docs/perf-notes.md; historical stacked-delta not reproducible under parallel-session load, explicitly not used (see FOLLOWUPS_DONE.md) -->

### FU-2026-05-24-005  ·  post-merge validation for PRs #432 + #433  ·  ✅ done in PR #447 — reinterpreted as FU-004 before/after A/B in docs/perf-notes.md; historical stacked-delta not reproducible under parallel-session load, explicitly not used
- **owner**: unassigned (this session's orchestrator if still around; else the next FU-006 toucher)
- **scope**: H1+H3 (PR #432) measured count(*) -47% and H2 (PR #433) measured count(*) -42% **independently** against main. The two attack orthogonal axes — iteration count (CSR shortcut + walk_rel closure) vs per-iteration clone cost (Binding SmallVec) — so combined should compound to ~-67% on count(*) and ~-33% on decorator IN. **Predicted but not empirically validated**: bench could not run both simultaneously without merging one first.
- **why-deferred**: blocked on PRs #432 + #433 both landing in main. Validation is a ~10 min `benchmark_ecp.py --runs 10` on the post-merge main binary vs a pre-#432 baseline binary held aside.
- **next-action**: after both PRs merge, run `benchmark_ecp.py` with the resulting main binary against the `skill-quick-reference` worktree binary (which is pre-#432 baseline; still available unless cleaned up). Quote the stacked deltas as an addendum on PR #433 description or in a `docs/perf-notes.md`. If stacking is NOT linear (one PR's win cancels another's) file a sub-FU explaining the interference.
- **size**: S (re-run bench + 1-paragraph report)
- **links**: PR #432; PR #433; FU-2026-05-23-006

<!-- FU-2026-05-24-006 → ✅ done in PR #431 (merged as a9566069) — processes.md card added + SKILL.md trace row; both sub-items closed (see FOLLOWUPS_DONE.md) -->

<!-- FU-2026-05-24-007 → ✅ done in PR #447 — pinned tail-lang clones in bootstrap_sample_repos.sh + release-binary baselines.md recapture (see FOLLOWUPS_DONE.md) -->

### FU-2026-05-24-007  ·  surfaced in this session's `.sample_repo` baseline verification  ·  ✅ done in PR #447 — pinned tail-lang clones in bootstrap_sample_repos.sh + release-binary baselines.md recapture
- **owner**: unassigned
- **scope**: `scripts/parity/baselines.md` is a 2026-05-14 dev-binary snapshot of wave-1 lang cold-index timings. The fixture composition drifted (lua 15→11 files, solidity 727→403, move 486→367, zig 1→31 with Zig provider now shipped) because `scripts/parity/benchmark_repos.py:31` does `git clone --depth 1` against upstream master — so the docs' numeric comparisons are no longer 1:1 reproducible. Numbers are also dev-profile; current main runs release ~3× faster anyway, compounding the drift.
- **why-deferred**: re-capturing without stabilising the fixture means the next regen of `.sample_repo` will redrift in days. Either pin clone commits or accept the docs as historical/directional only.
- **next-action**: pick one direction:
  (a) **Pin**: change `benchmark_repos.py` to clone at specific commits per wave-1 lang (`--depth 1 --branch <sha>`), then recapture baselines.md against that pinned snapshot using release binary;
  (b) **Sunset**: add a header banner to baselines.md marking it historical, point at a fresh living doc (e.g. `docs/perf-snapshots.md` regen'd at release-tag time via CI).
- **size**: S (either path; (a) requires picking commits, (b) requires re-thinking which numbers matter)
- **links**: `scripts/parity/baselines.md`; `scripts/parity/benchmark_repos.py:31`

<!-- FU-2026-05-24-008 → ✅ done in PR #447 — benchmark_ecp.py --analyze-runs N (default 1, --runs N>1 auto-implies) (see FOLLOWUPS_DONE.md) -->

### FU-2026-05-24-008  ·  surfaced in this session's `benchmark_ecp.py` runs  ·  ✅ done in PR #447 — benchmark_ecp.py --analyze-runs N (default 1, --runs N>1 auto-implies)
- **owner**: unassigned
- **scope**: `benchmark_ecp.py` runs `analyze (baseline)` and `analyze (incremental)` with `runs=1` (single sample). Wall-time variance ±20% is normal at this scale, so the bench can't distinguish a 20% indexing regression from machine drift. All other phases use `--runs N` (default 3, configurable); analyze is hardcoded `runs=1`.
- **why-deferred**: changing analyze to N runs means N cold/incremental cycles per bench invocation, doubling-to-tripling total wall time. Worth the cost for CI regression detection but not always for quick manual runs.
- **next-action**: split analyze into `--analyze-runs N` (default still 1 for compat, but `--runs 10` users get `--analyze-runs 10` auto-implied). Document the single-sample noise floor in the bench script's docstring so users interpret `analyze (baseline) ±20%` correctly.
- **size**: S (~10 LOC bench changes)
- **links**: `scripts/benchmark/benchmark_ecp.py`; PR #430 (most recent bench realign)


### ✅ FU-2026-05-25-004 — done in PR #448 (1-based line migration)
- **was**: `line` reported 0-based (raw tree-sitter `span.0`) across impact/find/inspect/routes/processes/rename/diff, while find.rs already +1 at one site (silent inconsistency). cypher had no `line` property at all (`RETURN n.line` → null).
- **decision**: MIGRATE (not document). Rationale surfaced when asked "+1 or -1 for an LLM": 0-based is actively wrong for the LLM consumer — editors/`grep -n`/compilers/stack-traces/citations are all 1-based, so the model reads "line 27" as 27 and won't apply a 0-based correction mid-task. Priority #2 (reduce hallucination), not cosmetics.
- **fix**: single conversion boundary `ArchivedNode::start_line()` (`span.0+1`) / `end_line()` (`span.2+1`) in graph.rs; span stays 0-based internally (range/hash/containment). ~30 CLI display sites rerouted; find.rs manual `+1` folded in. cypher `line`/`startLine`/`endLine` added (same 1-based). **Read-side conversion → no reindex.**
- **verified**: parse_with_budget (grep -n line 28) → find/impact report 28 (was 27), cypher `n.line,n.startLine,n.endLine` → 28,28,44 (was null). ecp-core 145 + cypher 111 (new test) + full CLI green, clippy clean.
- **not 14-lang**: output/query-layer change, not a parser/grammar/graph-construction change — analyzer parsers untouched (they still produce 0-based spans, correctly).
- **left as-is**: `session/overlay_writer.rs:336-337` already emits 1-based via inline `span.0+1` on a RawNode flow (separate type, pre-existing, correct).

### FU-2026-05-25-001  ·  split from FU-2026-05-23-025  ·  ✅ done in PR #449 (2026-05-25)
- **scope**: Wired `ecp impact --literal-coherence` into `ecp review` so PR review auto-flags PR #357-class filename split-brain (writer emits `session_meta.json`, reader opens `meta.json`).
- **resolution**: Added `literal_coherence` constituent to review *aggregate* mode (not `--verdicts`, which lacks the Engine/graph the primitive needs). `build_literal_coherence_payload` promoted to `pub`; `run_literal_coherence` scans the live graph snapshot (not diff-driven, so a pair fires when only one side changed); `literal_coherence_findings` (pure fn) maps each candidate → one `Severity::Warn` finding (primitive confidence is always "high") attributed to the writer site, gated by either-end-in-scope; `Source::LiteralCoherence` added.
- **tests**: 4 pure-fn tests — session_meta.json/meta.json fixture (literals from candidate payload, not hand-asserted), reader-only-in-scope still fires + writer-attributed, out-of-scope skipped, empty candidates → nothing. 18 passed / 0 failed in `commands::review::aggregate`.
- **links**: PR #449; `crates/ecp-cli/src/commands/review/aggregate.rs`; `crates/ecp-cli/src/commands/review/findings.rs`; `docs/skills/ecp/_shared/cli/review.md`; parent FU-2026-05-23-025 above.

### ✅ done in PR #450 · FU-2026-05-25-005 · surfaced in FU-003 root-cause + PR #448 session
- **scope**: Incremental / auto-ensure reindex never purged nodes left over from an OLD path/analyzer convention (FU-003's 3738 `src/`-rooted nodes survived ordinary reindexes; only `--force` cleared them). Latent correctness trap for every consuming LLM.
- **resolution**: `ensure_index` now reads a `graph.bin.builder_fingerprint` sidecar (falls back to commit meta.json) BEFORE `git_fingerprint_shortcut`. On mismatch ⇒ `Stale { needs_full_rebuild: true }` ⇒ `ensure_fresh` does a full `build_l2` (drops graph.bin), not an L1 overlay. Missing fingerprint defers to mtime walk (no spurious rebuild). `BUILDER_FINGERPRINT` moves every release so it catches drift `GRAPH_FORMAT_VERSION` can't.
- **links**: `crates/ecp-cli/src/auto_ensure.rs` (gate + sidecar + `fingerprint_drifted`); `crates/ecp-cli/src/build/orchestrator.rs` (sidecar write/back-fill).

### ✅ done in PR #450 · FU-2026-05-25-006 · surfaced in FU-003 --force rebuild output
- **scope**: `--force` only warned about MetaUnreadable L1 sessions, never reaped them; they accumulate in `~/.ecp` unbounded across binary upgrades.
- **resolution**: MetaUnreadable confirmed unrecoverable (no base_sha ⇒ never serveable/rebuildable). Reaped on sight in `invalidate_matching_l1` — rename to `<sid>.dead` + delayed rm, no sha-hint gate. New `InvalidateReport.meta_reaped` surfaced as `l1_meta_reaped=N` in the `l2.rebuilt` line.
- **links**: `crates/ecp-cli/src/build/force.rs`; `crates/ecp-cli/src/commands/admin/index.rs`.

### ✅ done in PR #450 (🚫 not-reproducible) · FU-2026-05-25-007 · surfaced running ecp from a worktree cwd
- **scope**: Claimed `ecp` from a worktree cwd resolves a distinct repo identity with no index → silent empty results.
- **resolution**: NOT REPRODUCIBLE on the current binary. Across 4 escalating scenarios (indexed HEAD / fresh SHA + sibling / fresh SHA + empty commits / `.claude/worktrees/<x>` with source), `repo_dir_name_for_cwd`'s git-common-dir collapse points the worktree at the parent's `~/.ecp/<repo>/`; warm-attach prints a `note:` line; empty `commits/` falls to `build_l2(worktree_root)` which builds correctly. The FU described pre-fix behaviour already covered by the `git_cache` absolute-path handling + warm-attach mechanism. Only residual is `graph_path.rs:20`'s literal `.ecp/graph.bin` fallback, which is fully recovered downstream (no error behaviour) — left as-is.
- **links**: `crates/ecp-cli/src/repo_identity.rs`; `crates/ecp-cli/src/graph_path.rs`; `crates/ecp-cli/src/auto_ensure.rs` (warm-attach).

### ✅ done in PR #454 (merged) · FU-2026-05-26-001 · surfaced during CompensatedBy (FU-008) dev — host freeze diagnosis
- **scope**: `~/.ecp` graph cache leaked to 16G (13G zombie). Three cleanup layers all missed: **L2 (main cause)** same-SHA `.gen.<ts>` generation dirs never converged (one SHA accumulated 25× 63MB graphs, no cleaner at all); **L1** `fs_safe::retire_dir_async`'s detached delete thread died with the short-lived CLI process leaving `<repo>.dead.*` dirs, failure swallowed by `let _ =`; **L3** `gc::sweep_sessions` existed but `admin gc` subcommand was never wired (`gc.rs:6` "isn't wired yet"). `prune --orphans` only sweeps orphan repos (common_dir gone), never touches live-repo retired/duplicate generations — a semantic gap that hid the leak.
- **resolution**: `sweep_stale_generations` (per-SHA, keep greatest `Generation` via reused `CommitDirName::parse` + Ord; skip `.building`-active SHA + fresh<10s); `sweep_retired_repos` (remove top-level `.dead.*`); `retire_dir_async` logs background-delete failures instead of swallowing; wired `ecp admin gc` subcommand (+`--dry-run`); `session_start` runs `prune` then `gc` under ONE flock (gc best-effort `|| true`, doesn't flip prune marker). Also fixed a real bug found mid-impl: the `.building` guard used `Path::with_extension` (replaces last dot-segment) but real markers are `<dirname>.building` (append, SHA-keyed) per orchestrator.rs — guard was dead, would delete dirs mid-build.
- **links**: `crates/ecp-cli/src/admin/gc.rs`; `crates/ecp-cli/src/commands/admin/gc.rs`; `crates/ecp-cli/src/commands/hook/session_start.rs`; `crates/ecp-cli/src/background.rs`; `crates/ecp-core/src/registry/fs_safe.rs`; spec+plan `docs/superpowers/{specs,plans}/2026-05-26-ecp-dead-graph-gc*`.

### ✅ fixed in PR #457 · FU-2026-05-25-008 · surfaced in PR #453 discussion
- **scope**: Promote Saga compensate/undo/rollback name-pairs from the on-the-fly `find-transaction-patterns` scan into a real heuristic `RelType::CompensatedBy` graph edge (new `post_process/saga_pairs.rs` pass, index-time emission). Resolves verb-sprawl + main-path exposure + hot-path cost.
- **resolution**: `source=compensator, target=operation`; reason encodes evidence (`saga:calls-back` 0.8 / `saga:name-only` 0.6). `is_heuristic()==true` → shown in impact `heuristic_callers` tagged `requires_verification`, `--no-heuristic` suppresses. 14-lang case handling (snake/camel/Pascal). `find-transaction-patterns` Saga half retired → reads the edge (15 CLI tests green, schema unchanged). Corpus-verified on isolated /tmp fixture (impact charge surfaces name-only compensator). Mid-impl fixes: format::rel_to_str exhaustive-match wiring (was a 6th missed dispatch site), schema reltypes inventory (21 variants) + heuristic-set test. Stacked on PR #453.
- **links**: `crates/ecp-analyzer/src/post_process/saga_pairs.rs`; `crates/ecp-core/src/graph.rs` (enum); `crates/ecp-cli/src/commands/{find_tx_patterns,format,schema}.rs`; spec+plan `docs/superpowers/{specs,plans}/2026-05-25-compensatedby-reltype-promotion*`.

### ✅ fixed in PR #457 · FU-2026-05-25-009 · surfaced in PR #453 discussion
- **scope**: Stale doc comments claimed Saga Outbox detection "deferred pending T5-33" (already landed); misleads readers.
- **resolution**: Rewrote `find_tx_patterns.rs` module doc (Saga half now reads CompensatedBy edge; Outbox is query-time scan) + skill doc `~/.claude/skills/ecp/_shared/cli/find-transaction-patterns.md` (removed T5-33 deferral, documents graph-backing). Bundled into FU-008's PR #457.
- **links**: `crates/ecp-cli/src/commands/find_tx_patterns.rs` module doc; skill doc.

### ✅ done · FU-2026-05-25-002 · surfaced in anonymous-callback-call-edges PR #443
- **scope**: Anonymous nodes stored as `<anonymous:line:col>` (position needed for uid-distinctness); verbose output name + in-row dual line-number confusion (name 1-based vs impact `line` column 0-based).
- **resolution**: BOTH halves resolved across two PRs. (1) anon-display: commit 87a144c1 (PR #446 `perf/llm-path-prefix-factor`) — `compress_for_llm` truncates `<anonymous:line:col>` → bare `<anonymous>` in Llm/toon default; json keeps full name as uid-distinct identifier. (2) line-base: FU-2026-05-25-004 / PR #448 — toolchain-wide 0→1 line migration via single read-side conversion boundary `ArchivedNode::start_line()`/`end_line()`. Verified 2026-05-26: cypher on a Python lambda fixture shows `<anonymous>` (Llm) vs `<anonymous:2:29>` (json). Dual-number confusion eliminated.
- **links**: PR #443 (origin), PR #446 (display), PR #448 (line-base via FU-004); `crates/ecp-cli/src/output.rs:232` is_anonymous_with_position.

### ✅ done in PR #499 · FU-2026-05-26-002 · surfaced in field-read-edge / ReadsField PR
- **scope**: JS class fields were not modeled as Property nodes (queries.scm had no property capture, unlike TS public_field_definition), so `obj.field` reads had no Property target and the ReadsField edge could not resolve for JS — pinned as a negative test. LLM node-coverage gap (filter B): JS field refactors fell back to grep. Was blocked by PR #455 (ReadsField), which merged 2026-05-25.
- **resolution**: Added `(field_definition property: (property_identifier)) @property` to javascript/queries.scm (class-body-only; `this.x=` constructor assignments are assignment_expression, never matched); mapped `property.name` → NodeKind::Property in spec.rs; routed the `@property` span through parser.rs root_span dispatch (owner_class filled by the existing stamp_owner_class_by_span pass). Flipped `javascript_field_read_has_no_property_target` → positive `javascript_reads_field` + added `javascript_constructor_assignment_is_not_a_property` guard. Verified: cargo test -p ecp-analyzer 2331 passed, clippy clean, real-graph cypher (Property{timeout,retries} ownerClass=Config + ReadsField readTimeout→timeout/readRetries→retries), perf A/B +0.96% parse (within noise) for +480 Property nodes, 3-agent adversarial review (zero false positives, parity baseline delta=0). Inherited TS-parity gaps (private/computed/string-keyed fields, multi-class same-file resolution heuristic, anon-class uid collision) tracked as FU-2026-05-29-002.
- **links**: PR #499; TS sibling crates/ecp-analyzer/src/typescript/queries.scm:208; FU-2026-05-29-002 (inherited gaps)

# Follow-ups · Open

<!-- Protocol: see `.claude/CLAUDE.md` "Follow-ups protocol" section.
     Resolved / wontfix entries: see `.claude/FOLLOWUPS_DONE.md`.
     This file is gitignored; the canonical path is the absolute
     `/home/enor/code-graph-nexus/.claude/FOLLOWUPS.md` regardless of
     which worktree the agent is editing from. -->

## Resolved → see FOLLOWUPS_DONE.md
<!-- One-line stubs for entries archived elsewhere. Sorted by FU id. -->

<!-- FU-2026-05-22-001 → ✅ done in PR #370 (merged as dbb71278) -->
<!-- FU-2026-05-22-002 → ✅ done in PR #350 -->
<!-- FU-2026-05-22-003 → ✅ done in PR #372 — coverage alias removal -->
<!-- FU-2026-05-22-004 → ✅ done in PR #372 — verify-resolver alias removal -->
<!-- FU-2026-05-22-005 → ✅ done in PR #345 -->
<!-- FU-2026-05-22-006 → ✅ done in PR #334 -->
<!-- FU-2026-05-23-005 → ✅ done in PR #372 — summary binary_commit_sha warn -->
<!-- FU-2026-05-23-009 → ✅ done in PR #380 — 6-lang TransactionScope (TS / Rust / Go / Ruby / Dart + Swift audit) -->
<!-- FU-2026-05-23-010 → superseded by FU-2026-05-23-009 -->
<!-- FU-2026-05-23-011 → ✅ done in PR #376 — 5-lang EnumVariant + BlindSpot rollout -->
<!-- FU-2026-05-23-012 → ✅ done 2026-05-25 feat/decorates-niche-langs — JS @decorator + Go symbol pragma + C/C++ [[attr]]/__attribute__; //go:build wontfix (file-scope); Ruby wontfix (no annotation system) -->
<!-- FU-2026-05-23-013 → ✅ done in PR #374 -->
<!-- FU-2026-05-23-014 → ✅ done in PR #376 -->
<!-- FU-2026-05-23-015 → ✅ done in PR #377 (deferred: M1 --telemetry-path / M2 hours clamping / M3 streaming aggregator) -->
<!-- FU-2026-05-23-016 → ✅ done in PR #372 — Namespace/Module container -->
<!-- FU-2026-05-23-017 → ✅ done in PR #372 — Kotlin interface NodeKind -->
<!-- FU-2026-05-23-018 → ✅ done in PR #380 — consolidated into FU-009 -->
<!-- FU-2026-05-23-019 → ✅ done in PR #372 — FETCH_WITH_METHOD regex fix -->
<!-- FU-2026-05-23-020 → ✅ done in PR #374 — hand-rolled ecp_schema MCP wrapper -->
<!-- FU-2026-05-23-022 → 🚫 wontfix: conflicts with ~/.claude/CLAUDE.md "NEVER delete .claude/worktrees/" -->
<!-- FU-2026-05-23-027 → ✅ done in PR #374 — language-matrix.md Schema emission coverage -->
<!-- FU-2026-05-23-028 → ✅ done in feat/blindspot-rollout commit 7c13046c -->
<!-- FU-2026-05-23-029 → ✅ done in feat/blindspot-rollout commit cb897c15 -->
<!-- FU-2026-05-23-030 → ✅ done in PR #373 (merged as 67a92fe1) — parity_gate_smoke 32s → 0.8s via cached pipeline -->
<!-- FU-2026-05-23-023 → ✅ done in PR #385 (merged as ffd1643f) — PathLiteral chained-call sink promotion, 5 langs (Kotlin/Python/Swift/Rust/C++) -->
<!-- FU-2026-05-23-024 → ✅ done in PR #385 (merged as ffd1643f) — PathLiteral sink-override for ext-change callees, 14 langs -->
<!-- FU-2026-05-23-037 → ✅ done in PR #385 (test-fixture-helpers migration, merged as ffd1643f); the FU-037 below (Mergify config validation, surfaced in PR #384) is a separate duplicate-numbered Open item, NOT yet done -->
<!-- FU-2026-05-23-038 → ✅ done in PR #385 (merged as ffd1643f) — shape_check render_text typed counters -->
<!-- FU-2026-05-23-040 → ✅ done in PR #385 (merged as ffd1643f) — cypher executor row-width invariant + debug_assert -->
<!-- FU-2026-05-23-043 → ✅ done in PR #385 (merged as ffd1643f) — pr-analyze 3 minor cleanups -->
<!-- FU-2026-05-23-008 → ✅ done in PR #395 (merged as f763d222) — partial: a+b shipped, c → FU-2026-05-23-048 -->
<!-- FU-2026-05-23-031 → ✅ done in PR #395 (merged as f763d222) — `ecp admin claude install skills ecp` CLI path -->
<!-- FU-2026-05-23-034 → ✅ done in PR #394 (merged as 6fab1368) — enclosing_fn_idx_by_span helper -->
<!-- FU-2026-05-23-044 → ✅ done in PR #394 (merged as 6fab1368) — ImpactJson.changed_paths envelope -->
<!-- FU-2026-05-23-032 → 🚫 wontfix 2026-05-24 (MEASURED 2.7% < 5% threshold; see FOLLOWUPS_DONE.md) -->
<!-- FU-2026-05-23-033 → ✅ done in PR #405 (merged as 05bef21f) — mechanical parser cleanup bundle -->
<!-- FU-2026-05-23-035 → ✅ done in PR #408 — Go is_db_begin_call guard-clauses -->
<!-- FU-2026-05-23-036 → ✅ done in PR #408 — Swift audit comment → language-matrix.md footnote -->
<!-- routes.rs 同款 6 sites 沒修；建議下次 routes 動的 PR 套上 has_owning_file() helper -->

## Parser & Schema

<!-- FU-2026-05-23-001 → 🚫 wontfix 2026-05-24: code-consistency only, ~800 LOC for no measurable user benefit on tail langs (Solidity / Move / Cairo / Verilog / SQL / Bash / Markdown / etc.); profile already cleared capture_index_for_name as non-bottleneck. See FOLLOWUPS_DONE.md -->
<!-- FU-2026-05-23-002 → ✅ done in working tree 2026-05-25 — extract_calls batched enclosing-container attach -->

<!-- FU-2026-05-23-012 → done 2026-05-25 — see FOLLOWUPS_DONE.md (split into JS babel @decorator + Go symbol-pragma + C/C++ [[attr]] / __attribute__; //go:build dropped as wontfix per file-level-not-symbol semantic; Ruby remains documented wontfix — no annotation system) -->


<!-- FU-2026-05-23-022 → 🚫 wontfix: conflicts with ~/.claude/CLAUDE.md "NEVER delete .claude/worktrees/" rule (see FOLLOWUPS_DONE.md) -->

<!-- FU-2026-05-23-026 → ✅ done in PR #417 (merged as c42cb03a) — strip_python_string_quotes lifted to framework_helpers + route_detector shim (see FOLLOWUPS_DONE.md) -->

<!-- FU-2026-05-23-027 → ✅ done in PR #374 (merged as c094f9ee, see FOLLOWUPS_DONE.md) — language-matrix.md Schema emission coverage section -->

<!-- FU-2026-05-23-028 → ✅ done in feat/blindspot-rollout commit 7c13046c (see FOLLOWUPS_DONE.md) -->

<!-- FU-2026-05-23-029 → ✅ done in feat/blindspot-rollout commit cb897c15 (see FOLLOWUPS_DONE.md) -->
<!-- routes.rs 同款 6 sites 沒修；建議下次 routes 動的 PR 套上 has_owning_file() helper -->

<!-- FU-2026-05-23-035 → ✅ done in PR #408 (see FOLLOWUPS_DONE.md) -->

### FU-2026-05-29-001  ·  surfaced in symbol-parse completeness audit (worktree feat/symbol-parse-gaps, PR #498)
- **owner**: unassigned
- **scope**: A 14-mainstream-lang parser audit (grammar `node-types.json` × `queries.scm` × `parser.rs` emit, cross-checked vs `.sample_repo` NodeKind counts) found symbol-extraction gaps beyond the two fixed in PR #498 (C `enumerator`→EnumVariant, Go defined-types/`type_alias`→Typedef). Remaining confirmed gaps + kind-accuracy bugs, each single-language-isolatable with a sibling template:
  - **Java** `record_declaration`: record type node never emitted (find/impact on record name is dead). Add `record_declaration → @class.name` + header `formal_parameters → @property.name`. filter A/B. size S.
  - **PHP** in-class trait `use_declaration`: traits ARE captured (Trait=203) but the class→trait composition edge is never recorded (`@heritage` only from extends/implements). `ecp impact --target <trait>` misses every using class. filter A. size S.
  - **C#** `operator/event/indexer/destructor_declaration`: emit no node at all, so calls *inside their bodies* drop their edges (`attach_to_enclosing` only attaches to Function/Method/Constructor) — corrupts impact's caller set. filter A. size M (one capture per member kind → Method/Property). Event also needs a node for who-subscribes/who-raises.
  - **Rust** `extern "C" { fn … }` (`function_signature_item` inside `foreign_mod_item`): FFI-declared callables invisible; captured only inside `trait_item` today. filter A/B. size S.
  - **Swift** `protocol_property_declaration`: parser added `protocol_function_declaration` capture but never the parallel property-requirement form, so a Trait's property contract is invisible while its method contract is complete. filter B. size S.
  - **Kotlin** `type_alias`: grammar node uncaptured, corpus Typedef=0; existing `kotlin_gaps_audit.rs::type_alias_not_emitted` pins it but with NO rationale comment (behavior-snapshot, not a validated design drop). Add `(type_alias (type_identifier) @typedef.name)`, flip the test. filter B. size S.
  - **Kind-accuracy bugs (emitted but wrong kind/semantics)**: Python `X: TypeAlias = …` → Variable (should be Typedef); C++ `constexpr/const` globals → Variable (should be Const, like every sibling); Ruby `class << self` methods → instance Method (should be class-level, like `def self.foo`); Kotlin companion-object methods → Function (should be Method — `is_class_method` parent-chain check misses `companion_object`).
- **why-deferred**: out-of-scope — PR #498 scoped to the two highest-confidence corpus-proven gaps (C/Go). Each item above is a separable per-language change with its own tests + risk surface; bundling all 10 would violate surgical-change discipline. JS `field_definition→Property` is NOT in this list — already in flight in worktree `feat/js-property-capture` (see [[FU-2026-05-26-002]]).
- **next-action**: pick highest-value first (Java record / PHP trait edge / C# body-call-drop are filter-A graph-completeness; the rest are filter-B node-coverage or kind-accuracy). One PR per language, TDD with the named sibling template. The 14-lang parser-change rule applies only if a fix touches shared primitives — pure per-lang queries.scm additions mapping to existing NodeKinds do not.
- **size**: L (aggregate; each sub-item S–M)
- **links**: PR #498 (the C/Go fix + full audit context in its description); audit cache `/tmp/ecp_audit_cache/` (per-lang NodeKind matrix + distilled grammar node types); [[FU-2026-05-26-002]] (JS, separate); memory project_ultrawork_audit_2026_05_29

## Build / Engine / CI

<!-- FU-2026-05-23-003 → ✅ done in PR #447 — pass16 fetch-shape par_iter (A/B: 22ms→6ms wall, flat CPU, Fetches count identical) (see FOLLOWUPS_DONE.md) -->

### FU-2026-05-23-021  ·  surfaced in PR #366 (ci tier split)
- **owner**: unassigned
- **scope**: 若 push:main 的 test wall-clock 又變難受（目前 3 個 OS × ~870s 序列），下一個 ROI 是 `cargo nextest archive` build-once / test-N pattern：一個 `build` job 編 + `nextest archive` 出 `.tar.zst`，N 個 `test` job download artifact + `nextest run --archive-file`。剩餘 test 時間照常並行；省的是 N-1 次重編 workspace 的 cost（14 個 tree-sitter parser 編譯很重）。**不要回頭加 per-platform shard expansion** — 已在 #366 commit message 釘住，因為 free-tier concurrency 容易被多 worktree 並行打到限速。
- **why-deferred**: 現況 push:main 跑 3 job 在容忍範圍，且 PR-time 已 skip 整個 test；尚未踩到 wall-clock 痛點。先觀察 #366 merged 後 N 個 push:main 的 wall 中位數再決定
- **next-action**: (1) Spike 一個 PR `perf/ci-nextest-archive` 試水 — 拆出 build job + 3 個 test job（ubuntu/macos/windows 各 download archive 跑）；(2) 量測 vs 現況 wall median；(3) 若 archive 傳輸 + download overhead 超過省的編譯時間，放棄、保持現狀
- **size**: M（~80 LOC workflow 改 + per-OS artifact key 設計 + 量測）
- **links**: PR #366 commit `436e6f7c` 末段（後路宣告）；nextest docs `https://nexte.st/book/reusing-builds.html`；`actions/upload-artifact` / `actions/download-artifact`

<!-- FU-2026-05-23-037 → 🚫 wontfix 2026-05-24: superseded by PR #401 (Mergify removed in favor of GitHub-native auto-merge). See FOLLOWUPS_DONE.md -->

## CLI / Commands

<!-- FU-2026-05-22-007 → ✅ resolved 2026-05-25 — `ecp schema blindspots` shipped as orthogonal third path; ecp side no longer occupies `blind-spots` name, conflict gone (see FOLLOWUPS_DONE.md) -->

<!-- FU-2026-05-23-039 → ✅ done in PR #409 (squash e27622e7, 2026-05-23) -->

<!-- FU-2026-05-23-041 → ✅ done in PR #403 (merged as 7474cc9f) — subprocess.rs::run_self + self_exe promoted, 3 consumers migrated (see FOLLOWUPS_DONE.md) -->

### FU-2026-05-23-042  ·  surfaced in PR #379 /simplify pre-merge review
- **owner**: unassigned
- **renumber-note**: 原號 FU-2026-05-23-035（撞 PR #380 /simplify Go nested-if 那條），2026-05-23 housekeeping 改為 042
- **scope**: `ecp impact --baseline --format json` 輸出 nested rich JSON（changed_symbols[*].{name,kind,filePath,line,change_type}、impact_by_symbol[*].impact[*].{depth,kind,ownerClass,uid,viaConfidence,viaReason} 等）給 LLM consumers 用很合理，但 `pr_analyze` 只取 `name` + `filePath`，多 90% 字段被 deserialize 後立刻丟。為了 bridge 兩種 consumer profile，T5 implementer 加了 `ImpactJson::{changed_files, impact_set_names, changed_symbol_names}` 三個 helper method，每次 invocation 多 ~5-10ms parse + 三個 method 維護成本。
- **why-deferred**: 不算設計不良而是 multi-consumer mismatch；本 PR 流量（每 PR push 1-5 次）的 helper cost < 1 min/year，修這個的 ROI 等於工程師咖啡時間。但若 pr-analyze 流量上來（每天 50+ PR push）就 worth it
- **next-action**: 兩條方案（流量達到 trigger 時擇一）：
  (A) `ecp impact --format minimal` 加新 output variant，輸出 `{changed_symbols: [{name, filePath}], impact_set: [names]}` 兩個 flat array — impact.rs +~30 LoC，pr_analyze 拿掉 3 個 helper method；
  (B) `ecp impact --fields a,b,c` 通用 projection — impact.rs +~60 LoC，未來 consumer 自由選欄位但 schema 漂移時容易撞
- **size**: S (方案 A) / M (方案 B)
- **links**: PR #379 /simplify efficiency review finding 6; `crates/ecp-cli/src/commands/dev/pr_analyze.rs:114-155`（helper methods）; `crates/ecp-cli/src/commands/impact.rs`（待加 format variant）

### FU-2026-05-27-002  ·  surfaced during `ecp gain` design (worktree feat/ecp-gain-dashboard)
- **owner**: unassigned
- **scope**: `ecp gain` dashboard 的 Usage 表格規劃了一欄 `Trend` per-command sparkline（`▁▃▅█`，每字元=一天用量，正規化到 8 級高度），讓人一眼看出某命令用量在升/降/平。設計時刻意先不做：核心三區塊（Usage/Performance/Errors + bar + 數字）已滿足「次數/效能/錯誤」三需求，sparkline 是純 dashboard 美感 + 趨勢 nice-to-have，graceful-degrade 設計（無它則 Trend 欄留空，dashboard 完全可用）。
- **why-deferred**: size-too-large for the focused renderer task — 需多一層 per-command × per-day 分桶聚合（insight.rs 的 `hourly_buckets` 是範式但要改 daily 且 per-command），加上 <2 天資料 degrade 成空白、跨午夜邊界等 edge case，塞進 T6/T7 會讓渲染器臃腫、拖慢核心 dashboard 先綠。
- **next-action**: 核心 `ecp gain` 合併後，在 `render_dashboard` 加 daily-bucket 聚合 fn（仿 insight.rs `hourly_buckets`，改 per-command/per-day）+ `▁▂▃▄▅▆▇█` 8 級映射 + Trend 欄渲染；加一個測試斷言「<2 distinct days → 空白 cell」graceful degrade。
- **size**: S
- **links**: spec `docs/superpowers/specs/2026-05-27-ecp-gain-usage-dashboard-design.md` §4.1（Trend 欄 mockup）; plan `docs/superpowers/plans/2026-05-27-ecp-gain-usage-dashboard.md` Task 7 + Self-Review「Sparkline note」; 聚合範式 `crates/ecp-cli/src/commands/insight.rs::hourly_buckets`

### FU-2026-05-27-003  ·  surfaced in `ecp usage` /simplify (worktree feat/ecp-gain-dashboard)
- **owner**: unassigned
- **scope**: `percentile` 與 jsonl-telemetry 讀取/聚合邏輯在 `insight.rs` 與 `usage.rs` 兩處重複（percentile 函式體 byte-identical；read_window vs read_file 形狀相近但非全同）。simplify ecp reuse 檢查確認 `usage` 沒重造通用 util（`ecp find "aggregate by tool"` → found:false），但 insight/usage 兩個 telemetry 聚合器各自帶一份 percentile。
- **why-deferred**: 經 final Opus review 評估後判定「兩聚合器形狀確實不同（usage 讀 cli-calls+calls 兩檔 + 跨 repo scan + 留 raw 供 --failures；insight 讀單檔 + 時間窗 + hourly bucket），為單一 6 行 percentile 硬抽共用模組得不償失」。caller_count=3 表示 insight 那份有獨立用戶。屬 minor DRY,不值得在本 PR 處理。
- **next-action**: 若日後出現第三個 telemetry-jsonl 讀取者,抽一個 `crates/ecp-cli/src/commands/telemetry_read.rs`（或 ecp-core）共用模組:統一 percentile + jsonl Value 提取 + repo-key 解析,讓 insight/usage/新讀者共用。三方都受益才動。
- **size**: S
- **links**: `crates/ecp-cli/src/commands/insight.rs:232`(percentile)+`read_window`; `crates/ecp-cli/src/commands/usage.rs::percentile`+`read_file`; final review finding #3

<!-- FU-2026-05-23-046 → ✅ done in PR #403 (merged as 7474cc9f) — orphan examples/dump_uid_collisions.rs deleted; ecp dev uid-audit is canonical (see FOLLOWUPS_DONE.md) -->

## Cypher engine

<!-- FU-2026-05-23-006 → pushdown approach in-flight PR #426 (perf/cypher-pushdown-decorator-in); replaces closed PR #424 (Value lifetime ripple avoided by planner-level specialization) -->
<!-- FU-2026-05-23-013 → ✅ done in PR #374 (merged as c094f9ee, see FOLLOWUPS_DONE.md) -->

<!-- FU-2026-05-23-014 → ✅ done in PR #376 (merged as b2d27e48, see FOLLOWUPS_DONE.md) -->

<!-- FU-2026-05-23-015 → ✅ done in PR #377 (merged as 38012c2b, see FOLLOWUPS_DONE.md) -->
<!-- FU-2026-05-23-015 deferred-into: M1 --telemetry-path hidden flag refactor; M2 hours > 168 silent clamping warning; M3 streaming aggregator for huge telemetry files -->

## Review / Impact

<!-- FU-2026-05-23-025 → ✅ done in working tree 2026-05-25 — `ecp impact --literal-coherence` auto-generates PathLiteral split-brain candidate pairs -->

<!-- FU-2026-05-25-001 → ✅ done in PR #449 — literal-coherence wired into `ecp review` aggregate mode (Source::LiteralCoherence, writer-attributed Warn, either-end-in-scope); session_meta.json fixture from candidate payload (see FOLLOWUPS_DONE.md) -->

## Docs / Distribution

### FU-2026-05-23-007  ·  surfaced in promotion-readiness review (2026-05-23)
- **owner**: c7cba51f (orchestrator, after Agent C Sonnet ad7a0bbe crashed mid-scaffold 2026-05-24)
- **status**: **[in-flight PR #406]** — `docs/benchmark-vs-competitors-fu007` opened 2026-05-24. Scaffold framework + ecp-only snapshot landed; real competitor numbers deferred to follow-up after `cargo install codescope` / `cargo install coraline` + CLI verb-table verification.
- **scope**: receipts 只 vs GitNexus（60× cold index）但 GitNexus 是 Node.js 不會被當競品；真正空間競品 codescope (SurrealDB)、coraline (SQLite) 沒同 corpus benchmark；無法支撐「我們比同類 Rust 競品更快」的廣宣 claim
- **why-deferred**: `scripts/parity/benchmark_vs_gitnexus.py` 框架已在但對手換掉非單行改動；要處理三套不同 install / CLI / output 格式
- **next-action**: 新增 `scripts/benchmark/benchmark_vs_competitors.py`；對 `.sample_repo` 對 codescope + coraline 跑 cold-index + 5 種 query；產出 markdown table + svg 圖表；納入 README「Performance」章節
- **size**: M
- **links**: `docs/competitive-landscape.md:58` Borrow list 第 4 條；`scripts/parity/benchmark_vs_gitnexus.py`；README.md:29-69（既有效能章節）

<!-- FU-2026-05-23-036 → ✅ done in PR #408 (see FOLLOWUPS_DONE.md) -->

<!-- FU-2026-05-23-045 → done in PR #396 (see FOLLOWUPS_DONE.md) -->

<!-- FU-2026-05-23-047 → done in PR #399 (see FOLLOWUPS_DONE.md) -->

<!-- FU-2026-05-24-001 → ✅ resolved by same perf/cypher-decorator-borrowed-variant PR as FU-006 -->
<!-- FU-2026-05-24-003 → ✅ done in working tree 2026-05-25 — schema v11 dense node_flags for FunctionMeta flags -->

### FU-2026-05-23-048  ·  carried over from FU-008 sub-part (c) when PR #395 shipped (a)+(b)
- **owner**: unassigned
- **scope**: Homebrew tap + npm wrapper for `ecp`. Current install surface is `curl -sSfL .../install.sh | sh` + `cargo install --git ...` — neither gets shared casually the way `brew install <tool>` or `npm i -g <tool>` does. npm wrapper should ONLY fetch the prebuilt binary tarball from the GH release matching the host triple, NOT trigger a Rust toolchain build (long install time = bounce rate). Homebrew tap should pull from the same release assets; tap rename / formula maintenance is non-trivial.
- **why-deferred**: scope-creep on PR #395 (which already added 3 distribution touchpoints). brew + npm both need their own packaging + version-bump workflow + release-pipeline integration; each is its own M-sized chunk of work.
- **next-action**: two independent PRs — (1) Homebrew tap repo (likely `coseto6125/homebrew-ecp` standalone), formula points at the existing GitHub Release `*.tar.gz` assets per platform; wire a CI job in this repo that runs `brew bump-formula-pr` on tag publish so the tap auto-syncs. (2) npm wrapper package (`@ecp/cli` or `ecp-cli`), `postinstall` script that downloads the matching prebuilt binary into `node_modules/.bin/`, validates checksum, refuses to fall back to cargo. Document the install order in README so users see `brew` / `npm i -g` as top-level options alongside the existing `curl install.sh`.
- **size**: M each (so M-L total, but ship independently)
- **links**: PR #395 description (deferred sub-part c); FU-2026-05-23-008 (parent); README.md:91-111 (existing install section to extend)

<!-- FU-2026-05-24-002 → ✅ done in PR #447 — +4 bench cypher patterns (COLLECT / IN-list / multi-hop / GROUP BY, all >60ms) (see FOLLOWUPS_DONE.md) -->
<!-- FU-2026-05-24-004 → ✅ done in PR #447 — Value::write_dedup_key structural key (A/B ~12% on DISTINCT, no control regression) (see FOLLOWUPS_DONE.md) -->
<!-- FU-2026-05-24-005 → ✅ done in PR #447 — FU-004 before/after A/B in docs/perf-notes.md; historical stacked-delta not reproducible under load, not used (see FOLLOWUPS_DONE.md) -->
<!-- FU-2026-05-24-007 → ✅ done in PR #447 — pinned tail-lang clones + release-binary baselines.md recapture (see FOLLOWUPS_DONE.md) -->
<!-- FU-2026-05-24-008 → ✅ done in PR #447 — benchmark_ecp.py --analyze-runs N (see FOLLOWUPS_DONE.md) -->

### FU-2026-05-24-009  ·  surfaced in this session's PR #410 / PR #416 auto-merge timing
- **owner**: unassigned
- **scope**: Two times this session, the `Auto-merge ready PRs` workflow (introduced by PR #401) enabled auto-merge on a PR within ~60 seconds of open — landing the PR before the orchestrator's /simplify review finished. Result: cleanup commits had to ship in separate follow-up PRs (PR #416 chasing PR #410). The workflow does not honour any signal that says "wait for /simplify first". PR #424 was also closed in favour of PR #426 partly because the original PR #410 shipped before /simplify could nudge an architectural pivot.
- **why-deferred**: needs design — adding a label-based gate (`needs-simplify`, `skip-auto-merge`) might slow normal small-PR throughput where simplify is overkill. Trade-off between agent-driven review discipline and human-driven push throughput.
- **next-action**: design spike — options:
  (a) label `simplify-pending` that the workflow respects (clear after /simplify finishes);
  (b) parse a `simplify-approved` PR review comment as positive signal;
  (c) honour explicit `Disable auto-merge` from PR author / orchestrator that the workflow leaves alone.
  Validate against actual session traces: in N+ sessions, did premature auto-merge cost more than the simplify discipline saved?
- **size**: M (workflow change + label rollout + retro on whether the trade-off works)
- **links**: PR #401 (Auto-merge workflow); PR #410 → PR #416 cleanup chain; PR #424 (closed in favour of #426)

<!-- FU-2026-05-25-002 → ✅ done: anon-display via 87a144c1 (PR #446) + line-base via FU-2026-05-25-004/PR #448 — see FOLLOWUPS_DONE.md -->

<!-- FU-2026-05-26-002 → ✅ done in PR #499 (merged, squash) — JS field_definition → Property capture; flips javascript_reads_field positive + adds this.x= guard (see FOLLOWUPS_DONE.md) -->

### FU-2026-05-29-002  ·  surfaced in PR #499 adversarial review (JS Property capture)
- **owner**: unassigned
- **scope**: JS class-field → Property capture (PR #499) intentionally mirrors the TS `public_field_definition` query, inheriting gaps the 3-agent review surfaced — all consistent with the TS sibling, none regressions: (a) private `#x` fields not captured (`private_property_identifier` ≠ `property_identifier`) → intra-class private-field `ecp impact` shows 0 readers; (b) computed `[Symbol.iterator]` / `[key]` fields not captured; (c) string/number-keyed fields (`'name' = v`, `42 = v`) not captured; (d) same-file multi-class same-named field resolves to the FIRST-declared Property (JS has no receiver type annotation — same Tier-1 SameFile heuristic as Callable targets); (e) anonymous class-expression same-named fields collide on uid (owner_class=None) → second tombstoned, BlindSpot correctly emitted.
- **why-deferred**: out-of-scope for PR #499 (its job was JS↔TS parity for the common `field = val` case). (a)-(c) must be fixed in BOTH JS and TS together to keep the sibling queries symmetric. (d) needs a design call (suppress the edge vs lower-confidence emit). (e) needs synthetic owner-naming for anonymous classes.
- **next-action**: (1) if intra-class private-field impact matters, add `(field_definition property: (private_property_identifier) @property.name) @property` to BOTH javascript/queries.scm and the TS arm; (2) document the multi-class same-file heuristic as a queries.scm comment; (3) optional regression test pinning the anon-class uid-collision BlindSpot.
- **size**: M (cross-lang private-field) / S (doc + collision test)
- **links**: PR #499; TS sibling `crates/ecp-analyzer/src/typescript/queries.scm:208`; resolver Tier-1 SameFile path; guard test `javascript_constructor_assignment_is_not_a_property`

### FU-2026-05-26-003  ·  surfaced in 0.4.3 release PR-cleanup session
- **owner**: session (release-0.4.3)
- **scope**: repo `.git` 累積 16936 loose objects (~21MB) + `.git/gc.log` 擋住自動 gc（"too many unreachable loose objects; run git prune"）。每次 git 操作噴 warning，非阻塞但拖慢。與 ~/.ecp 殭屍圖 GC (FU-2026-05-26-001) 是不同物件庫。
- **why-deferred**: out-of-scope（release 進行中、多 worktree + CI push 並行，不宜動物件庫）
- **next-action**: release 全部完成、所有 release worktree 移除後，`rm .git/gc.log && git prune && git gc`
- **size**: S
- **links**: FU-2026-05-26-001, memory project_release_043_pr_cleanup

<!-- FU-2026-05-27-001 → ✅ done in PR #473 — fingerprint sidecar write made synchronous (see FOLLOWUPS_DONE.md) -->

### FU-2026-05-29-010  ·  surfaced in PR (result-caveat)
- **owner**: unassigned
- **scope**: `result` caveat field covers find exact/fuzzy + impact + inspect + cypher, but NOT the bm25 (find) cross-repo path — per-repo staleness needs its own caveat shape (which of N repos is the warm-attach one). `emit_bucketed_with_metadata` is a deep helper without a single `engine`; threading caveat through run_batch/run_single/run_multi is the work.
- **why-deferred**: size-too-large
- **next-action**: thread per-repo caveat into emit_bucketed_with_metadata; decide caveat shape when multiple repos differ in freshness
- **size**: M
- **links**: PR feat/result-caveat-field; output::emit_with_caveat

### FU-2026-05-29-011  ·  surfaced in PR (result-caveat)
- **owner**: unassigned
- **scope**: extend `engine::caveat()` Some-arm to a SECOND completeness source: when `ecp impact`'s target is a high-collision name with Tier-3-ambiguity-suppressed call edges (DecisionTier::AmbiguousGlobal), surface "caller set may be incomplete: N same-named defs, bare calls unresolved". Today only warm-attach staleness produces a caveat. PR #503 fixed Rust crate:: imports but the Tier-3 cap still suppresses genuinely-ambiguous bare calls by design — those impact results should self-flag.
- **why-deferred**: out-of-scope (separate caveat source from staleness)
- **next-action**: plumb resolver DecisionTier::AmbiguousGlobal count to impact output, feed into caveat()
- **size**: M
- **links**: PR #503, feat/result-caveat-field, resolver.rs Tier-3 cap

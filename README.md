# mpm — Markdown Project Manager

對 markdown 待辦工作紀錄做結構化 CRUD 與圖譜查詢的工具，專為 LLM coding agent
打造。markdown 仍是人類可讀的真相來源；`mpm` 讓 agent **不必讀整份檔，就能精準
拿到所需的片段** —— 一筆 entry、一行 stub、一次關係跳轉、一次 PR 查詢。

它針對的是雙檔「follow-ups」協定（`FOLLOWUPS.md` 存進行中的工作、
`FOLLOWUPS_DONE.md` 存已解決的封存），但資料模型是通用的：有序的 entry、一小組
核心欄位、自由形式的額外欄位，以及具型別的關係（`blocked-on`、`links-to`、
`superseded-by`、`done-in-PR`、`carried-over-from`）。

## 為什麼需要

協定規定 *「開任何 PR 前先讀紀錄」*。對 agent 而言，這等於每個任務都要讀一份
27 KB+ 的檔案 —— 純粹的 context window 稅。`mpm` 用結構化查詢取代 grep，回傳
訊號密集的 [toon](https://crates.io/crates/etoon)（或 `--json`）：

```
mpm list --status open               # 分流，不讀全檔
mpm show FU-2026-05-23-042            # 單筆 entry
mpm query pr:498                      # 這個 PR 是否已關掉某 follow-up？
mpm graph FU-2026-05-23-048 --direction up     # 有什麼 follow-up 依賴（blocked on）這條？
mpm stub FU-2026-05-23-042           # 那一行 resolution stub
```

每次變更都會寫入結構化 store 並 **重新渲染兩份 markdown 檔**，讓人類面對的紀錄
保持同步且可 git-diff。一份可丟棄的 msgpack 快取（以檔案 mtime + schema 版本
守門）讓讀取維持次毫秒級；手動改的 markdown 由 `mpm import` 吸收回 store。

## 安裝

```
cargo install --git https://github.com/coseto6125/markdown-project-manager --bin mpm --locked
```

或從 [Releases](https://github.com/coseto6125/markdown-project-manager/releases) 取得預編譯的執行檔。

## 給 AI agent 用的 skill

`skills/mpm/SKILL.md` 是一份給 LLM coding agent 的 skill 定義：它教 agent
在三個時機（開工前查相關/阻擋工作、工作中記下延後項、結束後標記完成）透過
`mpm` 操作 follow-ups，而非讀整份 markdown。把它複製到 `~/.claude/skills/mpm/`
即可在 Claude Code 中啟用。內容經多輪 A/B 測試校準，連 Haiku 等級的模型也能
零失誤地產出正確命令（含 `done --pr` vs `--branch` 互斥、`graph` 方向語意、
重複 id 的處置等易錯點）。

## 使用方式

`mpm` 從 `--dir <.claude 目錄>` 解析紀錄（預設為 code-graph-nexus 的標準路徑）。
所有讀取指令都接受 `--json`；變更指令會把受影響的 id 印到 stdout，方便 agent 擷取。

### 建立 / 讀取

```
mpm add --category "Parser & Schema" --scope "..." --why "..." --size S --surfaced "PR #520"
mpm show <id> [--json]
mpm stub <id>
mpm list [--status open|done|wontfix] [--category C] [--size S|M|L] [--blocked] [--pr N] [--json]
mpm next-id
```

### 變更

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

### 圖譜 / 查詢 / 維護

```
mpm graph <id> [--direction up|down|both] [--depth N] [--json]
mpm query status:open size:L          # 扁平的 key:value AND 過濾
mpm query pr:385                       # 可用過濾鍵：status、category、size、pr、blocked-on、links-to、owner
mpm render [--check]                   # 重新渲染 markdown；--check 偵測漂移時 exit 1
mpm import [--dry-run]                 # 解析 markdown 進 store
mpm validate                           # dangling 連結、重複 id
```

## 資料模型

| 概念 | 說明 |
|---|---|
| Entry | `id` + `category` + `status` + 6 個核心欄位（owner/scope/why-deferred/next-action/size/links）+ 有序的額外欄位 |
| 欄位順序 | 逐筆 verbatim 保留（`original-scope` 原樣 round-trip 成 `original-scope`，一次性的鍵保留原位） |
| Status | `open` · `done` · `wontfix` · `blocked`（留在 open）· `superseded` |
| Edges | `blocked-on`、`links-to`（來自 `[[FU-id]]`）、`superseded-by`、`carried-over-from`、`done-in-PR`、`surfaced-in-PR` |
| 渲染 | canonical 形式；第一次 `import` + render 是一次性正規化，之後 byte-stable |

## 授權

MIT

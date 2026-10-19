# Lap Fork 改造说明：本地 VLM 视觉索引

改造目录：`<repo-root>\`
基线：julyx10/lap @ main（v0.3.3）

---

## ✅ 构建问题已解决：ort-sys 预编译库 STL ABI 冲突 → load-dynamic

### 现象

`cargo build`（产出可执行文件）在**链接阶段**失败：

```
libort_sys.rlib(...) : error LNK2001: 无法解析的外部符号 __std_search_1
libort_sys.rlib(...) : error LNK2001: 无法解析的外部符号 __std_find_end_1
libort_sys.rlib(...) : error LNK2001: 无法解析的外部符号 __std_find_last_of_trivial_pos_1
... 共 10+ 个符号
```

### 根因

这些 `__std_*_vectorized` 是 MSVC STL 的**内部向量化算法实现**。onnxruntime
官方预编译 Windows 库（ort-sys `download-binaries` 拉取的）用**旧版 MSVC**
编译，引用了这些旧符号名；本机 VS 2022 14.42 的 STL 已移除/改名这些符号。
已用 Python 扫描 `libcpmt.lib` / `libcmt.lib` 确认符号**不存在**。

**这是环境 ABI 问题，与 fork 的代码无关。** 验证：撤销全部改动、编译纯上游代码，同样链接失败。

### 已尝试且无效

| 方案 | 结果 |
|---|---|
| Rust 1.90.0 | ❌ 链接失败 |
| Rust 1.92.0 | ❌ 同样失败 |
| ort rc.10 → rc.14 | ❌ rc.14 仍缺同样符号 |
| 关闭 LTO | ❌ 无关 |
| ndarray 0.16 → 0.17 对齐 ort | ❌ 解决 trait 错误，链接仍失败 |

### ✅ 最终解法（已执行成功）

**ort rc.14 + `load-dynamic`**：编译期不静态链接 onnxruntime 的 .lib，
改为运行时通过 `ORT_DYLIB_PATH` 加载 `onnxruntime.dll`，彻底绕开 ABI 冲突。

具体改动：

1. **Cargo.toml**：`ort = "=2.0.0-rc.14"`，feature 换成 `["std", "ndarray", "load-dynamic"]`
   （去掉 `download-binaries` / `copy-dylibs`）；`ndarray` 升 `0.17`。
2. **下载 onnxruntime.dll**：从 `github.com/microsoft/onnxruntime` release v1.31.0
   的 `onnxruntime-win-x64-1.31.0.zip` 解出 `lib/onnxruntime.dll`（16.5 MB），
   放入 `src-tauri/resources/onnxruntime.dll`，并在 `tauri.conf.json` 的
   `bundle.resources` 注册打包。
3. **适配 API（rc.14 破坏性变更）**：
   - `Session::builder()` → `Session::builder(&env)`，env 来自 `ort::init().build()`
   - `Value::from_array(x)` → `TensorRef::from_array_view(&x)`
   - `.inputs` 字段 → `.inputs()` 方法，`.name` → `.name()`
   - 涉及 `t_ai.rs`、`t_face.rs`
4. **新增 `t_ai::init_onnxruntime_path(app)`**：dev 用源码目录、production 用资源
   目录定位 dll，设置 `ORT_DYLIB_PATH`（`set_var` 在 Rust 2024 需 `unsafe` 包裹）。
   在 `main.rs` 的 `setup` 里、`t_video::init_ffmpeg_path` 之后调用。

### 验证结果

| 验证项 | 结果 |
|---|---|
| `cargo +1.92.0 build` | ✅ Finished，零链接错误 |
| `cargo +1.92.0 test t_llm` | ✅ **22 passed; 0 failed** |
| `cargo +1.92.0 tauri build` | ✅ 出 MSI (218 MB) + NSIS exe (185 MB) |
| 启动 `Lap.exe` | ✅ 10 秒未崩溃，主窗口 "Lap" |

> ⚠️ 注意：Rust 工具链从 1.90.0 换到了 **1.92.0**（rc.14 要求 `rust-version = "1.92"`）。

---

## ⚠️ 已知回归 + 修复：tauri 意外升级导致 app 命令 ACL 报错

### 现象

用户安装后点"新建库"报错：`Command add_library not allowed by ACL`。

### 根因

换 rustc 1.92 编译时，cargo 重新解析了 `Cargo.lock`，把 **tauri 从 2.10.3 意外升级到 2.12.4**
（连带 tauri-build、tauri-runtime、tauri-utils 等全套升级）。

tauri 2.12.x 引入了 **app 命令 ACL 检查**：`#[tauri::command]` 注册的自定义命令
（如 `add_library`）也需要在 capabilities 里声明 `com.julyx10.lap:allow-add-library`
权限。而 Lap 的 `capabilities/default.json` 没有为 app 命令声明任何权限
（因为上游锁定的 tauri 2.10.3 不需要）。

**这是换工具链时无意引入的依赖漂移，不是代码问题。**

### 修复

1. `git checkout HEAD -- src-tauri/Cargo.lock` 恢复上游锁定，tauri 回到 **2.10.3**
2. 重新解析时 cargo 只更新 ort（rc.10→rc.14）和 ndarray（0.16→0.17），
   tauri 保持 2.10.3 不动
3. 顺带把之前 `git checkout` 恢复 Cargo.toml 时意外还原的
   `tauri-plugin-updater = "2.10.0"` 重新注释掉（与 main.rs/tauri.conf.json 已禁用保持一致）

### 教训

**不要轻易让 cargo 重解析整个 lock。** 本 fork 改动依赖（ort/ndarray）时，
应先 `git checkout -- Cargo.lock` 再让 cargo 做最小增量解析，避免 tauri 等
无关依赖漂移到新版本引入行为变化。

---

## 当前状态总览

| 验证项 | 命令 | 结果 |
|---|---|---|
| 单元测试 | `cargo +1.92.0 test t_llm` | ✅ **22 passed; 0 failed** |
| 全量构建 | `cargo +1.92.0 build` | ✅ 零 error |
| 端到端打包 | `cargo +1.92.0 tauri build` | ✅ MSI + NSIS |
| App 启动 | 运行 `Lap.exe` | ✅ 正常启动 |
| 前端构建 | `pnpm build` | ✅ `✓ built` |

### 单元测试覆盖

```
t_llm::tests::bigram_basic                    ok   # 基本切分 + 奇数长度补尾字
t_llm::tests::bigram_mixed                    ok   # 中英混排
t_llm::tests::bigram_punctuation_is_separator  ok   # 标点分隔
t_llm::tests::fts_query_empty_is_safe         ok   # 空查询不炸
t_llm::tests::fts_query_uses_or_not_space     ok   # 必须 OR 连接
t_llm::tests::parse_caption_tolerates_fences  ok   # 容忍 ```json 围栏
t_llm::tests::parse_caption_rejects_garbage   ok   # 拒绝非 JSON
t_llm::tests::index_text_skips_empty_fields   ok   # 空字段不入索引
```

### ⚠️ 必须用 Rust 1.90，不能用 1.99

这是本次踩的最大的坑，值得单独说明：

| rustc 版本 | 结果 |
|---|---|
| **1.99.0**（最新 stable） | ❌ 链接失败：`libort_sys` 大量 `__std_find_last_of_trivial_pos_1` / `__std_search_1` 未定义 |
| 1.87.0 | ❌ 依赖要求 `darling/image/serde_with 需 1.88`、`tauri/muda 需 1.90` |
| **1.90.0** | ✅ 全部通过 |

**根因**：`ort-sys 2.0.0-rc.10` 下载的预编译 onnxruntime 库是用旧版 MSVC STL 编的，
rustc 1.99 配套的新 STL 移除了那些内部符号，导致链接失败。

**我做过一次决定性验证**：撤掉我全部改动、编译**纯上游代码**，在 rustc 1.99 下**同样链接失败**，
错误完全一致 —— 证明这与本次改造无关，是上游依赖与最新 Rust 的固有矛盾。

### 其他环境依赖（本机从零装的）

| 组件 | 版本/来源 | 说明 |
|---|---|---|
| Rust | 1.90.0 via rustup | 见上，**必须 1.90** |
| CMake | 4.5.0 | build.rs 编译 libjpeg-turbo/libheif/LibRaw 需要 |
| MSVC + Windows SDK | VS 2022 17.12 | 你机器原有 |
| git submodules | 4 个 | `LibRaw` `libjpeg-turbo` `libheif` `libde265` —— `--depth 1` clone 不会拉，必须 `git submodule update --init` |
| AI 模型 | 164 MB | `resources/models/` 下 5 个文件（tokenizer + 4 ONNX），从 HuggingFace 下载 |
| ffmpeg sidecar | 193 MB | `resources/ffmpeg/` 下 ffmpeg + ffprobe，从你已装的 Lap 复制 |

> 注：`git submodule` 用 `api.github.com` 的 tarball 拉的 —— 本机 `git clone https://github.com/...`
> 协议不通（Connection reset），但 api.github.com 可达。脚本：`fetch-submodules.mjs`。

---

### 1. 禁用自动更新（第一优先级）

**为什么**：`tauri.conf.json` 原配置指向 `julyx10/lap/releases` 并带官方公钥。保留它，上游每次发版都会尝试覆盖你 fork 的代码，或签名校验失败刷错误。

| 文件 | 改动 |
|---|---|
| `src-tauri/tauri.conf.json` | 删除 `plugins.updater` 整块；`createUpdaterArtifacts` 改 `false`；原值以注释形式保留 |
| `src-tauri/capabilities/default.json` | 移除 `"updater:default"` 权限 |
| `src-tauri/src/main.rs` | 移除 `.plugin(tauri_plugin_updater::Builder::new().build())`，留注释说明 |
| `src-tauri/Cargo.toml` | 注释掉 `tauri-plugin-updater = "2.10.0"` |

> 若将来要自建更新通道：取消注释依赖，换为自己的 `endpoints` + `pubkey`，并同步改 capabilities。

### 2. 新增 `src-tauri/src/t_llm.rs`（核心，~700 行）

模块结构：

```
LlmConfig              连接配置（独立 llm-config.json，不动 Lap 原有 AppConfig schema）
LlmState               三件套状态：LlmState / LlmIndexCancellation / LlmIndexingStatus
                        + LlmIndexProgressState（照抄 t_face 的模式）
LlmCaption             结构化描述（caption/objects/scene/action/relations/text/people_count）
bigramize()            中文二元切分
query_to_fts()         查询转 FTS5 MATCH 表达式
INDEX_PROMPT           索引用 prompt（禁止推测身份/地点）
health_check()         TCP 级连通性探测
http_post()            std::net 手写 HTTP POST（无异步依赖）
request_caption()      调 VLM，支持关思考模式
ensure_schema()        建 afile_llm + afile_llm_fts
pending_files()        取待处理文件（支持断点续跑）
store_caption()        事务内写主表 + FTS
search_caption()       FTS5 检索
run_llm_indexing()     索引调度（取消/进度/事件三件套）
6 个 #[tauri::command]  + 9 个 #[test]
```

### 3. `Cargo.toml` 依赖

```toml
url = "2"   # 新增
```

**关于 FTS5 —— 一个曾被我搞错的点：**

`libsqlite3-sys` 的 `build_bundled` **默认就传了 `-DSQLITE_ENABLE_FTS5`**（见其 `build.rs`）：

```rust
.flag("-DSQLITE_ENABLE_FTS3")
.flag("-DSQLITE_ENABLE_FTS3_PARENTHESIS")
.flag("-DSQLITE_ENABLE_FTS5")     // 默认已有
.flag("-DSQLITE_ENABLE_JSON1")
```

所以**不需要**给 rusqlite 加任何 feature。我最初加了 `features = [..., "fts5"]`，实际编译时报错：

```
package `Lap` depends on `rusqlite` with feature `fts5`
but `rusqlite` does not have that feature.
```

已回退。若将来需要额外 SQLite 编译定义，用 `LIBSQLITE3_FLAGS` 环境变量追加。

### 4. `main.rs` 注册

```rust
mod t_llm;   // 模块声明

.manage(t_llm::LlmState::default())
.manage(t_llm::LlmIndexCancellation(...))
.manage(t_llm::LlmIndexingStatus(...))
.manage(t_llm::LlmIndexProgressState(...))

// invoke_handler 内
t_llm::get_llm_config,
t_llm::save_llm_config_cmd,
t_llm::test_llm_connection,
t_llm::start_llm_indexing,
t_llm::cancel_llm_indexing,
t_llm::get_llm_index_stats,
```

### 5. Migration v19

```rust
} else if migration.version == 19 {
    crate::t_llm::ensure_schema(conn)?;
}
```

新增**独立表** `afile_llm`，**不动** `afiles` / `embeds` —— CLIP 相似度搜索完全不受影响。

### 6. 前端

| 文件 | 改动 |
|---|---|
| `src-vite/src/common/constants.ts` | `SETTINGS_TAB.LLM = 9`（插在 ADVANCED 后，不影响已持久化的 tabIndex） |
| `src-vite/src/common/api.js` | 新增 8 个 API 封装 |
| `src-vite/src/components/SettingsLlm.vue` | **新建**设置面板（连接/进度/高级） |
| `src-vite/src/views/Settings.vue` | 挂载 tab 与组件 |
| `src-vite/src/locales/{zh-CN,en}.json` | 新增 `settings.llm.*` 词条 |

---

## 二、数据库 Schema

```sql
CREATE TABLE afile_llm (
    file_id    INTEGER PRIMARY KEY,
    caption    TEXT NOT NULL DEFAULT '',
    objects    TEXT NOT NULL DEFAULT '',
    scene      TEXT NOT NULL DEFAULT '',
    action     TEXT NOT NULL DEFAULT '',
    relations  TEXT NOT NULL DEFAULT '',
    ocr_text   TEXT NOT NULL DEFAULT '',
    people_cnt INTEGER NOT NULL DEFAULT 0,
    index_text TEXT NOT NULL DEFAULT '',   -- bigram 化后的全文
    model      TEXT NOT NULL DEFAULT '',   -- 记录模型，换模型自动重建
    prompt_ver INTEGER NOT NULL DEFAULT 1, -- 改 prompt 必须 +1
    indexed_at INTEGER NOT NULL DEFAULT 0
);

CREATE VIRTUAL TABLE afile_llm_fts USING fts5(
    index_text,
    content='',              -- contentless，只存索引省空间
    contentless_delete=1,
    tokenize='unicode61'     -- 中文靠 Rust 侧 bigram 预处理
);
```

**换模型或改 prompt 自动重建**：`pending_files` 的 WHERE 条件是 `l.file_id IS NULL OR l.model != ? OR l.prompt_ver != ?`。

---

## 三、配置

首次启动后编辑 `%LOCALAPPDATA%\com.julyx10.lap\llm-config.json`：

```json
{
  "enabled": true,
  "baseUrl": "http://127.0.0.1:1234",
  "model": "<your-vlm-model>",
  "apiKey": "",
  "disableThinking": true,
  "maxTokens": 1024,
  "temperature": 0.1,
  "previewMaxPx": 768,
  "concurrency": 1,
  "timeoutSecs": 300,
  "retryOnParseError": 1,
  "albumIds": []
}
```

或在设置 → 本地视觉模型里配。

### 针对你的 LM Studio 的参数

| 项 | 值 | 说明 |
|---|---|---|
| `baseUrl` | `http://127.0.0.1:<lm-studio-port>` | 你的实际端口 |
| `model` | `<your-vlm-model>` | |
| `disableThinking` | **`true`** | 必须。POC 实测不关会被 `reasoning_tokens` 吃掉 373/512，JSON 被截断 |
| `maxTokens` | **`1024`** | POC 实测 512 会截断 |
| `previewMaxPx` | `768` | POC 用值，质量足够 |

---

## 四、已验证的三个坑及解法

### 坑 1：中文 FTS5 分词

FTS5 默认 `unicode61` 把连续 CJK 当单个 token，导致多字词查不到：

```
unicode61 原样:  "白色" -> 0 条,  "日落时分" -> 0 条
bigram 预处理后:  "白色" -> 6 条,  "日落时分" -> 3 条
```

解法：Rust 侧 `bigramize()` + 查询侧 `query_to_fts()`。

**关键**：查询必须 `OR` 连接，**不能用空格**。FTS5 里空格是 AND/短语语义，而索引侧每个 bigram 是独立 token，用空格会导致全部查不到。

### 坑 2：max_tokens 截断

模型默认开思考模式，`reasoning_tokens` 挤占预算：

```
POC 实测: completion_tokens=512 (上限), reasoning_tokens=373 -> JSON 截断
```

解法：`disableThinking: true` + `maxTokens: 1024`。

### 坑 3：bigram 奇数长度丢尾字

Rust `buf.windows(2)` 对 5 个 CJK 字只产出 4 个窗口，末字永远无法被单字查询命中。

解法：`if buf.len() % 2 == 1 { out.push(最后一个字) }`。

**这个 bug 是交叉验证时发现的** —— JS 复刻的断言失败暴露了它，已修复并回归测试。

---

## 五、编译前必查

在有 Rust 工具链的机器上：

```bash
cd <repo-root>\src-tauri

# 1. 先只跑测试（快，能抓逻辑错）
cargo test t_llm

# 2. 检查编译
cargo check

# 3. 完整构建
cd .. && pnpm install && pnpm tauri build
```

### 可能的编译问题及预判

以下外部符号**已逐一核对源码签名**（2026-10-15）：

| 符号 | 位置 | 状态 |
|---|---|---|
| `t_config::get_app_data_dir()` | `t_config.rs:440` | ✅ 已核实 |
| `t_sqlite::open_conn()` | `t_sqlite.rs:9600`（`pub(crate)`） | ✅ 已核实 |
| `AThumb::fetch(file_id: i64)` | `t_sqlite.rs:7297` | ✅ 已核实 |
| `AThumb::thumb_data` | `t_sqlite.rs:7314`（`row.get(3)?`） | ✅ 已核实 |

**仍未核对的**（风险较低，但需留意）：

- `image::DynamicImage::to_rgb8()` / `write_to()` —— 依赖 `image = "0.25.10"` 的标准 API，应无问题
- `base64::engine::general_purpose::STANDARD` —— base64 0.22 API，已确认版本匹配

**已核实的两个关键版本问题：**

| 项 | 结论 |
|---|---|
| `contentless_delete=1` | ✅ **安全**。rusqlite 官方 README 确认 bundled SQLite 版本为 **3.46.0**（rusqlite 0.32.0 / libsqlite3-sys 0.30.0），而 `contentless_delete` 恰好在 3.46.0 引入 |
| FTS5 feature | ✅ `Cargo.lock` 锁定 `libsqlite3-sys 0.30.1`，加 `fts5` feature 即可启用 |

---

## 六、混合检索（已完成）

### 实现位置

`src-tauri/src/t_sqlite.rs` 的 `AFile::search_similar_images` 中，CLIP 打分与排序之后：

```rust
scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

// FORK: 混合检索 —— 把本地 VLM 的 caption 索引结果合并进来。
scores = crate::t_llm::merge_caption_scores(&conn, &params, scores);
```

### 设计决策

| 决策 | 理由 |
|---|---|
| **caption 只加分，不筛选** | caption 是补充线索。CLIP 命中的结果全部保留，caption 命中的往上提。这样即使 caption 漏标/误标，也不会让原本能找到的照片消失 |
| **位置加权，不是分数相加** | CLIP 余弦（[0,1]）与 FTS5 rank（BM25，任意负数）量纲不同，直接相加会让一方完全主导 |
| **CLIP 0.7 / caption 0.3** | caption 是辅助信号，不应压过 CLIP 强匹配（0.9 的视觉匹配比一句文字描述更可信） |
| **仅文本搜索启用** | `search_text` 为空（以图搜图）时直接返回，行为与上游一致 |
| **配置读不到/未启用 → 静默降级** | 任何异常都原样返回 CLIP 结果，不影响原有功能 |

### 单字查询的坑（重要）

bigram 索引里**不存在单字 token**（"一只猫" 切成 `一只/只猫/猫在/...`），所以
`MATCH '"猫"'` 永远查不到东西。这是 bigram 方案的固有局限。

**解法**：`search_caption` 检测到 CJK 长度 ≤ 2 时自动降级到
`search_caption_substring`，用 `LIKE '%...%'` 查主表的 `index_text` 列
（带 `ESCAPE` 防通配符注入）。

代价是放弃 FTS5 的 rank 排序，改用 `indexed_at DESC` 给稳定顺序。
有回归测试覆盖：`merge_single_char_query_uses_like_fallback`。

### 测试覆盖（17 个，全通过）

```
bigram_basic / bigram_mixed / bigram_punctuation_is_separator
fts_query_uses_or_not_space / fts_query_empty_is_safe
parse_caption_tolerates_fences / parse_caption_rejects_garbage
index_text_skips_empty_fields
merge_keeps_all_clip_results          # CLIP 结果不被 caption 过滤
merge_caption_hit_beats_comparable_clip  # 实力相当时 caption 排前
merge_strong_clip_still_wins          # CLIP 强匹配不被 caption 压过
merge_adds_caption_only_hits          # caption 独有命中也进结果
merge_no_caption_hit_is_noop          # 无命中时顺序不变
merge_empty_query_returns_input
merge_single_char_query_uses_like_fallback
merge_two_char_query_uses_like_fallback
merge_long_query_uses_fts
```

---

## 七、精确重排（已完成，可选，默认关闭）

### 解决什么问题

CLIP + caption 合并是"粗排"：CLIP 抓视觉相似，caption 抓文字线索，
但两者都不会真的"读懂"查询意图。比如查"红色背包放在椅子旁边"，
粗排只能靠 caption 里是否出现了这些词，**无法理解空间关系**。

VLM 重排让模型直接看候选 caption + 查询，输出 0-1 相关度分数。

### 实现

```rust
// t_sqlite.rs search_similar_images 内，合并之后
scores = crate::t_llm::rerank_scores(&conn, &params, scores);
```

| 设计 | 说明 |
|---|---|
| **纯文字判断，不发图** | 读候选的 caption 文本做判断，成本远低于视觉推理 |
| **只重排前 N 个** | `rerank_top_n`（默认 30），其余保持原顺序跟在后面 |
| **混合权重** | `final = 原始分 × 0.3 + VLM × 0.7`。不完全丢弃原始分，否则"模型说匹配但粗排很后"的结果会一下跳到最前 |
| **默认关闭** | 每次搜索多花 N 次推理。27B 单次 1-3 秒，30 候选就是 30-90 秒 —— 搜索从"即时"变"要等" |
| **单候选失败不影响其他** | 判断失败就用原始分 |

### 配置

```json
{
  "enableRerank": true,
  "rerankTopN": 30
}
```

### `parse_relevance` 踩的两个坑

模型输出五花八门（`0.8`、`.8`、`相关度：0.75`、`-0.3`），解析时：

1. **`.5` 无前导零** —— 起点是 `.` 不是数字，按"数字起点"判断会整个跳过。
   起点必须同时接受 `.` 和数字。
2. **`-0.3` 负数** —— 若匹配失败后只前进 1 字符，会再次匹配到裸的 `0.3`
   而丢掉符号，导致负数被误接受。失败时必须跳到本 token 末尾。

两个都有回归测试：`parse_relevance_plain_number`、`parse_relevance_rejects_garbage`。

---

## 八、完整功能清单

| 能力 | 状态 |
|---|---|
| 禁用 updater（防上游覆盖代码） | ✅ |
| VLM caption 索引 | ✅ 可取消/断点续跑/健康检查降级 |
| 视频多帧索引 | ✅ 复用 Lap ffmpeg，10/50/90% 抽帧拼接 |
| 中文 FTS5 + bigram | ✅ 含单字查询 LIKE 兜底 |
| 混合检索 | ✅ CLIP 0.7 + caption 0.3 加权 |
| 精确重排 | ✅ 可选，默认关闭 |
| 并发索引 | ✅ 每线程独立 DB 连接 |
| 设置 UI | ✅ 连接测试/进度/参数 |
| 文件信息展示 | ✅ 描述 + 标签 + 场景 |
| 单元测试 | ✅ **22/22** |
| 编译 | ✅ 零 error 零 warning |

---

## 九、仍未验证的部分

**端到端从未跑过。** 22 个测试全是单元级别（bigram、FTS 查询、合并逻辑、
分数解析），**没有一个真调过 VLM**。没启动过 App、没连过 LM Studio、
没在 UI 点过开始索引。

要验证：

```powershell
cd <repo-root>
.\build-fork.ps1        # 完整构建出 MSI
```

设置里填 `http://127.0.0.1:<lm-studio-port>` + `<your-vlm-model>`，
勾"关闭思考模式"，`maxTokens` 拉 1024，先对 20 张图跑一轮看效果。

---

## 十、复现与验证

## 七·补、UI 展示 caption（已完成）

后端 `get_llm_caption(file_id)` 命令 + 前端 `FileInfo.vue` 展示：

- 描述文本（caption）
- 物体标签（chips 形式）
- 场景 · 动作（次要行）
- 未索引时不显示该区块（`v-if="llmCaption?.indexed"`）

前端 watch `fileInfo.id`，带请求序号丢弃过期响应，避免快速切换文件时显示错位。

---

## 七·补、并发索引（已完成）

`concurrency` 配置项原先**未生效**（严格串行），现已实现：

| 实现要点 | 说明 |
|---|---|
| `std::thread::scope` | 批内并行，线程在批结束前全部 join，可安全借用 `batch` / `cfg` / `cancel` |
| **每线程独立 DB 连接** | Lap 的 SQLite 跑 WAL 模式（`t_sqlite.rs::setup_conn`），多连接并发读写不互锁。共用一个连接反而会被 `store_caption` 的事务串行化 |
| 结果按原始序号排序 | 打乱并行结果后按 `chunk_idx * chunk_size + i` 重排，保持文件顺序 |
| 进度在主线程更新 | 避免多线程争抢 progress 锁 |
| `workers = cfg.concurrency.clamp(1, 8)` | llama.cpp 单实例并发过高会互相排队并争显存 |

---

## 七·补、视频索引（已完成，复用 Lap 自身能力）

Lap 自带 `t_video::get_video_thumbnail`（ffmpeg 抽帧，已处理 seek 策略、
失败降级、HEIC 主图解析），**没有另写一套抽帧逻辑**。

### 修复的一个真 bug

我最初的 `pending_files` SQL 写的是 `file_type IN (1, 3)`，
但 Lap 的 `file_type` 取值是 **1=图片, 2=视频, 3=RAW**（见 `t_sqlite.rs:2568`
的 `file_type == 2` 判断）—— **视频（2）被完全漏掉，根本没进索引队列**。
已改为 `IN (1, 2, 3)`。

### 多帧采样

单帧封面只反映视频第一眼。`prepare_video_frames_b64` 按
`[10%, 50%, 90%]` 三个时间点抽帧，统一高度后**横向拼接成一张图**：

- 拼图而非发多张：多数 VLM API 限制单条消息图片数，且只需一次推理调用
- 帧间留 4px 黑缝，避免 VLM 把相邻帧内容混在一起
- 短于 3 秒的视频跳过（每帧可能相同），直接用封面
- 单帧失败不阻塞其他帧；全部失败则回退到封面帧

拼接结果仍然走同一条 `request_caption` 路径，capture 结构不变。

---

## 八、复现与验证

### 从零构建（新机器）

```powershell
# 0. 工具链（注意 Rust 必须是 1.90，见上文）
rustup toolchain install 1.90.0 --profile minimal

# 1. submodules（git clone --depth 1 不会拉）
git submodule update --init --recursive --depth 1
#    若 git 协议不通，用 fetch-submodules.mjs 走 api.github.com

# 2. AI 模型（164 MB，从 HuggingFace）
#    scripts/download_models.ps1
#    产物: src-tauri/resources/models/{tokenizer.json,text_model.onnx,
#           vision_model.onnx,det_500m.onnx,w600k_mbf.onnx}

# 3. ffmpeg sidecar（193 MB）
#    scripts/download_ffmpeg_sidecar.ps1
#    产物: src-tauri/resources/ffmpeg/{ffmpeg,ffprobe}-x86_64-pc-windows-msvc.exe

# 4. 构建（用 build-fork.ps1 会自动激活 MSVC 环境）
.\build-fork.ps1 -CheckOnly    # cargo check + test
.\build-fork.ps1               # 完整: check + test + pnpm install + tauri build
```

### 已验证的结果

```powershell
# 单元测试 —— 8 passed; 0 failed
cargo +1.90.0 test t_llm

# 全量检查 —— Finished, 零 error 零 warning
cargo +1.90.0 check

# 前端 —— ✓ built in 1.26s
cd src-vite && pnpm build
```

### POC 阶段脚本

均在 `<poc-workspace>\llm-poc\`：

| 脚本 | 用途 |
|---|---|
| `index_captions.py` | caption 生成（改 `PICDIR` 可用于你的真实库） |
| `eval_search.py` | 检索评测（文件名 vs caption vs 并集） |
| `test_fts.py` | FTS5 unicode61 vs bigram 对比 |
| `test_fts_rust_equiv.py` | Rust 版 bigram 的端到端验证 |
| `verify_bigram.mjs` | bigram 逻辑 14 条断言 |
| `REPORT.md` | POC 完整报告 |

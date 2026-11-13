# 01 — Serde Serialization Architecture & Compile-Time Reflection

| Field | Value |
| :--- | :--- |
| **Type** | Concept Note & Design Decision |
| **Domain** | [`./README.md`](./README.md) (`phase-c-disk-persistence`) |
| **Target Files** | [`../../../../Cargo.toml`](../../../../Cargo.toml) · [`../../../../src/lib.rs`](../../../../src/lib.rs) |
| **Verification Mode** | Codebase + Official Serde Specification (`VERIFIED` ✅) |
| **Confidence** | `VERIFIED` ✅ |
| **Updated** | 2026-11-09 |
| **Context** | Explains why Rust cannot raw-dump struct bytes, why `serde` and `serde_json` are split into two crates, how `#[derive]` macros work, and why `MetaData` requires `#[serde(untagged)]`. |

> **TL;DR:** Rust erases struct field names at compile time and stores heap pointers (`ptr`, `len`, `cap`) inside structs rather than inline text, making raw memory dumps impossible. **`serde`** uses compile-time AST macros (`#[derive(Serialize, Deserialize)]`) to teach `LogEntry` how to walk its fields before names are erased, while **`serde_json`** translates those walked fields into UTF-8 JSON bytes. Adding `#[serde(untagged)]` on `MetaData` strips Rust enum variant wrappers so dictionaries serialize cleanly as `{"task_id": 1}`.

---

## 1. Why the Rust Compiler Cannot Raw-Dump Struct Bytes

In CPython 3.14, every object carries a runtime dictionary (`obj.__dict__`) or type descriptor (`PyTypeObject->tp_members`) in RAM. When you call `json.dumps()`, Python inspects those field name strings at runtime. Rust has **zero runtime reflection**: once compiled to machine code, field names like `timestamp` and `message` no longer exist in the binary—only raw CPU register offsets (`[rsp + 0x18]`) remain.

Furthermore, dumping the raw `sizeof::<LogEntry>()` stack bytes to a file writes **memory addresses (pointers)**, not the actual strings or hash maps.

```text
Title: Stack vs Heap Memory Layout of LogEntry in RAM
Read: Left shows the 80-byte Stack struct; Right shows actual text on the Heap.

STACK (LogEntry = ~80 bytes)              HEAP (Separate Dynamic RAM)
┌──────────────────────────────────┐      ┌──────────────────────────────┐
│ log_type:  1 (ToolExecution)     │      │ 0x7fff_1040: "10:45:02.120"  │
│ log_level: 3 (INFO)              │      ├──────────────────────────────┤
│ timestamp:                       │      │ 0x7fff_98a0: "SQL query ok"  │
│   ├── ptr: 0x7fff_1040 ──────────┼─────►├──────────────────────────────┤
│   ├── len: 12, cap: 16           │      │ 0x7fff_c200: HashMap Buckets │
│ message:                         │      │   "task_id" -> Integer(2)    │
│   ├── ptr: 0x7fff_98a0 ──────────┼─────►└──────────────────────────────┘
│   ├── len: 12, cap: 16           │
│ metadata:  Some(Dict(0x7fff_c200))┼─────▲
└──────────────────────────────────┘
❌ RAW MEMORY DUMP writes `0x7fff_1040` (useless garbage pointer on restart)!
✅ SERIALIZATION follows `0x7fff_1040` to copy `"10:45:02.120"` with key `"timestamp"`.
```

| Dimension | CPython 3.14 (`json` / `pydantic`) | Rust Native (`rustc` + `serde`) |
| :--- | :--- | :--- |
| **Field Name Storage** | Kept alive in RAM inside `__dict__` / `PyTypeObject` at runtime. | **Erased completely** during compilation; zero runtime overhead. |
| **How Inspection Works** | Runtime dictionary iteration (`for k, v in obj.__dict__.items()`). | Compile-time procedural macro (`#[derive(Serialize)]`) reads source AST. |
| **String / Dict Layout** | Heap-allocated `PyUnicodeObject` / `PyDictObject` pointers. | Stack header (`ptr, len, cap`) pointing to separate heap allocations. |
| **Runtime Speed** | Slower (inspects types & pointers dynamically on every call). | **10x–30x faster** (monomorphized inline machine code per struct). |

---

## 2. Why `serde` AND `serde_json` Both Exist (The N × M Problem)

A common misconception is that **Serialization == JSON**. In systems engineering, **Serialization** means converting an in-memory pointer graph into a linear sequence of primitives, whereas **JSON** is merely one specific text syntax (alongside Bincode, MessagePack, TOML, YAML, and Parquet).

If every format crate had to inspect Rust structs on its own, every struct author would have to implement 10 different traits (`ToJson`, `ToToml`, `ToBincode`, `ToYaml`). Instead, Rust splits the job into two halves—just like LLVM splits compilers into a **Frontend (`serde`)** and a **Backend (`serde_json`)**:

```text
Title: The Serde Two-Layer Decoupling Architecture
Read: Left = Your Structs (`serde`); Middle = Visitor Contract; Right = Formats.

 PART 1: `serde` Crate                 THE BRIDGE               PART 2: Format Crates
 (Knows YOUR Struct Fields,            (Format-Agnostic         (Knows Syntax Rules:
  Knows ZERO about JSON `{}`)           Token Stream)            Quotes, Brackets, Binary)
┌───────────────────────────┐         ┌──────────────────┐     ┌─────────────────────────┐
│ #[derive(Serialize)]      │         │ serialize_struct │     │ `serde_json`            │
│ pub struct LogEntry       ├────────►│ serialize_field  ├────►│  ──► {"log_type":"..."} │
│                           │         │  ("timestamp")   │     ├─────────────────────────┤
│ (Generated at compile     │         │ serialize_str    │     │ `toml`                  │
│  time from source code)   │         │  ("10:45:02")    ├────►│  ──► log_type = "..."   │
└───────────────────────────┘         └──────────────────┘     ├─────────────────────────┤
                                                               │ `bincode` (Binary)      │
                                                               │  ──► [01 00 08 31 30..] │
                                                               └─────────────────────────┘
```

| Crate | Role in [`../../../../Cargo.toml`](../../../../Cargo.toml) | What it actually contains | Does it know JSON syntax? |
| :--- | :--- | :--- | :--- |
| **`serde`** | `serde = { version = "1.0.229", features = ["derive"] }` | The `Serialize` and `Deserialize` traits + the `#[derive(...)]` macro compiler plugin. | **No.** It does not know what `{`, `}`, `:`, or `"` are. |
| **`serde_json`** | `serde_json = "1.0"` | The JSON Formatter & Parser (`to_string`, `to_writer`, `from_str`, `from_reader`). | **Yes.** It turns `serde` events into RFC 8259 JSON bytes. |

---

## 3. How `#[derive(Serialize, Deserialize)]` Works Under the Hood

You do **not** write `impl Serialize for LogEntry` by hand. When you place `#[derive(Serialize, Deserialize)]` above `pub struct LogEntry` in [`../../../../src/lib.rs`](../../../../src/lib.rs), the `serde_derive` compiler plugin reads your struct definition *before* `rustc` erases the field names and automatically expands ~80 lines of zero-cost Rust code:

```text
Title: What #[derive(Serialize)] Generates Behind the Scenes
Read: Left is what you write in lib.rs; Right is what the compiler sees.

YOU WRITE (5 Lines):                      COMPILER GENERATES AUTOMATICALLY:
────────────────────                      ─────────────────────────────────
#[derive(Serialize)]                      impl serde::Serialize for LogEntry {
pub struct LogEntry {                         fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    pub log_type: LogCategory,                where S: serde::Serializer {
    pub log_level: LogLevel,                      let mut state = s.serialize_struct("LogEntry", 5)?;
    pub timestamp: String,                        state.serialize_field("log_type", &self.log_type)?;
    pub message: String,                          state.serialize_field("log_level", &self.log_level)?;
    pub metadata: Option<MetaData>,               state.serialize_field("timestamp", &self.timestamp)?;
}                                                 state.serialize_field("message", &self.message)?;
                                                  state.serialize_field("metadata", &self.metadata)?;
                                                  state.end()
                                              }
                                          }
```

### Why `LogEntry` Must Be Serialized Instead of `DrawerLogItem`

| Candidate Struct | Defined In | Can Derive `Serialize`? | Data Fidelity | Verdict |
| :--- | :--- | :--- | :--- | :--- |
| **`DrawerLogItem`** | `slint::include_modules!()` (`ui/types.slint`) | **No ❌** (Generated inside macro; cannot attach `#[derive(Serialize)]` without orphan wrapper). | **Lossy ❌** (`MetaData` dict is flattened into a preview string; numbers become strings). | **Rejected** (UI view model only). |
| **`LogEntry`** | [`../../../../src/lib.rs`](../../../../src/lib.rs) | **Yes ✅** (Native Rust struct owned by our crate). | **100% Lossless ✅** (Preserves `LogCategory`, `LogLevel`, and recursive `MetaData`). | **Selected** (Canonical domain record). |

---

## 4. Serde Attributes & The `#[serde(untagged)]` Enum Law

When serializing a Rust `enum` like `MetaData` in [`../../../../src/lib.rs`](../../../../src/lib.rs), Rust enums can hold data inside variants (`MetaData::Integer(42)`). By default, `serde` wraps every variant in a JSON object using the variant name as the key (**Externally Tagged**).

For `LogLevel` (`INFO`, `ERROR`) and `LogCategory` (`ApiCall`, `ToolExecution`), the default behavior is fine because they have no inner payload—they become simple strings like `"INFO"` and `"ToolExecution"`.

However, for `MetaData`, the default behavior creates deeply nested, ugly JSON (`{"Dict": {"task_id": {"Integer": 1}}}`). Adding **`#[serde(untagged)]`** instructs `serde` to serialize the inner value directly without the variant wrapper!

```text
Title: MetaData JSON Output — Default vs #[serde(untagged)]
Read: Compare the bloated default output (Left) with clean untagged output (Right).

RUST VALUE:
MetaData::Dict({ "task_id": MetaData::Integer(2), "tool": MetaData::String("sql") })

❌ WITHOUT #[serde(untagged)] (Default)       ✅ WITH #[serde(untagged)]
───────────────────────────────────────       ──────────────────────────
{                                             {
  "Dict": {                                     "task_id": 2,
    "task_id": { "Integer": 2 },                "tool": "sql"
    "tool": { "String": "sql" }               }
  }
}
```

| Serde Attribute | Where to Place It | Exact Behavior | Use Case in ColdWind |
| :--- | :--- | :--- | :--- |
| **`#[serde(untagged)]`** | Above `pub enum MetaData` | Strips enum variant names (`Bool`, `Integer`, `Dict`); serializes inner payload directly. | **Mandatory** on `MetaData` so JSON matches standard Python `dict` / `list` / `int`. |
| **`#[serde(rename_all = "...")]`** | Above `pub enum LogCategory` *(Optional)* | Converts `ToolExecution` $\rightarrow$ `"TOOL_EXECUTION"` if set to `"SCREAMING_SNAKE_CASE"`. | Optional cosmetic match with Python enum `.value`. |
| **`#[serde(skip_serializing_if = "Option::is_none")]`** | Above `pub metadata: Option<MetaData>` *(Optional)* | Omits `"metadata": null` from the JSON line when `metadata` is `None`. | Saves ~18 bytes per log line when metadata is empty. |
| **`#[serde(default)]`** | Above struct field | Uses `Default::default()` (`None` / `0` / `""`) if a field is missing when reading old files. | Forward-compatibility when reading older `.jsonl` files. |

### ⚠️ The Variant Ordering Law for `#[serde(untagged)]`

When **deserializing** (`serde_json::from_str`) an `#[serde(untagged)]` enum, `serde` has no variant tag in the JSON to tell it which variant to pick. Therefore, **`serde` tries each variant from top to bottom in the exact order they are declared in your Rust `enum`, stopping at the first variant that succeeds!**

```rust
// ✅ CORRECT ORDER in src/lib.rs:
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetaData {
    Bool(bool),                       // 1. Matches `true` / `false`
    Integer(i64),                     // 2. MUST come BEFORE Float(f64)! Matches `42`
    Float(f64),                       // 3. Matches `3.14` (If placed above Integer, `42` would become `42.0`!)
    String(String),                   // 4. Matches `"text"`
    List(Vec<MetaData>),              // 5. Matches `[...]`
    Dict(HashMap<String, MetaData>),  // 6. Matches `{...}`
}
```

---

## 5. Exact Implementation Blueprint for `src/lib.rs` (Part A)

Apply these exact additions to [`../../../../Cargo.toml`](../../../../Cargo.toml) and [`../../../../src/lib.rs`](../../../../src/lib.rs):

```rust
// In src/lib.rs:
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    DEBUG,
    ERROR,
    WARNING,
    INFO,
    CRITICAL,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogCategory {
    ApiCall,
    ToolExecution,
    AgentWorkflow,
    McpServer,
    ErrorTraceback,
    OTHER,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetaData {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<MetaData>),
    Dict(HashMap<String, MetaData>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    pub log_type: LogCategory,
    pub log_level: LogLevel,
    pub timestamp: String,
    pub message: String,
    pub metadata: Option<MetaData>,
}
```

---

## Q&A Log

| # | Question | Answer (concrete) | Date |
| :--- | :--- | :--- | :--- |
| **Q1** | Why can't the Rust compiler serialize `LogEntry` automatically when it already knows the struct's byte size? | Because `String` and `HashMap` fields only store 24-byte/48-byte heap pointers (`ptr, len, cap`) on the stack, not the actual text, and `rustc` erases all field names at compile time. | 2026-11-09 |
| **Q2** | Why do both `serde` and `serde_json` exist when JSON itself means serialization? | `serde` is format-agnostic (it extracts struct fields at compile time via `#[derive]`), while `serde_json` is the JSON driver that formats those extracted fields into UTF-8 JSON syntax. | 2026-11-09 |
| **Q3** | Do I have to write `impl Serialize for LogEntry` manually like we wrote `impl Display`? | No. Adding `#[derive(Serialize, Deserialize)]` auto-generates the entire trait implementation at compile time with zero runtime reflection overhead. | 2026-11-09 |
| **Q4** | What happens if I forget `#[serde(untagged)]` on `MetaData`? | Every value gets wrapped in its Rust enum variant name in JSON (e.g., `{"Dict": {"task_id": {"Integer": 1}}}` instead of `{"task_id": 1}`). | 2026-11-09 |
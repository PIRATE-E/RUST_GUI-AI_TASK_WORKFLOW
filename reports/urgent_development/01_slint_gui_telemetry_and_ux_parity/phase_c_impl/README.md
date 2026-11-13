# Phase C: Telemetry Disk Persistence & ACID Cold Replay

| Field | Value |
| :--- | :--- |
| **Domain** | `phase-c-disk-persistence` |
| **Parent Index** | [`../README.md`](../README.md) |
| **Source Targets** | [`../../../../src/lib.rs`](../../../../src/lib.rs) · [`../../../../src/main.rs`](../../../../src/main.rs) · [`../../../../Cargo.toml`](../../../../Cargo.toml) |
| **Doc Count** | 3 Technical Reports + 1 Sub-domain README |
| **Verification Mode** | Codebase + Systems Research (`VERIFIED` ✅) |
| **Updated** | 2026-11-09 |

> **TL;DR:** Phase C decouples live RAM telemetry from historical disk storage. Thread 3 serializes domain `LogEntry` structs (never lossy UI strings) into an append-only `logs/<ticket_id>.swap` file via an 8 KB `BufWriter`, atomically renames it to `logs/<ticket_id>.jsonl` via `fsync` + `rename(2)` when `is_generating` turns `false`, and streams historical turns on demand via `BufReader` without stalling the 144 FPS Slint UI thread.

---

## At a glance

| Doc | Type | What it answers | Fused context & key mechanism | Status |
| :--- | :--- | :--- | :--- | :--- |
| [`./01-serde-serialization-architecture.md`](./01-serde-serialization-architecture.md) | **Concept Note** | Why can't Rust raw-dump structs, why are `serde` and `serde_json` separate, and why does `MetaData` need `#[serde(untagged)]`? | Rust erases field names at compile time and stores heap pointers (`ptr/len/cap`). `serde` generates AST visitor code; `serde_json` formats UTF-8 JSON. | `VERIFIED` ✅ |
| [`./02-acid-swap-file-ingestion.md`](./02-acid-swap-file-ingestion.md) | **Design / Decision** | How do we stream `LogEntry` to `.swap` files and atomically commit to `.jsonl` using Linux syscalls? | Maps Python `open()` to `OpenOptions` + `File` (`fd`) + `BufWriter`. Uses `flush()` (`write(2)`) + `sync_all()` (`fsync(2)`) + `std::fs::rename` (`renameat2(2)`). | `VERIFIED` ✅ |
| [`./03-cold-replay-and-edge-cases.md`](./03-cold-replay-and-edge-cases.md) | **Design / Investigation** | How do we load finished tickets from disk on `[>]` click at 144 FPS and handle crashed `.swap` leftovers? | Streams line-by-line via `BufReader::lines()`, maps `LogEntry` to `DrawerLogItem`, and tolerates truncated final lines in crashed `.swap` files. | `VERIFIED` ✅ |

---

## Architecture Map

```text
Title: Phase C End-to-End Persistence & Replay Topology
Read: Follow Thread 3 (Left) for live ACID writes and Thread 0 (Right) for cold reads.

                     [ Python Core Emits Log ]
                                 │
                                 ▼ (PyO3 FromPyObject)
                     ┌───────────────────────┐
                     │   LogEntry (lib.rs)   │
                     └───────────┬───────────┘
                                 │
          ┌──────────────────────┴──────────────────────┐
          ▼ (1. UI Path: DrawerLogItem::from)           ▼ (2. Disk Path: serde_json::to_writer)
┌───────────────────────────┐                 ┌───────────────────────────────┐
│ Arc<Mutex<Vec<Item>>>     │                 │ BufWriter<File> (8 KB RAM)    │
│ + Live VecModel.push()    │                 │   └──► logs/<id>.swap         │
└───────────────────────────┘                 └───────────────┬───────────────┘
                                                              │
                                              [Turn Ends: is_generating = false]
                                                              │
                                                              ▼
                                              ┌───────────────────────────────┐
                                              │ 1. writer.flush()   [write]   │
                                              │ 2. file.sync_all()  [fsync]   │
                                              │ 3. fs::rename()     [atomic]  │
                                              └───────────────┬───────────────┘
                                                              │
                                                              ▼
┌───────────────────────────┐                 ┌───────────────────────────────┐
│ User Clicks [>] on Old    │ ──(Reads)─────► │ logs/<id>.jsonl (Verified)    │
│ Bubble (is_generating: 0) │                 │   └──► BufReader::lines()     │
└───────────────────────────┘                 └───────────────────────────────┘
```

---

## Reading path

| Order | File | Who should read this | Prerequisite |
| :--- | :--- | :--- | :--- |
| **1** | [`./01-serde-serialization-architecture.md`](./01-serde-serialization-architecture.md) | Anyone touching [`../../../../src/lib.rs`](../../../../src/lib.rs) or adding fields to `LogEntry` / `MetaData`. | Basic Rust structs & enums. |
| **2** | [`./02-acid-swap-file-ingestion.md`](./02-acid-swap-file-ingestion.md) | Systems engineers wiring Thread 3 disk streaming in [`../../../../src/main.rs`](../../../../src/main.rs). | Report 01 (`Serialize` trait). |
| **3** | [`./03-cold-replay-and-edge-cases.md`](./03-cold-replay-and-edge-cases.md) | UI/Systems engineers wiring `app.on_inspect_ticket` (`else` branch) in [`../../../../src/main.rs`](../../../../src/main.rs). | Reports 01 & 02 (`Deserialize` + `.swap`). |

---

## Key takeaways

| Principle | Rule enforced in ColdWind | Why it matters |
| :--- | :--- | :--- |
| **Domain over Presentation** | Serialize `LogEntry`, never `DrawerLogItem`. | Preserves typed integers, floats, and nested dicts for analytics; `DrawerLogItem` is a lossy UI projection. |
| **Zero-Allocation Streaming** | Use `serde_json::to_writer(&mut buf_writer, &log)` instead of `to_string()`. | Writes UTF-8 JSON bytes directly into the 8 KB `BufWriter` buffer without allocating a temporary `String` on every log. |
| **ACID Swap Isolation** | Active turns write to `<ticket_id>.swap`; only completed turns become `<ticket_id>.jsonl`. | Guarantees zero ambiguity between an in-flight/crashed turn and a verified completed turn. |
| **Kernel vs Userspace Flush** | Pair `writer.flush()` with `writer.get_ref().sync_all()` before `std::fs::rename`. | Prevents zero-length or half-written `.jsonl` files if the OS crashes while data is still sitting in the Linux Page Cache. |
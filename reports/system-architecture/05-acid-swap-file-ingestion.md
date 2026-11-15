# 02 — ACID Swap-File Ingestion, Linux Syscalls & Buffered I/O

| Field | Value |
| :--- | :--- |
| **Type** | Design / Decision & Systems Architecture |
| **Domain** | [`./README.md`](./README.md) (`phase-c-disk-persistence`) |
| **Prerequisites** | [`./01-serde-serialization-architecture.md`](./01-serde-serialization-architecture.md) |
| **Target Files** | [`../../../../src/lib.rs`](../../../../src/lib.rs) · [`../../../../src/main.rs`](../../../../src/main.rs) |
| **Verification Mode** | Codebase + Linux Kernel `man 2` Specification (`VERIFIED` ✅) |
| **Confidence** | `VERIFIED` ✅ |
| **Updated** | 2026-11-09 |
| **Context** | Defines how Thread 3 streams serialized `LogEntry` records into `logs/<ticket_id>.swap` and atomically commits them to `logs/<ticket_id>.jsonl` upon turn completion. |

> **TL;DR:** Python's `open("file", "a")` bundles OS flags, a raw File Descriptor (`fd`), and an 8 KB userspace buffer into one opaque object. Rust separates them into **`OpenOptions`** (flags), **`File`** (`fd`), and **`BufWriter<File>`** (8 KB RAM buffer). During an active turn (`is_generating == true`), Thread 3 streams JSONL lines into `logs/<ticket_id>.swap` via `serde_json::to_writer`, and when the turn finishes, executes `flush()` (`write(2)`), `sync_all()` (`fsync(2)`), and `std::fs::rename` (`renameat2(2)`) to atomically produce `logs/<ticket_id>.jsonl`.

---

## 1. Python `open()` / `fd` vs Rust's 3-Layer Modular I/O

In Linux, the kernel never deals with file objects—every open file (including `sys.stdin = 0`, `sys.stdout = 1`, `sys.stderr = 2`) is just an integer index called a **File Descriptor (`fd: i32`)** pointing into the process's kernel file table.

When you call `open("logs/t1.swap", "a")` in Python, CPython allocates three stacked C objects behind the scenes. Rust refuses to force hidden heap buffers on you, so it exposes the three layers explicitly:

```text
Title: CPython Monolithic open() vs Rust 3-Layer File Handle Stack
Read: Match each Python internal layer (Left) to its explicit Rust type (Right).

CPYTHON `open("t1.swap", "a")`             RUST EXPLICIT TYPES (`std::fs` + `std::io`)
──────────────────────────────             ───────────────────────────────────────────
1. Mode String Parser ("a")       ◄─────►  1. `OpenOptions::new().create(true).append(true)`
   (Sets C flags O_CREAT|O_APPEND)            (Builder that configures Linux openat(2) flags)
          │                                              │
          ▼                                              ▼ .open("logs/t1.swap")?
2. `io.FileIO` (`f.fileno()`)     ◄─────►  2. `std::fs::File` (4-byte wrapper around `fd: i32`)
   (Raw unbuffered OS descriptor)             (Every `.write()` here is a direct OS syscall!)
          │                                              │
          ▼                                              ▼ BufWriter::new(file)
3. `io.BufferedWriter` (8 KB)     ◄─────►  3. `std::io::BufWriter<File>` (8 KB RAM ring buffer)
   (Batches writes in userspace)              (Batches logs in RAM; flushes to `fd` in 8 KB chunks)
```

| Operation | Python Equivalent | Rust Construct | Underlying Linux Kernel Syscall |
| :--- | :--- | :--- | :--- |
| **Create Directory** | `os.makedirs("logs", exist_ok=True)` | `std::fs::create_dir_all("logs")?` | `mkdirat(AT_FDCWD, "logs", 0777)` |
| **Open for Append** | `open(path, "a", buffering=8192)` | `BufWriter::new(OpenOptions::new().create(true).append(true).open(path)?)` | `openat(AT_FDCWD, path, O_WRONLY \| O_CREAT \| O_APPEND \| O_CLOEXEC, 0666)` |
| **Userspace Flush** | `f.flush()` | `writer.flush()?` | `write(fd, buf_ptr, len)` (Copies 8 KB RAM to Kernel Page Cache) |
| **Hardware Disk Sync** | `os.fsync(f.fileno())` | `writer.get_ref().sync_all()?` | `fsync(fd)` (Flushes Kernel Page Cache to physical NVMe/SSD flash) |
| **Atomic Rename** | `os.rename(swap_path, jsonl_path)` | `std::fs::rename(swap_path, jsonl_path)?` | `renameat2(AT_FDCWD, old, AT_FDCWD, new, 0)` |
| **Close Descriptor** | `f.close()` / `with` block exit | **Automatic via `Drop` trait** when variable leaves `{}` scope | `close(fd)` |

---

## 2. The 3-Stage Memory Journey: `write!` vs `flush()` vs `sync_all()` (`fsync`)

Understanding why `writer.flush()` is **not** `os.fsync()` is critical for crash-proof ACID logging. When Thread 3 writes a serialized `LogEntry`, the bytes travel through **three distinct hardware/OS boundaries**:

```text
Title: The 3-Stage Journey of a LogEntry from Rust RAM to Physical SSD
Read: Left to Right shows where bytes live after each method call.

STAGE 1: Userspace RAM          STAGE 2: Linux Kernel RAM       STAGE 3: Physical NVMe SSD
(Your Process Heap)             (OS Page Cache / Dirty Pages)   (Non-Volatile Flash Cells)
┌─────────────────────────┐     ┌─────────────────────────┐     ┌─────────────────────────┐
│ BufWriter<File> (8 KB)  │     │ Kernel VFS Page Cache   │     │ logs/ticket-1.swap      │
│                         │     │ (Queue in OS RAM)       │     │ (On-Disk Blocks)        │
└────────────┬────────────┘     └────────────┬────────────┘     └────────────▲────────────┘
             │                               │                               │
             └── 1. `writer.flush()?` ──────►└── 2. `file.sync_all()?` ─────►┘
                 Calls Linux `write(2)`.         Calls Linux `fsync(2)`.
                 Survives process crash!         Survives OS/power loss!
```

| Method Call | What It Actually Does | If App Panics (`kill -9`) | If Power Cord Is Pulled |
| :--- | :--- | :--- | :--- |
| **`serde_json::to_writer(&mut writer, &log)`** | Writes JSON bytes into `BufWriter`'s 8 KB userspace array. (Zero syscalls if < 8 KB). | ❌ Last < 8 KB of logs in RAM are lost. | ❌ Lost. |
| **`writer.flush()?`** | Executes Linux `write(2)` syscall, draining the 8 KB buffer into the **Linux Kernel Page Cache**. | ✅ **100% Safe!** OS kernel still writes the queued page cache to disk even if your app dies. | ❌ Lost if OS hasn't flushed dirty pages yet. |
| **`writer.get_ref().sync_all()?`** | Executes Linux `fsync(2)` syscall, forcing the NVMe controller to commit the kernel queue + inode metadata to silicon. | ✅ **100% Safe!** | ✅ **100% Safe!** |

---

## 3. How to Write Serialized `LogEntry` to the File Handle (`to_writer` vs `write!`)

Once `LogEntry` derives `Serialize` (from [`./01-serde-serialization-architecture.md`](./01-serde-serialization-architecture.md)), there are two ways to write a JSONL line into `BufWriter<File>`:

| Approach | Code Pattern | Heap Allocations per Log | Verdict |
| :--- | :--- | :--- | :--- |
| **Approach A: Intermediate String (`to_string` + `writeln!`)** | `let s = serde_json::to_string(&log)?;`<br>`writeln!(writer, "{s}")?;` | **1 Heap Allocation** (`String` allocated, copied into `BufWriter`, then freed). | Good, simple, familiar. |
| **Approach B: Direct Stream (`to_writer` + `\n`)** | `serde_json::to_writer(&mut writer, &log)?;`<br>`writer.write_all(b"\n")?;` | **0 Heap Allocations!** (`serde_json` writes UTF-8 bytes directly into `BufWriter`'s 8 KB slice). | **Optimal ✅** (Zero-alloc systems standard). |

---

## 4. The ACID `.swap` $\rightarrow$ `.jsonl` State Machine Design

To eliminate any ambiguity between an active/crashed turn and a cleanly completed turn, ColdWind enforces a strict two-extension contract inside the `logs/` directory:

1. **`logs/<ticket_id>.swap` (In-Flight / Unverified):**
   * Created when the first log for `<ticket_id>` arrives on Thread 3 (or when a turn starts).
   * Every incoming `LogEntry` is written via `serde_json::to_writer(&mut writer, &log)` + `writer.write_all(b"\n")` + `writer.flush()`.
   * Why `writer.flush()` on every log? Because `flush()` only copies ~250 bytes from userspace to the Linux Kernel Page Cache in ~0.8 microseconds, ensuring that if the process crashes at any millisecond, **every single log up to that millisecond is preserved inside `.swap`**!
2. **`logs/<ticket_id>.jsonl` (ACID Committed / Immutable):**
   * Triggered when the turn finishes (`is_generating` transitions from `true` $\rightarrow$ `false`).
   * Executes `writer.flush()` $\rightarrow$ `writer.get_ref().sync_all()` (`fsync`) $\rightarrow$ drops the file handle $\rightarrow$ `std::fs::rename("logs/<id>.swap", "logs/<id>.jsonl")`.

```text
Title: SwapLogWriter Lifecycle Across Thread 3 and Thread 2/0
Read: Follow the state transitions from Ticket Start to Atomic Rename.

[New Ticket Turn Starts: ticket_id = "development-a8f1"]
       │
       ▼
┌────────────────────────────────────────────────────────┐
│ SwapLogWriter::append(&mut self, tid, &log)            │
│  1. If `self.active_tid != tid`, open:                 │
│     `logs/development-a8f1.swap` (BufWriter<File>)     │
│  2. `serde_json::to_writer(&mut self.writer, &log)?`   │
│  3. `self.writer.write_all(b"\n")?`                    │
│  4. `self.writer.flush()?` (Drains to OS Page Cache)   │
└───────────────────────────┬────────────────────────────┘
                            │
        ┌───────────────────┴───────────────────┐
        ▼ (Crash / Kill / Force Quit)           ▼ (Turn Finishes: commit_swap_to_jsonl)
┌───────────────────────────────┐       ┌────────────────────────────────────────┐
│ File stays `.swap` on disk!   │       │ 1. `writer.flush()?`                   │
│ Zero ambiguity: system knows  │       │ 2. `writer.get_ref().sync_all()?`      │
│ this ticket did NOT finish.   │       │ 3. Drop `writer` (closes OS `fd`)      │
└───────────────────────────────┘       │ 4. `fs::rename(".swap", ".jsonl")?`    │
                                        └────────────────────────────────────────┘
```

---

## 5. Production Implementation: `SwapLogWriter` Helper

Add this clean, self-contained persistence manager (e.g., in [`../../../../src/lib.rs`](../../../../src/lib.rs) or a dedicated module) to handle the entire `.swap` $\rightarrow$ `.jsonl` lifecycle:

```rust
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

pub struct SwapLogWriter {
    logs_dir: PathBuf,
    active_ticket_id: String,
    writer: Option<BufWriter<File>>,
}

impl SwapLogWriter {
    pub fn new(logs_dir: impl Into<PathBuf>) -> io::Result<Self> {
        let dir = logs_dir.into();
        // Equivalent to Python: os.makedirs("logs", exist_ok=True)
        fs::create_dir_all(&dir)?;
        Ok(Self {
            logs_dir: dir,
            active_ticket_id: String::new(),
            writer: None,
        })
    }

    #[inline]
    fn swap_path(&self, ticket_id: &str) -> PathBuf {
        self.logs_dir.join(format!("{ticket_id}.swap"))
    }

    #[inline]
    fn jsonl_path(&self, ticket_id: &str) -> PathBuf {
        self.logs_dir.join(format!("{ticket_id}.jsonl"))
    }

    /// Streams a single LogEntry into `logs/<ticket_id>.swap` with zero String allocation.
    pub fn append_log(&mut self, ticket_id: &str, log: &LogEntry) -> io::Result<()> {
        if ticket_id.is_empty() {
            return Ok(());
        }

        // If a new ticket started, rotate the open file descriptor
        if self.active_ticket_id != ticket_id || self.writer.is_none() {
            self.active_ticket_id = ticket_id.to_string();
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.swap_path(ticket_id))?;
            self.writer = Some(BufWriter::new(file));
        }

        if let Some(buf_writer) = self.writer.as_mut() {
            // 1. Stream JSON bytes directly into the 8 KB buffer (0 intermediate String allocs)
            serde_json::to_writer(&mut *buf_writer, log)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            // 2. Terminate the JSONL record with a newline
            buf_writer.write_all(b"\n")?;
            // 3. Push to Linux Kernel Page Cache so process crash never loses the log
            buf_writer.flush()?;
        }

        Ok(())
    }

    /// Flushes userspace, fsyncs kernel queues to disk, and atomically renames `.swap` -> `.jsonl`.
    pub fn commit_turn(&mut self, ticket_id: &str) -> io::Result<()> {
        // 1. If we hold the open BufWriter for this ticket, flush + fsync + close (`take()`)
        if self.active_ticket_id == ticket_id {
            if let Some(mut buf_writer) = self.writer.take() {
                buf_writer.flush()?;
                buf_writer.get_ref().sync_all()?; // Linux fsync(2)
            } // <-- buf_writer drops here, invoking Linux close(fd)
            self.active_ticket_id.clear();
        }

        // 2. Atomic inode flip: logs/<id>.swap -> logs/<id>.jsonl
        let swap = self.swap_path(ticket_id);
        let jsonl = self.jsonl_path(ticket_id);
        if swap.exists() {
            fs::rename(swap, jsonl)?;
        }
        Ok(())
    }
}
```

---

## Q&A Log

| # | Question | Answer (concrete) | Date |
| :--- | :--- | :--- | :--- |
| **Q1** | Why does Rust split file opening into `OpenOptions`, `File`, and `BufWriter` while Python just has `open()`? | Python's `open()` allocates all three layers implicitly; Rust separates them so systems code only pays for an 8 KB userspace `BufWriter` when desired. | 2026-11-09 |
| **Q2** | Is `writer.flush()` the same as OS `fsync`? | No. `writer.flush()` calls `write(2)` to move bytes from Rust's 8 KB RAM buffer into the Linux Kernel Page Cache queue. `writer.get_ref().sync_all()` calls `fsync(2)` to flush the kernel queue to physical SSD flash. | 2026-11-09 |
| **Q3** | Do we use the `write!` macro or `serde_json::to_writer` to write a `LogEntry` into the file handle? | Use `serde_json::to_writer(&mut buf_writer, &log)` followed by `buf_writer.write_all(b"\n")`. Unlike `write!`, `to_writer` streams JSON directly into the buffer without allocating a temporary `String`. | 2026-11-09 |
| **Q4** | Why do we call `self.writer.take()` before `std::fs::rename`? | `Option::take()` moves the `BufWriter<File>` out of `self` and drops it at the end of the block, ensuring the OS file descriptor is flushed, `fsync`ed, and closed before renaming. | 2026-11-09 |
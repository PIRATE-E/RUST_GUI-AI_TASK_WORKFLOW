---
name: notebooklm-rag
description: Complete programmatic API & Agent Skill for Google NotebookLM (Gemini Notebook). Use PROACTIVELY for managing notebooks, bulk uploading sources (files, URLs, YouTube, Google Drive), deep research, zero-token grounded RAG queries, generation (audio, video, slides, quiz, flashcards, mind maps, reports), downloads, notes, labels, collections, and MCP integration.
---

# 📓 NotebookLM Complete Agentic RAG & Automation Skill

Full programmatic access to Google NotebookLM (Gemini Notebook) via `notebooklm-py` CLI and Python API—including capabilities not exposed in the web UI.

---

## 🏗️ 1. Architecture Overview (Two-Stage Agentic RAG)

```
┌────────────────────────────────────────────────────────┐
│ 1. SOURCE COMPILER & EXTRACTOR (Antigravity Agent)     │
│    - Web scraping, YouTube transcripts, local notes     │
│    - Batch upload via CLI / Python SDK                 │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│ 2. GROUNDED KNOWLEDGE ENGINE (NotebookLM Server-Side)  │
│    - Server-side document analysis & zero-token RAG    │
│    - Cited answers strictly grounded in uploaded docs  │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│ 3. INTERACTIVE TUTOR & VISUAL DESIGNER (Antigravity)   │
│    - 5-Phase Teaching Protocol (AGENTS.md)             │
│    - Visual diagrams (ASCII / Mermaid) & guided hints  │
└───────────────────────────┴────────────────────────────┘
```

---

## 🛠️ 2. Agent Setup & Authentication Verification

### Verification Rule
Before starting any workflow, verify authentication:
```bash
notebooklm auth check --test --json
```
Expect `"status": "ok"` AND `"checks.token_fetch": true`. (Bare `auth check` without `--test` is a false-positive trap).

### Context Isolation in Automation
In automated/parallel workflows, **never** rely solely on `notebooklm use <id>` (which mutates global state). Always pass the explicit notebook flag: `-n <notebook_id>` or `--notebook <notebook_id>`.

---

## 🚦 3. Autonomy Rules

### Run Automatically (No Confirmation Needed)
- Context & Info: `notebooklm status`, `notebooklm auth check`, `notebooklm auth inspect`, `notebooklm doctor`
- Lists: `notebooklm list`, `source list`, `artifact list`, `label list`, `collection list`, `note list`, `language list`
- Queries & Prompts: `notebooklm ask "..."` (without `--save-as-note`), `suggest-prompts`, `history`
- Non-destructive creation: `notebooklm create`, `source add`, `source add-research`, `label create`, `collection create`
- Language settings: `notebooklm language get`, `notebooklm language set`

### Ask Before Running (Requires Confirmation)
- Destructive actions: `delete`, `source delete`, `source delete-by-title`, `source clean`, `note delete`, `artifact delete`, `label delete`, `collection delete`, `auth logout`, `clear`, `ask --new` (pass `-y/--yes` once approved).
- Long-running generation: `generate audio`, `generate video`, `generate slide-deck`, `generate report`, `generate quiz`, `generate flashcards`, `generate infographic`, `generate data-table`, `generate mind-map`.
- Writing to disk: `download *`

---

## 📚 4. Exhaustive Command Reference

### A. Session & Auth Commands
```bash
notebooklm login                                    # Interactive OAuth login
notebooklm login --browser-cookies brave            # Import cookies from Brave
notebooklm auth check --test --json                 # Validate network authentication
notebooklm auth refresh                             # Refresh active session cookies
notebooklm auth logout                              # Clear saved credentials
```

### B. Notebook Management Commands
```bash
notebooklm list --json                              # List notebooks in JSON
notebooklm create "Title" --use                     # Create notebook and set as active context
notebooklm use <id>                                 # Set active working notebook
notebooklm rename "New Title"                       # Rename current notebook
notebooklm delete -n <id> -y                        # Delete notebook without prompt
notebooklm summary --topics                         # Get AI summary & topic overview
notebooklm metadata -n <id> --json                  # Export full notebook metadata
```

### C. Source Ingestion & Research Commands
```bash
notebooklm source list -n <id>                      # List all sources in notebook
notebooklm source add "https://url.com" -n <id>     # Add web URL or local file (.md, .pdf)
notebooklm source add-drive <drive_id> "Title"      # Add Google Drive doc/slide/sheet
notebooklm source add-research "Topic" --mode deep  # Automated deep web research
notebooklm source fulltext <src_id> -f markdown     # Extract fulltext as markdown
notebooklm source wait <src_id> --timeout 300       # Wait for source indexing completion
notebooklm source clean -n <id> -y                  # Clean duplicate/broken sources
notebooklm source delete <src_id> -y                # Delete specific source
```

### D. Label & Collection Organization
```bash
# Labels (Sources within a notebook)
notebooklm label create "Trees" --emoji 🌲 -n <id>
notebooklm label add "Trees" <src_id1> <src_id2> -n <id>
notebooklm label generate --scope unlabeled -n <id>

# Collections (Group whole notebooks)
notebooklm collection create "DSA Mastery"
notebooklm collection add "DSA Mastery" <nb_id1> <nb_id2>
```

### E. Zero-Token RAG Chat & Configuration
```bash
notebooklm ask "Explain Big-O notation"             # Query grounded notebook sources
notebooklm ask -c <conv_id> "Follow up question"   # Continue conversation thread
notebooklm ask "Summary" --save-as-note            # Query and save response as note
notebooklm configure --mode learning-guide          # Set predefined chat persona
notebooklm suggest-prompts --mode 4                 # Get AI-suggested chat prompts
```

### F. Multi-Format Artifact Generation
```bash
notebooklm generate audio "Focus on recursion" --format deep-dive --wait
notebooklm generate video "Explainer for kids" --format explainer --wait
notebooklm generate slide-deck "DSA Summary" --format detailed --wait
notebooklm generate quiz --difficulty hard --quantity standard --wait
notebooklm generate flashcards --difficulty medium --wait
notebooklm generate infographic --style scientific --wait
notebooklm generate mind-map --kind interactive
notebooklm generate report "Comprehensive Guide" --format study-guide --wait
```

### G. Downloads & Exports
```bash
notebooklm download audio ./audio_summary.mp3
notebooklm download slide-deck ./slides.pdf --format pdf
notebooklm download quiz ./quiz.md --format markdown
notebooklm download flashcards ./cards.json --format json
notebooklm download mind-map ./map.json
notebooklm download report ./study_guide.md
```

---

## 🐍 5. Production Python SDK Automation Scripts

### Script 1: Bulk Directory Source Ingestion
```python
#!/usr/bin/env python3
"""Bulk upload local notes and documents into NotebookLM."""

import subprocess
from pathlib import Path

def bulk_upload_directory(directory_path: str, notebook_id: str):
    """Upload all markdown files from a local directory into a target notebook."""
    folder = Path(directory_path)
    files = list(folder.glob("**/*.md"))
    print(f"📁 Found {len(files)} markdown files in {directory_path}")
    
    for file_path in files:
        print(f"🚀 Uploading {file_path.name}...")
        cmd = ["notebooklm", "source", "add", str(file_path), "-n", notebook_id]
        res = subprocess.run(cmd, capture_output=True, text=True)
        if res.returncode == 0:
            print(f"  ✅ Uploaded: {file_path.name}")
        else:
            print(f"  ❌ Error: {res.stderr.strip()}")

if __name__ == "__main__":
    # Example usage
    bulk_upload_directory("00_fundamentals", notebook_id="YOUR_NOTEBOOK_ID")
```

### Script 2: Automated Multi-Format Artifact Pipeline
```python
#!/usr/bin/env python3
"""Generate and download study artifacts in batch."""

import subprocess
import json

def build_topic_artifacts(notebook_id: str, output_dir: str):
    """Generate study guide, quiz, and mind-map for a notebook."""
    out_path = Path(output_dir)
    out_path.mkdir(parents=True, exist_ok=True)
    
    print("📜 Generating Study Guide Report...")
    subprocess.run([
        "notebooklm", "generate", "report", "Mastery Summary",
        "--format", "study-guide", "--wait", "-n", notebook_id
    ])
    subprocess.run([
        "notebooklm", "download", "report", str(out_path / "study_guide.md"),
        "-n", notebook_id
    ])
    
    print("🧩 Generating Hard Practice Quiz...")
    subprocess.run([
        "notebooklm", "generate", "quiz", "--difficulty", "hard",
        "--wait", "-n", notebook_id
    ])
    subprocess.run([
        "notebooklm", "download", "quiz", str(out_path / "quiz.md"),
        "--format", "markdown", "-n", notebook_id
    ])
    
    print("🧠 Generating Interactive Mind Map...")
    subprocess.run([
        "notebooklm", "generate", "mind-map", "--kind", "interactive",
        "-n", notebook_id
    ])
    subprocess.run([
        "notebooklm", "download", "mind-map", str(out_path / "mind_map.json"),
        "-n", notebook_id
    ])
    print(f"✨ Artifact pipeline complete. Saved to {output_dir}")

if __name__ == "__main__":
    build_topic_artifacts(notebook_id="YOUR_NOTEBOOK_ID", output_dir="00_fundamentals/artifacts")
```

---

## 🎯 6. Best Practices for AI Agents

1. **Zero-Token Memory Layer**: Use `notebooklm ask` for accurate, cited answers from uploaded sources instead of loading large text files directly into the active prompt window.
2. **Deterministic Context**: In scripts or multi-step tasks, always pass `-n <id>` to guarantee operations target the intended notebook.
3. **Structured Outputs**: Use `--json` on CLI calls (`notebooklm list --json`, `ask --json`, `artifact list --json`) for easy programmatic parsing.

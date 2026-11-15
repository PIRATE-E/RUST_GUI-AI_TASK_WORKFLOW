# 📜 Master Guide & Cheat Sheet Law — The Gold-Standard Authoring Rules for `learn_rust` (v1.0)

> **What this file is:** THE canonical template + rulebook for every master guide and cheat sheet in `learn_rust`. Any AI agent or human who authors, audits, or repairs one of these sheets MUST obey this document before touching a single line. Its purpose: **all sheets at the same tier must look identical** — same sections, same order, same headers — so the learner builds *muscle memory* (eyes always know where to look after months away) and any maintenance AI knows *the one obvious home for every fact*.
> **Why this law exists:** The audit (`facts.md`) proved the sheets drift by generation — Topic 00/01 fossils at 3.5/12, the Topic 04 gold guide at 10/12, nothing chained the format forward or backfilled the past. Structure is rigid and hard to change later — that rigidity is only acceptable if the skeleton is *complete* up front. This law is that complete skeleton. Nothing important is left homeless.
> **What makes this law different from every other cheat-sheet law:** it is bound to **`CONCEPT_MANIFEST.md`**. Every line of every sheet must be verifiable against the manifest's CUMULATIVE AVAILABLE list for the sheet's topic; every future-topic concept must carry an explicit `⏳ Deferred until Topic N` marker. A sheet that teaches beautifully but leaks Topic 08 syntax into Topic 06 is **defective by definition**.
>
> **Precedence order:** `AGENTS.md` (constitution) → **this file** (sheet law) → `context.md` (per-file work order) → `facts.md` (reference only). On conflict, ask the human; never guess.

---

## 🌳 PART 1 — The Three-Tier Sheet Tree

Every reference sheet lives at exactly ONE tier. **One fact, one home**: every piece of information is OWNED by exactly one tier; all other tiers may only LINK to it, never duplicate it.

```text
learn_rust Reference Sheet Tree
================================================================================

TIER 00 — ROOT PROGRESS DASHBOARD  (README.md at workspace root)
   Job: Progress truth (topics, tests, notes, milestones) + the discovery
        index (links to AGENTS.md, CONCEPT_MANIFEST.md, reports/).
   Never holds: teaching content of any kind.
   │
   ├── TIER 0 — TOPIC MASTER GUIDE  (one per topic: <topic>/notes/00_*_master_guide.md)
   │     Job: WHOLE-TOPIC re-learning entry point. "Which domain does this
   │          concept belong to, and what is the Python↔Rust paradigm shift?"
   │     Holds: 30s topic tree (routes to DOMAINS), topic Rosetta Stone,
   │          topic-level invariant matrix, learning curve, compiler map,
   │          domain registry, deferral map.
   │     Never holds: syntax templates, trap archives, deep derivations.
   │     │
   │     ├── TIER 1 — CHEAT SHEET  (war chest, one per concept family)
   │     │     Two homes, same skeleton:
   │     │       • Cross-cutting:  patterns/cheat_sheets/NN_<name>_cheat_sheet.md
   │     │       • In-domain:     <topic>/notes/<domain>/cheat_sheet.md
   │     │     Job: SELF-CONTAINED WAR CHEST for ONE concept family.
   │     │          "Which exact syntax/pattern, and what breaks if I'm wrong?"
   │     │     Holds: decision tree, invariant matrix, syntax bank, memory
   │     │            model, compiler diagnostics, trap archive, sources, links.
   │     │     │
   │     │     └── TIER 2 — CONCEPT NOTES  (one per concept: notes/<domain>/0X_*.md)
   │     │           Job: DEEP-DIVE derivation for ONE concept — the lesson
   │     │                itself, where the AI teaches phase-by-phase.
   │     │           Governed by AGENTS.md §2 (📺 video sources, Invariant/Target
   │     │           Matrix, Learner Pitfall Archive). Sheets LINK to these;
   │     │           they NEVER duplicate their derivations.
   │     │
   │     └── (Tier 0 also routes to sibling domains via its Domain Registry)
   └──
```

### 1.1 The Ownership Matrix — what lives WHERE

| Content | Tier 0 Master Guide | Tier 1 Cheat Sheet | Tier 2 Concept Note |
| :--- | :---: | :---: | :---: |
| Topic-level Python↔Rust paradigm (Rosetta Stone) | ✅ OWNS | domain delta only | — |
| 30s topic tree (prompt → DOMAIN routing) | ✅ OWNS | 30s tree (prompt → exact syntax) | — |
| Topic-level invariant matrix (§2.3 at topic scale) | ✅ OWNS | family-level matrix | derivation |
| Learning curve across the topic's domains | ✅ OWNS | mini progression of the family | — |
| Compiler enforcement map (topic's error codes) | ✅ OWNS (survey) | ✅ OWNS (exact diagnostics table) | full error walkthroughs |
| Domain registry + progress + ownership | ✅ OWNS | — | — |
| Deferral map (`⏳ Topic N`) | ✅ OWNS (topic-wide) | ✅ OWNS (in-tree markers) | ✅ wherever needed |
| Syntax bank / copy-paste templates | 🚫 FORBIDDEN | ✅ OWNS | illustrative snippets |
| Visual memory models (layouts, state tables) | contrast-level only | ✅ OWNS (family-level) | full traces |
| Trap archive (mistake → why → 10s fix) | 🚫 FORBIDDEN (route only) | ✅ OWNS (condensed, ≥3) | ✅ full pitfall history |
| 📺 Curated video sources | ✅ OWNS (topic shelf) | ✅ OWNS (family shelf) | ✅ OWNS (concept shelf) |
| Physical metaphors (§3 Phase 2) | topic paradigm metaphor | ✅ family metaphors per mechanism | full metaphor derivation |
| Real-world production hooks (§3 Phase 2) | topic-level survey | ✅ family hooks | deep dives |
| Video deep-dives, line-by-line traces, formulas' full proofs | 🚫 | 🚫 | ✅ OWNS |

> **💡 Core Rule — the Duplication Test:** If you are about to write a fact that already exists in another sheet, STOP. Link to it instead. The only allowed duplicates are *counts* in the dashboard/registry, which summarize — never duplicate — the sheets below. **Never copy a file to give it a second home** (see `facts.md` defects #1–#2: the byte-identical pairs).

### 1.2 Naming Law

- Tier 0: `<topic>/notes/00_<topic>_master_guide.md` — the ONLY files allowed the words "master guide".
- Tier 1 cross-cutting: `patterns/cheat_sheets/NN_<name>_cheat_sheet.md` (NN = global sequence).
- Tier 1 in-domain: `<topic>/notes/<domain>/cheat_sheet.md`.
- Tier 2: `<topic>/notes/<domain>/0X_<concept>.md` — unique two-digit numbers per domain, **never two files sharing a number** (see `facts.md` defect #6).

---

## 🗂️ PART 2 — Tier 0 Skeleton: THE TOPIC MASTER GUIDE (8 fixed sections, M1–M8)

Every topic master guide contains EXACTLY these 8 sections, in this order, with these headers (swap the topic name). Target length: **150–250 lines**. A master guide routes and paradigms; it never teaches syntax internals.

```markdown
# 🧭 <Topic> Master Guide — 30-Second Routing & Python ↔ Rust Paradigm

> **Primary Objective:** Within 30 seconds of facing any <topic> question, route to the correct
> DOMAIN sheet, state the Python↔Rust paradigm shift, and know the compiler rule guarding it.
> **Python Anchor:** <the ONE CPython mechanism this topic replaces at compile time>

## 📺 1. Curated Learning Sources
## 🐍 2. Python ↔ Rust Rosetta Stone
## 🧭 3. 30-Second Topic Decision Tree
## ⚖️ 4. Domain Contrast & Invariant Matrix
## 📈 5. Cognitive Learning Curve (Which Domain First)
## 🛡️ 6. Compiler Enforcement Map
## 🗺️ 7. Domain Registry & Navigation Index
## ⏳ 8. Manifest Boundaries & Deferral Map
```

### Per-section law

| ID | Section | Purpose | MANDATORY contents | Quality bar |
| :--- | :--- | :--- | :--- | :--- |
| **M1** | Title + Primary Objective | The 30-second promise + the Python anchor | Topic name; ONE blockquote promising the routing/paradigm skill; **always "30 seconds"**; one `Python Anchor:` line naming the CPython mechanism (e.g., "every object is `Rc<RefCell<PyObject>>` under the GIL") | Must name what the reader can DO, not what the file contains |
| **M2** | Curated Learning Sources | §2.2 mandate, elevated to Tier 0 | 3–5 validated links (Rust Book chapter, one video, one article) with a one-line "why this source" each | Every link opens; no dead paths; prefer the AGENTS.md §6 resource anchors |
| **M3** | Rosetta Stone | The topic's Python↔Rust translation table | Rows: Python pattern → Rust equivalent → **runtime & memory reality** (CPython cost vs Rust compile-time elimination) | ≥ 8 rows; reality column mandatory — this is the project's identity |
| **M4** | 30-Second Topic Decision Tree | Prompt → DOMAIN (never to single concepts) | One ASCII tree; 3–5 top-level questions; leaves are DOMAIN names + their Tier 1 sheet links | The umbrella tree ends at domains; concept-level trees belong to Tier 1 |
| **M5** | Domain Invariant Matrix | §2.3 mandate at topic scale | Table: dimension × domain — ownership mode / access restriction / Python primitive it replaces / **fatal Python trap** / complexity target | State the §2.3 protocol explicitly: Operation = Read ⇒ Shared Aliasing; Operation = Write ⇒ Exclusive Only (when applicable) |
| **M6** | Learning Curve | The "which domain first" rationale (learner's #1 missing asset: J = 0/24) | One ASCII curve (complexity ▲ vs phases →) with domains placed in learning order + 2–3 line rationale per phase ("learn X before Y because…") | Curve ≤ 25 lines; every phase names its anchor concept notes |
| **M7** | Compiler Enforcement Map | Compiler-as-co-teacher survey | Table: compiler rule / error code(s) (`E0502`, `E0072`…) / runtime bug it prevents / Python equivalent of that bug | Exact codes mandatory — "borrow checker complains" is defective |
| **M8** | Domain Registry & Deferral Map | Navigation + honesty | Tree or table: domain → Tier 1 sheet link → concept notes (Tier 2) with one-line scope → status (🟢/🟡/⏳). PLUS the deferral ledger: every future-topic concept this topic touches, with `⏳ Deferred until Topic N` | Registry matches disk exactly (no missing domains — see `facts.md` defect #14); counts match the root dashboard |

---

## 🧮 PART 3 — Tier 1 Skeleton: THE CHEAT SHEET WAR CHEST (12 fixed sections, C1–C12)

Every cheat sheet — cross-cutting or in-domain — contains EXACTLY these 12 sections, in this order (swap the family name). Target length: **120–250 lines**. A cheat sheet is a self-contained war chest: tree → matrix → syntax → memory → diagnostics → traps → sources.

```markdown
# 🧩 <Family> Cheat Sheet & 30-Second Invariants

> **Primary Objective:** <what the reader can recall/do in 30 seconds>
> **Master Guide:** [link up] · **Concept Notes:** [link down]

## 📺 1. Curated Learning Sources
## 🧭 2. 30-Second Decision Tree
## ⚖️ 3. Invariant & Operations Matrix
## 🐍 4. Python Bridge Delta
## 🧠 5. Physical Metaphor Register
## 🛠️ 6. Syntax Bank
## 🔬 7. Visual Memory Model
## 🛡️ 8. Compiler Diagnostics Table
## ⚠️ 9. Trap Archive (3-part form)
## ⏳ 10. Deferred Concepts (Manifest Boundary)
## 📈 11. Family Learning Progression
## 📎 12. Concept Notes Directory
```

### Per-section law

| ID | Section | Purpose | MANDATORY contents | Quality bar |
| :--- | :--- | :--- | :--- | :--- |
| **C1** | Title + Objective + tier links | 30s promise + the tier contract | Family name; ONE blockquote; up-link to the master guide; down-link statement | Links repo-relative, verified to exist |
| **C2** | Curated Learning Sources | §2.2 mandate | 2–4 validated links with one-line "why" | Same bar as M2 |
| **C3** | 30-Second Decision Tree | Prompt → exact syntax/pattern | One ASCII tree; 3–5 questions; leaves are SYNTAX with one-line mechanism | Tree ≤ 40 lines; every leaf carries the pattern name + one-line why |
| **C4** | Invariant & Operations Matrix | §2.3 mandate | Table: operation → borrow/access mode → aliasing allowed → mutation allowed (and complexity/ownership columns where relevant) | The Read⇒Shared / Write⇒Exclusive bifurcation visible in one glance |
| **C5** | Python Bridge Delta | This project's identity rule | The family-specific CPython contrast (e.g., `sys.getrefcount()` vs `Rc::strong_count()`; `PyFunctionObject` heap closure vs 24-byte unboxed struct) — NOT the topic Rosetta (that's M3's) | Every Rust claim paired with its CPython reality; no orphan comparisons |
| **C6** | Physical Metaphor Register | §3 Phase 2 mandate | One metaphor PER mechanism, each mapping 1:1 onto the actual data movement (e.g., Pringles can → LIFO drop order) | No jargon before its metaphor; metaphor must be falsifiable against the mechanism |
| **C7** | Syntax Bank | Copy-paste patterns | Minimal runnable snippets (≤ 25 lines each) with the invariant lines commented | **Manifest-verified**: every method/operator appears in the topic's CUMULATIVE AVAILABLE list — banned ones go ONLY into C10 as deferred previews |
| **C8** | Visual Memory Model | §3 Phase 4 mandate | One ASCII diagram or state table: stack/heap layout, ownership arrows, or state transitions — with a `Title:` line in a ```text fence | A reader who sees only this section can draw the layout from memory |
| **C9** | Compiler Diagnostics Table | Compiler-as-co-teacher | Table: exact diagnostic (`error[E0507]`… or panic message) → rule violated → **runtime bug prevented** → Python program that has that bug at runtime | Exact codes/panic strings mandatory, sourced from real encounters |
| **C10** | Trap Archive | §2.4 mandate, condensed for recall | ≥ 3 traps in EXACT 3-part form: **Assumed** → **Reality** → **10-Second Recall Fix** | Traps sourced from the learner's real pitfall history (Tier 2 notes / memory log); never invent traps |
| **C11** | Deferred Concepts | learn_rust-specific: the manifest boundary | Every future-topic concept adjacent to this family, with `⏳ Deferred until Topic N` and ONE line of what it will unlock | Follow the in-tree marker style: "⏳ BANNED IN TOPIC NN — arrives Topic MM: <one-liner>" (the Topic 06 domain sheets' model) |
| **C12** | Directory & Progression | Links down + mini curve | One bullet per Tier 2 concept note (link + one-line scope) + a 2–4 line mini progression ("learn 01 before 02 because…") | Links verified; progression closes the J-criterion gap at family level |

---

## 📄 PART 4 — Tier 2 Interface (Concept Notes)

Tier 2 concept notes (`notes/<domain>/0X_<concept>.md`) are **already governed by AGENTS.md §2** — this law does NOT modify them. Sheets interact with Tier 2 through exactly two contracts:

1. **C12 / M8 Directory** links down to every concept note with a one-line scope statement.
2. **Pitfall feeding** (Tier 2 → C10): when a concept note's Learner Pitfall Archive gains a pitfall, the family cheat sheet's Trap Archive gains the condensed 3-part form. The full derivation stays in Tier 2; only the 10-second recall form rises.

**FORBIDDEN:** duplicating a Tier 2 derivation (video deep-dives, step-by-step traces, full proofs) anywhere in Tier 0 or Tier 1.

---

## ✍️ PART 5 — Universal Writing Rules (The Python-Bridge-First Expressiveness Contract)

These rules apply to EVERY section of EVERY tier. They are the tutor's constitution elevated to law — a section that violates them is defective even if all sections are present.

1. **Answer first, inside every section.** The first 1–2 sentences carry the section's main point; examples and caveats come after.
2. **Python bridge before Rust mechanics.** Every Rust concept is introduced through what CPython does at runtime (refcount, heap object, pointer chasing) and only then how Rust moves the cost to compile time. This is the project's founding rule — a sheet section without its Python grounding is defective.
3. **Physical metaphor before mathematics.** Every abstract mechanism gets its 1:1 physical metaphor BEFORE any formula or formal term.
4. **No orphan formulas.** Every equation is preceded by a plain-English sentence explaining what it calculates and why it matters.
5. **No code shorthand as explanation.** State the mechanism in words FIRST; the snippet illustrates.
6. **Define jargon inline.** First use of any Rust term gets a 4–10 word plain-English meaning (with a Python analogy where one exists). Assume a second-language English reader.
7. **One rule per bullet.** Never fold two rules into one line.
8. **Self-contained sections.** No "as mentioned above." Each section must survive alone — 30-second recall after months away depends on this.
9. **The 10-Second Expressiveness Bar:** a section is sufficiently expressive when the learner can (a) DECIDE in ≤ 10 seconds, (b) STATE the mechanism in one line, (c) RECALL the trap without re-reading. If any fails, rewrite it.
10. **Every claim carries its anchor.** Every pattern, trap, and memory claim cites its workspace file or its manifest entry. No invented benchmarks, no invented pitfalls.
11. **Invariant awareness everywhere.** For any borrow/lookup/access pattern, state whether Operation = Read ⇒ Shared Aliasing Allowed, or Operation = Write ⇒ Exclusive Access Only (AGENTS.md §2.3).
12. **Prerequisite gating everywhere.** Any content depending on an uncovered topic is marked `⏳ Deferred until Topic N` — never silently introduced. The Queue guide's "[Topic 5 Preview]" pattern and the shared-ownership sheet's in-tree BAN marker are the two approved forms.

---

## 🎨 PART 6 — Formatting & Manifest Law

1. **Headers exactly as specified** in PART 2 / PART 3 — same emoji, same numbering, same order. This IS the muscle-memory contract.
2. **ASCII only for diagrams — NEVER Mermaid.** Every ASCII block opens with a `Title:` line inside a fenced ```text block.
3. **Fixed emoji palette:** 🧭 tree/routing · ⚖️ invariant · 🐍 Python bridge · 🧠 metaphor/memory · 🛠️ syntax · 🔬 visual model · 🛡️ compiler · ⚠️ traps · 📺 sources · ⏳ deferred · 📈 curve · 📎 directory · 🟢/🟡/🔵/🔴 progress markers.
4. **Math in LaTeX** (`$...$` inline, `$$...$$` display) — matching existing sheets.
5. **Tables are GFM**; never let a cell exceed ~2 lines; no blank lines inside a table body (see `facts.md` defect #4).
6. **Line budgets:** Master guide 150–250 · cheat sheet 120–250 · **war chest absorbing an archetype library 250–400** (the Topic 04 split target) · decision tree ≤ 40 · single syntax pattern ≤ 25. Exceeding a budget means content belongs at a different tier.
7. **The timing promise is always "30 seconds."** The 10-second and 5-second variants are retired.
8. **Link convention (AMENDED [2026-10-03, later] — supersedes the same-day earlier decision):** all inter-document links are **repo-relative paths resolved against the containing file's directory** (e.g. `../../04_error_handling/notes/errors/04_the_question_mark_operator.md`). **`file:///` absolute URIs are FORBIDDEN** — standard markdown renderers (GitHub, VS Code preview, web viewers) resolve links relative to the current file and do not make `file:///` URIs clickable, so absolute links silently break navigation (empirically verified by the learner). Every link's target must be verified to exist at authoring time — extract every link destination, resolve it against the file's directory, and `stat` it. *(History: the learner first chose absolute paths expecting better hyperlink navigation, then verified they don't render as links — the audit's original repo-relative recommendation is restored.)*
9. **Progress honesty:** complete = listed with 🟢; scaffolded/unsolved = ⏳; never present a plan as done (see `facts.md` defect #5: the empty `visualizations/` promises).
10. **⛔ THE MANIFEST GATE (learn_rust-specific, hard rule):** before authoring ANY sheet section, run the AGENTS.md §12 verification: every type, method, operator, and syntax pattern in the sheet (including code comments, tree leaves, and tables) must appear in the CUMULATIVE AVAILABLE list for that topic in `CONCEPT_MANIFEST.md`. Banned elements may appear ONLY inside the Deferred Concepts section (C11) or deferral map (M8) as one-line previews with their marker. **If the manifest has no home for a needed element, the manifest is amended FIRST (with the human's approval), then the sheet is authored.**

---

## 🚚 PART 7 — Migration Map (current content → target home)

The per-file work order with acceptance checklists lives in `context.md`. Summary of the moves:

| Content | Today | Target home |
| :--- | :--- | :--- |
| Topic 04 war chest (565-line mixed guide) | `00_error_handling_master_guide.md` | Split: umbrella M1–M8 stays; archetype blueprints + pitfall archive + checklist move down to `07_error_handling_cheat_sheet.md` (C-war chest) — OR the guide is re-titled a Tier 1 war chest and a thin umbrella is born; see `qa_log.md` Q2 |
| Byte-identical cheat sheets 00 & 01 | 2 duplicate copies | ONE home each (see `qa_log.md` Q1): master guide keeps topic routing; the cheat sheet becomes the family war chest (or the duplicate is deleted) |
| Arc / Arc<Mutex> leaves in Topic 06 guide + sheet 10 | Unmarked recommendations | In-tree `⏳ BANNED IN TOPIC 06 — Topic 08` markers (match the domain sheets' form) |
| `.map_err`/`.ok_or_else`/`.and_then` closures in Topic 04 sheet + guide templates | Contradicts the guide's own Pitfall 8 | Either ⏳ markers + match-based Topic 04 alternative shown first (historical fidelity), or — now that Topic 05 is complete — an explicit "unlocked by Topic 05" note; either way the contradiction is resolved, not left silent |
| String/&str methods (push_str, chars, char_indices…) | Taught in sheet 02, whitelisted NOWHERE | `CONCEPT_MANIFEST.md` Topic 01 amendment (CUMULATIVE AVAILABLE: string-slice methods), then sheet 02 re-verifies against it |
| `05_lifetimes_intro.md` + `05_lifetimes_and_elision_rules.md` numbering collision | Two files, one number | Merge scope or renumber (see `qa_log.md` Q5) |
| 14 empty `visualizations/` dirs | Phantom promise | Delete, or fill from existing note diagrams, or remove the promise from AGENTS.md §2 + all trees (see `qa_log.md` Q6) |
| Missing learning curves (J = 0/24) | Nowhere | Every Tier 0 gains M6; every Tier 1 gains C12-progression |
| Missing 📺 sources (I = 2/24) | 2 sheets | Every sheet gains M1/C2 shelves (sources already exist in the Tier 2 notes to curate from) |
| Dashboard defects (focus line, split table, absolute links, unindexed manifest/reports) | `README.md` | Fix per `context.md` §3.0 (P0) |

---

## 🆕 PART 8 — New Topic Instantiation Checklist (Topics 07–13)

When a new topic folder is activated, the FIRST documentation act is to instantiate this tree — BEFORE any concept is taught. This prevents the next generational spread.

```text
NEW TOPIC ACTIVATION — mandatory order
================================================================================
[ ] 1. Expand the topic's section in CONCEPT_MANIFEST.md FIRST (INTRODUCES /
        CUMULATIVE AVAILABLE / EXPLICITLY BANNED) — every later artifact
        verifies against it (AGENTS.md §12). Fix manifest gaps discovered
        later by amendment, never by silent usage.
[ ] 2. Create <topic>/notes/00_<topic>_master_guide.md (PART 2 skeleton,
        M1–M8). Unsolved domains in the registry are marked ⏳/⬜ honestly —
        structure exists from day one, content grows as concepts are mastered.
[ ] 3. Create ONE Tier 1 cheat sheet per expected domain (PART 3 skeleton,
        C1–C12) — cross-cutting sheets go to patterns/cheat_sheets/ with the
        next global NN; domain sheets live in notes/<domain>/cheat_sheet.md.
[ ] 4. Register every sheet in the master guide's M8 Domain Registry.
[ ] 5. Index the master guide + sheets in the root dashboard (Tier 00), with
        repo-relative links, and refresh the Current Focus line.
[ ] 6. Log the instantiation in the memory graph (Session Activity Log).
[ ] 7. As each concept is mastered, grow the OWNING concept note (Tier 2) and
        feed its pitfalls UP to the cheat sheet's C10 archive; as each domain
        completes, update the master guide registry + dashboard counters
        (6-Point Post-Exercise Sync, AGENTS.md §5.6).
================================================================================
```

> **💡 The Rigidity Bargain:** Reference sheets are hard to modify later — that is a feature IF the skeleton is complete from day one. PART 8 exists so Topics 07–13 inherit the standard for free, and the learner's 30-second muscle memory compounds across all 14 topics instead of resetting at each one.

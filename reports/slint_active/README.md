# 🎨 Slint Active UI Reports & Architectural Choices

> **Domain:** Live UI architecture, layout specifications, component design decisions, and Android $\leftrightarrow$ Slint translation logs.

This directory documents the concrete UI designs authored by the AI Agent for the `ai-workflow-gui` project. Every design decision, widget selection, stretch constraint, and property binding contract is detailed here for human review and interview preparation.

---

## 📑 Reports Index

| # | Topic | File | Status | Description |
|---|---|---|---|---|
| 01 | Chat Stack UI Architecture | [`01_chat_stack_ui_architecture.md`](01_chat_stack_ui_architecture.md) | 🟢 Complete | Reverse message stack, ListView native scrolling, bottom fixed input bar, ChatMessage struct, and Slint $\leftrightarrow$ Rust bindings. |
| 02 | Properties & Data Models Cheat Sheet | [`cheat_sheet_properties_and_models.md`](cheat_sheet_properties_and_models.md) | 🟢 Active | Tier 1 War Chest (C1–C12): Decision tree, property invariants, syntax bank, memory model, and 7-part trap archive. |
| 03 | Slint Types & Smart Pointers Cheat Sheet | [`cheat_sheet_slint_types_and_smart_pointers.md`](cheat_sheet_slint_types_and_smart_pointers.md) | 🟢 Active | Tier 1 War Chest (C1–C12): ModelRc<T> collection model, SharedString, VecModel, ChatMessage struct, and 7-part trap archive. |
| 04 | WhatsApp-Style Chat Bubble System | [`02_whatsapp_bubble_layout.md`](02_whatsapp_bubble_layout.md) | 🟢 Complete | Compact bubble sizing (320px fixed width), export struct ChatMessage, directional alignment (gravity), and ListView integration. |

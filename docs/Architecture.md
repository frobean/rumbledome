# RumbleDome Architecture & Working Agreement

**Purpose:** This document bootstraps any contributor (including future-AI-you) on the project’s intent, structure, iteration rhythm, and non-negotiable rules. If a session resets, read this first.

---

## 0) TL;DR for Future Me (Cold-Start Checklist)

1. Read: `/docs/Context.md`, `/docs/DesignSpec.md`, `/docs/ImplementationSpec.md`, `/docs/Interfaces.md`, `/docs/CalibrationAndDefaults.md`, `/docs/MustNotDrop.md`.
2. Confirm invariants in **MustNotDrop.md** (e.g., *duty=0% ⇒ no-boost*).
3. Follow the **Iteration Loop** (below) — never “vibe code”.
4. If anything new becomes critical, **amend the spec first**, then code.
5. All code changes must have **tests** and must not “drop stitches” from the checklist.

---

## 1) Intent (Why this exists)

- Build a **full-dome** EBC that’s **closed-loop**, **self-learning**, and **fail-safe**.
- Optimize for **driveability** (street, road course) not just drag-strip launch maps.
- Configuration in **pressure units (PSI/kPa)**, not raw duty.
- Robust engineering: modular, testable, explainable.

---

## 2) Project Structure

We keep a strict split between **core logic** and **hardware**. Core compiles and runs on desktop (sim) and on MCU (firmware).

# RumbleDome: AI-Accelerated Boost Controller

Welcome to **Mad Hacks: RumbleDome** — a custom, full-dome electronic boost controller built around the Teensy 4.1 microcontroller and written in Rust.  

This project is an experiment in co-developement by a human architect (me) and an AI assistant (ChatGPT).  
This README serves two purposes:
- **For humans**: a high-level overview of the project, goals, and how to navigate the repo.  
- **For AI**: a “bootloader” that explains where to find specs, interfaces, and manifests to resume work after resets.  

---

## ⚠ NOTE and WARNING

This project is experimental on basically every level. I am working through it to teach myself a number of things:  

- **Rust as a programming language**  
- **AI collaboration** — how to design and structure a development process that makes use of AI to produce consistent and usable results without ending up with a complete pile of trash at the end. The AI can work and reason somewhat autonomously, but I reserve the control to override any decision anywhere in the code.  
- **Microcontroller programming** — this is my first from-scratch firmware.  
- **Electronic boost control theory and physics** — I have a very specific goal I am aiming for in the level of integration and control between my aftermarket turbo system and the OEM systems.  
- **Ford CAN bus** — specifically for the stock (non-FRPP) Gen 2 Coyote engine management.  
- **Basic electronics** — because why bother learning with blinky LEDs and elementary exercises when I have a fun idea that no one else has ever built, along with the potential to blow up an expensive engine if things go south...  

I'm making this open source and available on the off chance that someone else might find it interesting or educational.  

**Legal / liability disclaimers:**  
- See adjacent LICENSE.md

Consider yourself warned.  

--

## 📖 Project Overview
- **Firmware**: Rust (no unsafe where possible), modular design, Teensy 4.1 target.  
- **Hardware**:
  - Teensy 4.1 MCU  
  - 4-port MAC solenoid (30 Hz, 0% duty = lower dome pressure = failsafe to no boost)  
  - 3 pressure sensors (0–30 psi, 0.5–4.5 V output)  
  - ST7735R TFT LCD (1.8" 128×160)  
  - CAN bus transceiver (SN65HVD230 or similar)  
  - OBD2 adapter for CAN connection  
- **Concept**: Closed-loop, self-learning dome pressure control with multiple user profiles and fail-safes.  

---

## 🗂 Repo Structure
- `/src` → Rust firmware modules (code TBD, APIs defined in Interfaces.md).  
- `/docs` → Design documents and specifications.  
  - `Context.md` → High-level design context (narrative + goals).  
  - `DesignSpec.md` → Detailed design specification.  
  - `Interfaces.md` → API boundaries and module contracts.  
  - `Definitions.md` → Acronyms, jargon, and domain-specific terminology.  
- `.github/workflows/manifest.yml` → Generates a machine-readable manifest of all repo files, with raw URLs, on checkin to main.  

---

## 🧭 Bootstrap Instructions (AI + Human)
If you are **an AI assistant** resuming work:
1. Load this README.  
2. Load `docs/Context.md`, `docs/DesignSpec.md`, and `docs/Interfaces.md`.  
3. Use `.github/workflows/manifest.yml` to resolve the current repo structure and raw file URLs.  
4. Maintain continuity: never drop requirements listed in the specs.  
5. Work module-by-module using Interfaces.md as the contract.  

If you are **a human developer**:
1. Start by reading `docs/Context.md` for the project narrative.  
2. Review `docs/DesignSpec.md` for detailed architecture and constraints.  
3. Use `docs/Interfaces.md` to understand how modules interact.  
4. Build/test steps will be documented in `/docs/Build.md` (TBD).  

---

## 🛡 Collaboration Rules
- Specs and context docs are the **single source of truth**.  
- Any new insights → update the docs first, then code.  
- Code must be **verbose, modular, and testable**.  
- Failure paths must **always fail safe** (drop to zero boost).  
- AI will generate files for human review → human commits them to GitHub.  

---

## 🤖 AI Working Agreements
When assisting with this project, the AI must:
1. **Anchor to README.md** at the start of every session to regain context.  
2. **Always check the manifest** for the latest repo structure before assuming file availability.  
3. **Never drop requirements**: anything listed in Context.md, DesignSpec.md, or Interfaces.md is binding until explicitly removed.  
4. **Work module-by-module**: respect API contracts, don’t introduce cross-cutting hacks.  
5. **Document assumptions clearly**: mark speculative areas with `⚠ SPECULATIVE` so the human knows to verify.  
6. **Preserve clarity and style**: verbose variable names, self-documenting code, proper comments for math/algorithms.  
7. **Approachability in generated code/docs**: Never assume the reader is an expert in the math, jargon, microcontroller, physics, or theory.
8. **Fail safe in code paths**: defaults and error states must never result in uncontrolled boost.  
9. **Keep testability in mind**: unit tests must be able to run with fake data without hardware.  
10. **Surface gaps**: if required details are missing from the spec, pause and request clarification rather than guessing silently.  
11. **Respect layering**: HAL abstractions first, hardware-specific logic later.  

---

## 🚦 Status
- [ ] Context defined  
- [ ] Interfaces defined  
- [ ] Design spec fleshed out  
- [ ] Rust firmware scaffolding  
- [ ] Unit test harness  
- [ ] HAL abstraction for CAN/sensors  
- [ ] Self-learning logic  

---

## 👨‍💻 Humans
- **Build instructions**: TBD  
- **Wiring diagrams**: TBD  
- **Contribution**: Fork, branch, PR.  

---

## 🤖 AI
- Always anchor to this README on reset.  
- Never assume missing context → fetch from specs.  
- Output must align with module APIs and coding conventions.  

---

*Mad Hacks: RumbleDome — because sometimes boost control needs a little chaos, carefully engineered.*
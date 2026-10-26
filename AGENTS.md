# Agent Guidelines

Mandatory instructions for any AI agent working in this codebase. Every rule is binding and non-negotiable.

---

## 1. Spec-Driven Development (SDD) Mandate

**Consult specifications before writing code. Never code from ungrounded assumptions.**

Before writing, editing, or refactoring code:
1. Read the foundational project documents for the domain you are touching:
   - **Context & Motivation**: `docs/CONTEXT.md` - architecture philosophy and problems solved.
   - **Product Requirements**: `docs/PRD.md` - functional boundaries, user personas, and performance benchmarks (<30MB RAM).
   - **System Architecture**: `docs/ARCHITECTURE.md` - subsystem boundaries (Supervisor, DNS, SSL, Config, Inspector), portable filesystem layout (`bin/`, `data/`, `templates/`, `www/`), and platform abstraction layer.
   - **System Design**: `docs/SYSTEM-DESIGN.md` - process state machines, Win32 Job Objects / Linux process groups, virtual host auto-detection rules, and UDP DNS packet logic.
   - **IPC Contract**: `docs/IPC-SPEC.md` - exact Tauri command names, TypeScript interfaces, and streaming event schemas.
   - **Domain Vocabulary**: `GLOSSARY.md` - use canonical terms (*Service*, *Stack*, *Virtual Host*, *Local Domain*, *Root CA*, *Supervisor*, *Tray*); never use deprecated synonyms.
   - **Architectural Decisions**: `docs/adr/` - check ADRs 0001 through 0006 before altering service paradigms.
   - **Active Sprint Tickets**: `.scratch/bamp-mvp/spec.md` and `.scratch/bamp-mvp/issues/<NN>-<slug>.md` - claim one ticket at a time, fulfill its completion criteria, and record progress.
2. If your proposed implementation contradicts any specification, stop and update the ticket or author an ADR before touching code.

---

## 2. Up-To-Date Documentation & Zero Outdated Priors

**Model training weights contain stale APIs. Always query authoritative reference documentation.**

### 2.1 Tauri v2 Local Documentation
- **Source**: `docs/tauri/` (`llms-full.txt`, `guides.txt`, `reference.txt`).
- **Rule**: For any Tauri configuration (`tauri.conf.json`), Rust IPC command declaration, plugin usage (`tauri-plugin-opener`), system tray setup, or frontend invocation (`@tauri-apps/api`), inspect `docs/tauri/` first. Never rely on memorized Tauri v1 or beta patterns.

### 2.2 Context7 MCP Documentation Server
- **Tools**: Call `resolve-library-id` and `query-docs` on the `context7` MCP server.
- **Rule**: Whenever introducing or working with external libraries, frameworks, SDKs, or Rust crates (e.g. React 19, Tailwind CSS, `tokio`, `notify`, `rcgen`, `serde`, `windows-sys`, `libc`), query `context7` for the current API signatures, breaking changes, and configuration best practices. Query even when you believe you know the syntax.

---

## 3. Frontend Craft & Design Standards (Hallmark & Impeccable)

**Build interfaces that look engineered by top-tier designers, never generic AI-generated templates.**

When implementing, modifying, or styling any UI component, page, or dashboard:
1. **Hallmark Floor (`.agents/skills/hallmark/SKILL.md`)**:
   - Declare every color and font token in CSS variables at `:root` (or Tailwind theme config). No raw hex colors or arbitrary font families scattered in component markup.
   - Never use `transition: all`. Never animate layout dimensions (`width`, `height`, `top`, `left`); use `transform` and `opacity` only.
   - Enforce `overflow-x: clip` on root containers.
   - Provide clear, instant `:focus-visible` styling on every interactive element.
   - Provide `prefers-reduced-motion: reduce` fallbacks for animations.
   - Cap and center desktop containers at maximum bounds.
2. **Impeccable Craft (`.agents/skills/impeccable/SKILL.md`)**:
   - Operate in **Operate Mode**: High-density information architecture, instant scannability, consistent visual rhythm, and native desktop tool aesthetics (reminiscent of Raycast, Linear, or Herd).
   - Use crisp typography hierarchy, clear status pill badges (Running = green, Stopped = zinc, Conflict = amber, Error = red), and zero decorative visual noise.
   - Verify rendered results with real content and handle every state: Loading, Empty, Active, Port Conflict, and Error states.

---

## 4. Engineering Principles & Guardrails

### 4.1 Zero Zombie Guarantee
- Every child process spawned by the Rust backend MUST be registered to a Win32 Job Object (Windows) or Process Group (Linux).
- When the BAMP application quits or receives SIGINT/SIGTERM, all child processes (Nginx, PHP, MySQL, Mailpit) must be terminated with zero orphaned survivors.

### 4.2 Simplicity First
- Minimum code that solves the ticket. Nothing speculative.
- No unrequested abstractions, no speculative wrappers, no over-engineering.
- If 50 lines solves the problem cleanly, write 50 lines.

### 4.3 Surgical Changes
- Touch only files directly related to your active ticket.
- Match existing style. Do not reformat adjacent unrelated code.
- Clean up only your own orphans.

### 4.4 Goal-Driven Verification
- Every ticket defines checkable completion criteria.
- Execute automated checks (e.g. `cargo check`, `pnpm build`, unit tests) and verify criteria before marking any ticket resolved.

---

## 5. Issue Tracking & Workflow Conventions

- Work is tracked in `.scratch/<feature-slug>/issues/<NN>-<slug>.md`.
- Claim tickets sequentially (`Status: claimed`).
- Append implementation notes and test verifications under `## Comments`.
- Set `Status: resolved` only when all completion criteria pass.

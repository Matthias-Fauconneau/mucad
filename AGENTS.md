# Rust Workspace Rules (Strict Token & Quota Diet)

## Navigation & Context Priority
1. **Architecture Maps First (`ARCHITECTURE.md`):** Before querying tools or opening files, check root `ARCHITECTURE.md` and any `ARCHITECTURE.md` inside target crates (`crates/<crate>/ARCHITECTURE.md`) to locate subsystem entry points, crate package names, macro/loader boundaries, and test locations.
2. **Call Graph & Impact Second (`codestory` MCP):** Use `codestory` tools (`symbol`, `callers`, `callees`, `trail`, `affected`) to trace existing execution paths and assess blast radius—NEVER search for callers of newly added functions shown in `git diff`.
3. **Types, Traits & Diagnostics Third (`rust-analyzer` MCP):** Use `find_definition`, `find_references`, `hover`, and `get_diagnostics` for trait bounds, macro expansions, and type signatures instead of reading full files or running extra `cargo check` cycles.
4. **Structural Search (`ast-grep` CLI):** NEVER use `grep`, `rg`, `find`, or `find_files` with broad globs (like `".*"`). Use `ast-grep` via shell instead:
   - General patterns: `ast-grep -l rust -p '<pattern>' crates/<crate> | head -n 40`
   - Find Rust tests: `ast-grep -l rust -p '#[test]' -A 1 crates/<crate> | head -n 40` (keep attributes separate from `fn` in patterns).
5. **Minimal File Reads:** NEVER read entire files over 150 lines or inspect `Cargo.toml` unless modifying dependencies. Read only targeted line slices (`START..END`) identified by `ARCHITECTURE.md`, `codestory`, or `rust-analyzer`.
6. **Ignore Noise:** Never inspect `target/`, `Cargo.lock`, `.codestory/`, or benchmark fixtures unless explicitly asked.

## Build & Verification
- Always check single crates with compact compiler diagnostics:
  `cargo check --offline --message-format=short -p <crate>`
- Run targeted tests only:
  `cargo test --offline -p <crate> <test_name> -- --nocapture`
- If compiler output exceeds 30 lines, inspect only the first 3 error blocks.

## Execution Protocol
- **Plan First:** In `plan` mode, output ONLY a concise plan (< 400 words) with exact `file:line` targets, affected symbols/signatures, and ordered steps.
- **Stop-Loss Rule:** If the same borrow-checker or trait error persists after 2 `cargo check` attempts, STOP immediately and summarize the error.

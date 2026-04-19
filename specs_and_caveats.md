Notes by Kimi K2.5

---

**Your Specs Recap:**
- **Inline window**: Constrain any command (e.g., `tail -f`, `yes`) to a fixed height (e.g., 20 lines) starting at the **current cursor position** (not bottom-fixed)
- **Non-destructive**: Push existing terminal content up/down to make room; do not overwrite history above; do not use alternate screen buffer (keep scrollback "natural")
- **Clean exit**: On SIGINT or normal exit, restore terminal to sane state (reset scroll region, position cursor after the block, no screen erasure)
- **Environment**: Konsole and JetBrains IDE terminal, mixed usage of xonsh/zsh
- **Explicit rejection**: Full-terminal multiplexers (tmux/zellij) due to mouse/scrollback capture concerns; wanting "just a box" without the PTY abstraction layer

---

**Issues That Apply to the Rust Version Too (early attempts were in bash):**

**1. Cursor Position Detection Fragility**
- **Issue**: The `\e[6n` query can return garbage, timeout, or return `(0,0)` if the terminal is in a bad state (leftover raw mode, previous crash, Konsole quirks). Bash parsed this as row 1, causing the "region starts at top of screen" bug.
- **Rust impact**: `crossterm::cursor::position()` can fail or return `(0,0)`. You must validate `row > 0` and have a fallback (run command unconstrained rather than crashing or mis-positioning).

**2. State Leakage Between Invocations**
- **Issue**: If the previous run crashed after setting scroll region but before cleanup, the next run inherits a constrained scroll region and/or raw mode, causing "chaotic/infinite regions."
- **Rust impact**: You **must** execute `\e[r` (reset scroll region) and `disable_raw_mode()` immediately on startup, before any other terminal operations, to clean up your own previous crashes.

**3. 0-indexed vs 1-indexed Hell**
- **Issue**: crossterm/Rust uses 0-indexed rows (0 = first line). ANSI DECSTBM (scroll region) and CPR (cursor position report) use 1-indexed. Off-by-one errors cause regions to be 1 line short or 1 line too long.
- **Rust impact**: Convert carefully: `print!("\x1b[{};{}r", row + 1, end_row + 1);`

**4. Clamping to Available Space**
- **Issue**: If cursor is at row 22 of 24 and user requests 20 lines, you only have 3 lines available. Inserting 20 lines pushes 17 lines off-screen (into oblivion, not scrollback).
- **Rust impact**: Calculate `available = height - row` and `actual_lines = min(requested, available)`. If `available <= 0`, bail out to unconstrained mode.

**5. Terminal Mode Handoff**
- **Issue**: Cursor query requires raw mode, but the child command (e.g., `vim`, `tail`) expects cooked mode. If you spawn the child while in raw mode, line buffering breaks.
- **Rust impact**: `crossterm::terminal::enable_raw_mode()` for the query, then `disable_raw_mode()` *before* `Command::spawn()`, then re-enable only if you need to query again (you don't).

**6. SIGINT Propagation vs Cleanup**
- **Issue**: If you die on SIGINT without cleanup, the terminal stays in the scroll region. If you catch SIGINT but don't forward it to the child, the child keeps running orphaned.
- **Rust impact**: The `ctrlc` handler + `Drop` guard pattern works, but ensure you `child.kill()` and `child.wait()` in the handler path, otherwise the guard drops while the child is still writing to the terminal.

**7. Konsole-Specific Quirks**
- **Issue**: Konsole handles `ESC [ L` (insert line) and scroll region resets differently than xterm/kitty. The "infinite region" and "screen erasure on exit" bugs were likely Konsole interpreting rapid sequence changes as "clear screen."
- **Rust impact**: Test specifically on Konsole. You may need to add `stdout.flush()` after every escape sequence and small `thread::sleep(Duration::from_millis(10))` if Konsole drops sequences sent too fast.

**8. Resize (SIGWINCH) During Setup**
- **Issue**: If user resizes terminal between the cursor query and the scroll region setup, `end_row` points off-screen, causing chaos.
- **Rust impact**: Handle `SIGWINCH` (crossterm has `Resize` event) or snapshot size atomically. Minimum: check terminal size *immediately* after cursor query and bail if it changed.

**9. The "Prompt on Next Line" Scroll**
- **Issue**: To place the prompt on a new line after the region, you must print `\n`. If at the bottom of the screen, this scrolls the entire terminal up by one, "erasing" the top line (moving it to scrollback).
- **Rust impact**: Inevitable tradeoff. You either leave the cursor at the last line of output (risking prompt overwrite) or scroll one line. Document this behavior.

**10. Child Exit Status**
- **Issue**: The wrapper should return the child's exit code, or `130` for SIGINT, to maintain shell semantics (for scripting `$?` checks).
- **Rust impact**: `std::process::exit(child.wait()?.code().unwrap_or(130));` or similar in the Drop guard context (tricky because Drop happens after scope exit; use a wrapper function or explicit cleanup call instead of relying solely on Drop for the final exit code).

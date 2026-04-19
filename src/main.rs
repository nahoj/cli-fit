use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct RegionGuard {
    end_row: u16,
}

impl Drop for RegionGuard {
    fn drop(&mut self) {
        // Reset scroll region to full screen
        print!("\x1b[r");
        // Position cursor at end of our block (1-indexed)
        print!("\x1b[{};1H", self.end_row + 1);
        println!();
        io::stdout().flush().unwrap();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: bottom <lines> <command> [args...]");
        std::process::exit(1);
    }

    let lines: u16 = args[1].parse().expect("Invalid line count");
    let cmd = &args[2];
    let cmd_args = &args[3..];

    // Get cursor position (0-indexed) and terminal size
    crossterm::terminal::enable_raw_mode().unwrap();
    let (_col, row) = crossterm::cursor::position().unwrap();
    let (_w, h) = crossterm::terminal::size().unwrap();
    crossterm::terminal::disable_raw_mode().unwrap();

    let available = h.saturating_sub(row);
    let actual_lines = lines.max(1);
    let deficit = actual_lines.saturating_sub(available);

    // If not enough space below cursor, scroll terminal up and move cursor up
    let row = if deficit > 0 {
        print!("\x1b[{}S", deficit); // scroll up: top lines go to scrollback
        print!("\x1b[{}A", deficit); // cursor up to new start position
        row.saturating_sub(deficit)
    } else {
        row
    };

    let end_row = row + actual_lines - 1; // 0-indexed

    // Insert lines at cursor (push content down)
    print!("\x1b[{}L", actual_lines);
    // Set scroll region (1-indexed, inclusive)
    print!("\x1b[{};{}r", row + 1, end_row + 1);
    // Move cursor to top of new region
    print!("\x1b[{};1H", row + 1);
    io::stdout().flush().unwrap();

    let _guard = RegionGuard { end_row };

    // Setup Ctrl+C handler
    let interrupted = Arc::new(AtomicBool::new(false));
    let i = interrupted.clone();
    ctrlc::set_handler(move || {
        i.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut child = Command::new(cmd)
        .args(cmd_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn command");

    // Wait for child or interrupt
    loop {
        if interrupted.load(Ordering::SeqCst) {
            let _ = child.kill();
            let _ = child.wait();
            break;
        }
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(50)),
            Err(_) => break,
        }
    }

    // Guard drops here, resetting terminal
}
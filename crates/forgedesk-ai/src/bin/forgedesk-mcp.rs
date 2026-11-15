use std::io::{self, BufRead, Write};

use forgedesk_ai::mcp::handle_request;
use forgedesk_ai::ContextSelection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw = std::env::var("FORGEDESK_AI_CONTEXT")?;
    let context: ContextSelection = serde_json::from_str(&raw)?;
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(response) = handle_request(&line, &context) {
            writeln!(stdout, "{response}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

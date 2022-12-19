//! Create and manage bevy_console commands.

use crate::*;

// Modified from the log command example (https://github.com/RichoDemus/bevy-console/blob/main/examples/log_command.rs)
/// Prints given arguments to the console
#[derive(ConsoleCommand)]
#[console_command(name = "log")]
pub struct PrintToLog {
    /// Message to print
    msg: String,
    /// Number of times to print message
    num: Option<i64>,
}

pub fn print_to_log(mut log: ConsoleCommand<PrintToLog>) {
    if let Some(Ok(PrintToLog { msg, num })) = log.take() {
        let repeat_count = num.unwrap_or(1);

        for _ in 0..repeat_count {
            reply!(log, "{msg}");
        }

        log.ok();
    }
}

//! A minimal terminal spinner for the long blocking yt-dlp call.
//!
//! yt-dlp runs under `-J`, which forces quiet mode, so it prints nothing while
//! working - without this the tool looks hung for the length of a download.
//! Granular percentages are not available here: the `--downloader ffmpeg`
//! external downloader only reports progress once, on completion.

use std::io::{IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const INTERVAL: Duration = Duration::from_millis(100);

/// Animates a spinner on stderr until dropped. Does nothing at all when stderr
/// is not a terminal, so piped and CI output stays clean.
pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Spinner {
    pub fn start(message: &str) -> Self {
        if !std::io::stderr().is_terminal() {
            return Spinner {
                running: Arc::new(AtomicBool::new(false)),
                handle: None,
            };
        }

        let running = Arc::new(AtomicBool::new(true));
        let message = message.to_string();

        let handle = {
            let running = Arc::clone(&running);
            thread::spawn(move || {
                let mut frame = 0;
                while running.load(Ordering::Relaxed) {
                    let mut stderr = std::io::stderr().lock();
                    let _ = write!(stderr, "\r{} {}", FRAMES[frame % FRAMES.len()], message);
                    let _ = stderr.flush();
                    drop(stderr);

                    frame += 1;
                    thread::sleep(INTERVAL);
                }
            })
        };

        Spinner {
            running,
            handle: Some(handle),
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();

            // Blank the spinner line so it does not linger above whatever is
            // printed next.
            let mut stderr = std::io::stderr().lock();
            let _ = write!(stderr, "\r{}\r", " ".repeat(80));
            let _ = stderr.flush();
        }
    }
}

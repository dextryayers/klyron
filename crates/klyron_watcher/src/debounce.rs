use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use crossbeam_channel::{unbounded, Receiver};
use glob::Pattern;

use crate::{HmrUpdate, WatchEvent};
use crate::fs::is_ignored_by_patterns;

pub struct Debouncer {
    debounce: Duration,
}

impl Debouncer {
    pub fn new(debounce: Duration) -> Self {
        Self { debounce }
    }

    pub fn run<F>(
        &self,
        rx: Receiver<WatchEvent>,
        running: Arc<AtomicBool>,
        ignore: Vec<Pattern>,
        callback: F,
    ) where
        F: Fn(HmrUpdate) + Send + 'static,
    {
        while running.load(Ordering::SeqCst) {
            let mut added = Vec::new();
            let mut changed = Vec::new();
            let mut removed = Vec::new();
            let deadline = Instant::now() + self.debounce;

            loop {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    break;
                }
                match rx.recv_timeout(remaining) {
                    Ok(WatchEvent::Create(p)) => {
                        if !is_ignored_by_patterns(&p, &ignore) {
                            added.push(p);
                        }
                    }
                    Ok(WatchEvent::Modify(p)) => {
                        if !is_ignored_by_patterns(&p, &ignore) {
                            changed.push(p);
                        }
                    }
                    Ok(WatchEvent::Remove(p)) => {
                        if !is_ignored_by_patterns(&p, &ignore) {
                            removed.push(p);
                        }
                    }
                    Ok(WatchEvent::Rename(from, to)) => {
                        if !is_ignored_by_patterns(&from, &ignore) {
                            removed.push(from);
                        }
                        if !is_ignored_by_patterns(&to, &ignore) {
                            added.push(to);
                        }
                    }
                    Ok(WatchEvent::Any(p)) => {
                        if !is_ignored_by_patterns(&p, &ignore) {
                            changed.push(p);
                        }
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => break,
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                        running.store(false, Ordering::SeqCst);
                        return;
                    }
                }
            }

            if !added.is_empty() || !changed.is_empty() || !removed.is_empty() {
                callback(HmrUpdate {
                    added,
                    changed,
                    removed,
                    timestamp: SystemTime::now(),
                });
            }
        }
    }

    pub fn debounce_events(events: Vec<WatchEvent>, debounce: Duration) -> Vec<WatchEvent> {
        if events.is_empty() {
            return events;
        }
        let mut deduped = Vec::new();
        let mut last_event: Option<(WatchEvent, Instant)> = None;

        for event in events {
            if let Some((ref last, time)) = last_event {
                if time.elapsed() < debounce && std::mem::discriminant(&event) == std::mem::discriminant(last) {
                    continue;
                }
            }
            last_event = Some((event.clone(), Instant::now()));
            deduped.push(event);
        }

        deduped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debouncer_new() {
        let d = Debouncer::new(Duration::from_millis(300));
        assert_eq!(d.debounce, Duration::from_millis(300));
    }

    #[test]
    fn test_debounce_events_empty() {
        let result = Debouncer::debounce_events(vec![], Duration::from_millis(100));
        assert!(result.is_empty());
    }

    #[test]
    fn test_debounce_events_deduplicates() {
        let events = vec![
            WatchEvent::Modify(PathBuf::from("a.js")),
            WatchEvent::Modify(PathBuf::from("a.js")),
            WatchEvent::Create(PathBuf::from("b.js")),
        ];
        let result = Debouncer::debounce_events(events, Duration::from_millis(500));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_debounce_events_no_dedup() {
        let events = vec![
            WatchEvent::Create(PathBuf::from("a.js")),
            WatchEvent::Modify(PathBuf::from("b.js")),
        ];
        let result = Debouncer::debounce_events(events, Duration::from_millis(0));
        assert_eq!(result.len(), 2);
    }
}

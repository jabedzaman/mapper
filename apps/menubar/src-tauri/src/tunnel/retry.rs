use std::time::Duration;

/// Backoff schedule for auto-reconnect, indexed by attempt number (capped
/// at the last entry). After `MAX_RETRY_ATTEMPTS` consecutive failures we
/// give up rather than retry a permanently dead host forever.
const RETRY_DELAYS_SECS: [u64; 5] = [2, 4, 8, 16, 30];
pub const MAX_RETRY_ATTEMPTS: u32 = 6;

pub fn retry_delay(attempt: u32) -> Duration {
    let idx = (attempt as usize).min(RETRY_DELAYS_SECS.len() - 1);
    Duration::from_secs(RETRY_DELAYS_SECS[idx])
}

use crate::tunnel::types::TunnelState;

/// All valid state transitions for a tunnel.
/// Returns `Some(next_state)` if the transition is valid, `None` otherwise.
pub fn transition(current: &TunnelState, event: StateEvent) -> Option<TunnelState> {
    match (current, event) {
        // === From STOPPED ===
        (TunnelState::Stopped, StateEvent::StartRequested) => Some(TunnelState::Starting),

        // === From STARTING ===
        (TunnelState::Starting, StateEvent::HealthCheckPassed) => Some(TunnelState::Healthy),
        (TunnelState::Starting, StateEvent::ProcessDied) => Some(TunnelState::Reconnecting),
        (TunnelState::Starting, StateEvent::FatalError) => Some(TunnelState::Failed),
        (TunnelState::Starting, StateEvent::StopRequested) => Some(TunnelState::Stopped),

        // === From HEALTHY ===
        (TunnelState::Healthy, StateEvent::HealthCheckFailed) => Some(TunnelState::Degraded),
        (TunnelState::Healthy, StateEvent::ProcessDied) => Some(TunnelState::Reconnecting),
        (TunnelState::Healthy, StateEvent::StopRequested) => Some(TunnelState::Stopped),

        // === From DEGRADED ===
        (TunnelState::Degraded, StateEvent::HealthCheckPassed) => Some(TunnelState::Healthy),
        (TunnelState::Degraded, StateEvent::ProcessDied) => Some(TunnelState::Reconnecting),
        (TunnelState::Degraded, StateEvent::StopRequested) => Some(TunnelState::Stopped),

        // === From RECONNECTING ===
        (TunnelState::Reconnecting, StateEvent::StartRequested) => Some(TunnelState::Starting),
        (TunnelState::Reconnecting, StateEvent::FatalError) => Some(TunnelState::Failed),
        (TunnelState::Reconnecting, StateEvent::StopRequested) => Some(TunnelState::Stopped),

        // === From FAILED ===
        (TunnelState::Failed, StateEvent::StartRequested) => Some(TunnelState::Starting),
        (TunnelState::Failed, StateEvent::StopRequested) => Some(TunnelState::Stopped),

        // All other combinations are invalid.
        _ => None,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateEvent {
    StartRequested,
    StopRequested,
    HealthCheckPassed,
    HealthCheckFailed,
    ProcessDied,
    FatalError,
}

/// Calculates exponential backoff delay with jitter.
/// Returns delay in milliseconds capped at `max_delay_ms`.
pub fn backoff_delay_ms(attempt: u32, initial_ms: u64, max_ms: u64, multiplier: f64) -> u64 {
    if attempt == 0 {
        return initial_ms;
    }
    let raw = initial_ms as f64 * multiplier.powi(attempt as i32);
    // Add ±10% jitter to avoid thundering herd.
    let jitter = raw * 0.1 * (rand_jitter() - 0.5) * 2.0;
    ((raw + jitter) as u64).min(max_ms)
}

/// Simple pseudo-random jitter in [0.0, 1.0) using system time nanoseconds.
fn rand_jitter() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    (ns % 1_000_000) as f64 / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stopped_to_starting() {
        assert_eq!(
            transition(&TunnelState::Stopped, StateEvent::StartRequested),
            Some(TunnelState::Starting)
        );
    }

    #[test]
    fn healthy_to_degraded() {
        assert_eq!(
            transition(&TunnelState::Healthy, StateEvent::HealthCheckFailed),
            Some(TunnelState::Degraded)
        );
    }

    #[test]
    fn invalid_transition_is_none() {
        assert_eq!(
            transition(&TunnelState::Stopped, StateEvent::HealthCheckPassed),
            None
        );
    }

    #[test]
    fn backoff_grows() {
        let d0 = backoff_delay_ms(0, 1000, 60_000, 2.0);
        let d1 = backoff_delay_ms(1, 1000, 60_000, 2.0);
        let d2 = backoff_delay_ms(2, 1000, 60_000, 2.0);
        assert!(d0 <= d1);
        assert!(d1 <= d2);
    }

    #[test]
    fn backoff_is_capped() {
        let d = backoff_delay_ms(100, 1000, 60_000, 2.0);
        assert!(d <= 60_000);
    }
}

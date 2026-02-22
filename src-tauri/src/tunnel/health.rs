use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

/// Performs a TCP connection attempt to `host:port`.
/// Returns `true` if the connection succeeds within the given timeout.
pub async fn tcp_check(host: &str, port: u16, timeout_ms: u64) -> bool {
    let addr = format!("{host}:{port}");
    let dur = Duration::from_millis(timeout_ms);
    match timeout(dur, TcpStream::connect(&addr)).await {
        Ok(Ok(_stream)) => true,
        _ => false,
    }
}

/// Repeated health check runner. Calls `on_result` with the check outcome.
/// Stops when the `shutdown` channel receives a signal.
#[allow(dead_code)]
pub async fn run_health_check_loop<F>(
    host: String,
    port: u16,
    interval_ms: u64,
    timeout_ms: u64,
    mut shutdown: tokio::sync::oneshot::Receiver<()>,
    mut on_result: F,
) where
    F: FnMut(bool) + Send + 'static,
{
    loop {
        tokio::select! {
            _ = &mut shutdown => break,
            _ = tokio::time::sleep(Duration::from_millis(interval_ms)) => {
                let ok = tcp_check(&host, port, timeout_ms).await;
                on_result(ok);
            }
        }
    }
}

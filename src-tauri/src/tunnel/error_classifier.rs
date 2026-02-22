use crate::tunnel::types::TunnelErrorKind;

/// Classifies raw SSH stderr/stdout strings into a structured error kind.
pub fn classify(output: &str) -> TunnelErrorKind {
    let lower = output.to_lowercase();

    if lower.contains("broken pipe") || lower.contains("connection reset by peer") {
        return TunnelErrorKind::BrokenPipe;
    }
    if lower.contains("connection timed out")
        || lower.contains("timed out")
        || lower.contains("operation timed out")
    {
        return TunnelErrorKind::ConnectionTimeout;
    }
    if lower.contains("permission denied (publickey")
        || lower.contains("permission denied (password")
        || lower.contains("authentication failed")
        || lower.contains("too many authentication failures")
    {
        return TunnelErrorKind::AuthFailure;
    }
    if lower.contains("address already in use")
        || lower.contains("bind: address already in use")
        || lower.contains("error: bind")
    {
        return TunnelErrorKind::PortInUse;
    }
    if lower.contains("no route to host")
        || lower.contains("network is unreachable")
        || lower.contains("network unreachable")
    {
        return TunnelErrorKind::NetworkUnreachable;
    }
    if lower.contains("could not resolve hostname")
        || lower.contains("name or service not known")
        || lower.contains("nodename nor servname provided")
    {
        return TunnelErrorKind::UnknownHost;
    }
    if lower.contains("host unreachable")
        || lower.contains("connection refused")
        || lower.contains("no such host")
    {
        return TunnelErrorKind::HostUnreachable;
    }
    if lower.contains("permission denied") {
        return TunnelErrorKind::PermissionDenied;
    }

    TunnelErrorKind::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_broken_pipe() {
        assert_eq!(classify("Broken pipe"), TunnelErrorKind::BrokenPipe);
    }

    #[test]
    fn classifies_auth_failure() {
        assert_eq!(
            classify("Permission denied (publickey,gssapi-keyex,gssapi-with-mic)"),
            TunnelErrorKind::AuthFailure
        );
    }

    #[test]
    fn classifies_port_in_use() {
        assert_eq!(
            classify("Error: bind: address already in use"),
            TunnelErrorKind::PortInUse
        );
    }

    #[test]
    fn classifies_unknown() {
        assert_eq!(classify("something completely unexpected"), TunnelErrorKind::Unknown);
    }
}

use crate::tunnel::types::TunnelConfig;
use tokio::process::Child;
use std::process::Stdio;

/// Creates a temporary askpass script that echoes the password and then deletes itself.
///
/// `SSH_ASKPASS` is an OpenSSH built-in: when set, SSH calls that executable to obtain
/// a password instead of reading from the terminal.  No third-party tools are required.
///
/// The script self-destructs on first execution so no plaintext credentials persist on
/// disk after the SSH authentication handshake completes.
fn create_askpass_script(password: &str) -> std::io::Result<std::path::PathBuf> {
    use std::os::unix::fs::PermissionsExt;

    // Single-quote–escape the password for safe embedding in a POSIX shell script.
    let escaped = password.replace('\'', "'\\''");
    let script = format!(
        "#!/bin/sh\nrm -f \"$0\"\nprintf '%s\\n' '{}'\n",
        escaped
    );

    let path = std::env::temp_dir().join(format!(
        ".stg_askpass_{}.sh",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
    ));
    std::fs::write(&path, script)?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))?;
    Ok(path)
}

/// Builds the SSH argument list for a forwarding tunnel.
///
/// Produces: `ssh -N -T -o ... -L local:remote user@host -p port [-i identity] [extra]`
pub fn build_ssh_args(config: &TunnelConfig) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();

    // Do not allocate a TTY and do not execute a remote command.
    args.push("-N".to_string());
    args.push("-T".to_string());

    // Keep-alive: abort if the server stops responding after 3 × 10s.
    args.push("-o".to_string());
    args.push("ServerAliveInterval=10".to_string());
    args.push("-o".to_string());
    args.push("ServerAliveCountMax=3".to_string());

    // Fail immediately if the port-forward binding fails.
    args.push("-o".to_string());
    args.push("ExitOnForwardFailure=yes".to_string());

    // BatchMode=yes disables password prompts and is appropriate only for
    // key-based auth. When a password is supplied we omit it so sshpass can
    // inject the credential; when neither key nor password is configured we
    // still omit it so the user gets a clear "Permission denied" rather than
    // a silent failure.
    let using_key = config.identity_file.as_deref().is_some_and(|s| !s.is_empty());
    let using_password = config.ssh_password.as_deref().is_some_and(|s| !s.is_empty());
    if using_key && !using_password {
        args.push("-o".to_string());
        args.push("BatchMode=yes".to_string());
    }

    // Host-key verification.
    if !config.strict_host_checking {
        args.push("-o".to_string());
        args.push("StrictHostKeyChecking=no".to_string());
        args.push("-o".to_string());
        args.push("UserKnownHostsFile=/dev/null".to_string());
    }

    // Local forwarding specification.
    args.push("-L".to_string());
    args.push(format!(
        "127.0.0.1:{}:{}:{}",
        config.local_port, config.remote_host, config.remote_port
    ));

    // SSH port.
    args.push("-p".to_string());
    args.push(config.ssh_port.to_string());

    // Identity file.
    if let Some(ref identity) = config.identity_file {
        if !identity.is_empty() {
            args.push("-i".to_string());
            args.push(identity.clone());
        }
    }

    // Extra user-defined flags.
    for arg in &config.extra_args {
        args.push(arg.clone());
    }

    // user@host must be last.
    args.push(format!("{}@{}", config.ssh_user, config.ssh_host));

    args
}

/// Spawns the SSH process and returns the `Child` handle.
/// stdout/stderr are piped so we can read them for error classification.
///
/// When `ssh_password` is set the password is delivered via the `SSH_ASKPASS`
/// mechanism — a self-deleting temporary shell script that OpenSSH calls during
/// authentication.  No third-party tools (sshpass, expect, …) are needed.
pub fn spawn_ssh(config: &TunnelConfig) -> std::io::Result<Child> {
    let ssh_args = build_ssh_args(config);
    let using_password = config.ssh_password.as_deref().is_some_and(|s| !s.is_empty());

    if using_password {
        let password = config.ssh_password.as_deref().unwrap_or_default();
        let askpass = create_askpass_script(password)?;
        tokio::process::Command::new("ssh")
            .args(&ssh_args)
            // SSH_ASKPASS: path to the program SSH calls to read the password.
            .env("SSH_ASKPASS", &askpass)
            // SSH_ASKPASS_REQUIRE=force tells OpenSSH >= 8.4 to call the askpass
            // program unconditionally, even without a DISPLAY variable set.
            .env("SSH_ASKPASS_REQUIRE", "force")
            // Fallback for older OpenSSH versions: a non-empty DISPLAY triggers the
            // askpass path even on headless systems.
            .env("DISPLAY", std::env::var("DISPLAY").unwrap_or_else(|_| ":0".into()))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
    } else {
        tokio::process::Command::new("ssh")
            .args(&ssh_args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
    }
}

/// Collects the stderr output of a completed or failed SSH process.
pub async fn collect_stderr(child: &mut Child) -> String {
    use tokio::io::AsyncReadExt;
    if let Some(stderr) = child.stderr.take() {
        let mut buf = String::new();
        let mut reader = tokio::io::BufReader::new(stderr);
        let _ = reader.read_to_string(&mut buf).await;
        buf
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tunnel::types::{HealthCheckConfig, ReconnectConfig, TunnelConfig};

    fn sample_config() -> TunnelConfig {
        TunnelConfig {
            name: "test".into(),
            ssh_host: "example.com".into(),
            ssh_port: 22,
            ssh_user: "admin".into(),
            local_port: 5432,
            remote_host: "db.internal".into(),
            remote_port: 5432,
            identity_file: None,
            ssh_password: None,
            strict_host_checking: true,
            extra_args: vec![],
            reconnect: ReconnectConfig::default(),
            health_check: HealthCheckConfig::default(),
        }
    }

    #[test]
    fn args_include_forwarding_spec() {
        let args = build_ssh_args(&sample_config());
        let fwd_idx = args.iter().position(|a| a == "-L").expect("-L flag missing");
        assert_eq!(args[fwd_idx + 1], "127.0.0.1:5432:db.internal:5432");
    }

    #[test]
    fn args_include_user_at_host_last() {
        let args = build_ssh_args(&sample_config());
        assert_eq!(args.last().unwrap(), "admin@example.com");
    }

    #[test]
    fn args_include_identity_when_set() {
        let mut cfg = sample_config();
        cfg.identity_file = Some("~/.ssh/id_rsa".into());
        let args = build_ssh_args(&cfg);
        let i_idx = args.iter().position(|a| a == "-i").expect("-i flag missing");
        assert_eq!(args[i_idx + 1], "~/.ssh/id_rsa");
    }

    #[test]
    fn batch_mode_set_only_with_key_no_password() {
        let mut cfg = sample_config();
        cfg.identity_file = Some("~/.ssh/id_rsa".into());
        let args = build_ssh_args(&cfg);
        assert!(args.windows(2).any(|w| w[0] == "-o" && w[1] == "BatchMode=yes"));
    }

    #[test]
    fn batch_mode_absent_with_password() {
        let mut cfg = sample_config();
        cfg.ssh_password = Some("secret".into());
        let args = build_ssh_args(&cfg);
        assert!(!args.windows(2).any(|w| w[0] == "-o" && w[1] == "BatchMode=yes"));
    }

    #[test]
    fn strict_host_checking_disabled_adds_flags() {
        let mut cfg = sample_config();
        cfg.strict_host_checking = false;
        let args = build_ssh_args(&cfg);
        assert!(args.windows(2).any(|w| w[0] == "-o" && w[1] == "StrictHostKeyChecking=no"));
        assert!(args.windows(2).any(|w| w[0] == "-o" && w[1].contains("UserKnownHostsFile")));
    }
}

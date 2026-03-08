use crate::discover::registry;

/// Run the `clov route` command.
///
/// Prints the CLOV-rewritten command to stdout and exits 0.
/// Exits 1 (without output) if the command has no CLOV equivalent.
///
/// Used by shell hooks to rewrite commands transparently:
/// ```bash
/// REWRITTEN=$(clov route "$CMD") || exit 0
/// [ "$CMD" = "$REWRITTEN" ] && exit 0  # already CLOV, skip
/// ```
pub fn run(cmd: &str) -> anyhow::Result<()> {
    match registry::rewrite_command(cmd) {
        Some(rewritten) => {
            print!("{}", rewritten);
            Ok(())
        }
        None => {
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_supported_command_succeeds() {
        // We can't easily test exit code here, but we can test the registry directly
        assert!(registry::rewrite_command("git status").is_some());
    }

    #[test]
    fn test_run_unsupported_returns_none() {
        assert!(registry::rewrite_command("terraform plan").is_none());
    }

    #[test]
    fn test_run_already_clov_returns_some() {
        assert_eq!(
            registry::rewrite_command("clov git status"),
            Some("clov git status".into())
        );
    }
}

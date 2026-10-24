//! Turns low-level transfer errors into messages a person can act on.

/// Maps an error's full text (e.g. `format!("{e:#}")`) to a short, plain
/// explanation. The original text should still be logged for debugging.
pub fn friendly_message(error: &str) -> String {
    let e = error.to_lowercase();
    let has = |needles: &[&str]| needles.iter().any(|n| e.contains(n));

    let msg = if has(&["cancel"]) {
        "The download was cancelled."
    } else if has(&["code not found"]) {
        "That code wasn't found or has expired. Ask the sender for a new one."
    } else if has(&["code server"]) {
        "The code server can't be reached right now. Ask the sender for the ticket or QR code instead, or try again later."
    } else if has(&["no space left", "disk full", "not enough space"]) {
        "There isn't enough free space to save these files."
    } else if has(&[
        "permission denied",
        "read-only",
        "operation not permitted",
        "access denied",
    ]) {
        "Can't save to that folder. Choose a different save location and try again."
    } else if has(&[
        "invalid ticket",
        "invalid blob",
        "failed to parse",
        "invalid base32",
        "invalid length",
        "unexpected end",
    ]) {
        "That doesn't look like a valid ticket or code. Check it and try again."
    } else if has(&[
        "timed out",
        "timeout",
        "connect",
        "unreachable",
        "network or io",
        "relay",
        "no route",
        "dns",
        "connection",
        "reset by peer",
        "broken pipe",
        "send side no longer has",
        "stalled",
        "lost",
    ]) {
        "Couldn't reach the sender. Check that you're both online and that the sender's app is still open and sharing, then try again."
    } else {
        "The transfer failed. Please try again."
    };
    msg.to_string()
}

#[cfg(test)]
mod tests {
    use super::friendly_message;

    #[test]
    fn unknown_code() {
        assert!(friendly_message("Failed to get blob: code not found").contains("wasn't found"));
    }

    #[test]
    fn server_down_is_not_called_a_sender_problem() {
        let m = friendly_message("Failed to get blob: code server unreachable (timed out)");
        assert!(m.contains("code server"));
        assert!(!m.contains("reach the sender"), "{m}");
    }

    #[test]
    fn connection_problems_mention_the_sender() {
        for e in [
            "Timed out connecting to the sender",
            "generic network error: connection reset by peer",
            "A network or IO operation failed",
        ] {
            assert!(friendly_message(e).contains("reach the sender"), "{e}");
        }
    }

    #[test]
    fn a_stalled_transfer_is_a_connection_problem() {
        assert!(friendly_message("Transfer stalled: no data for 30 seconds")
            .contains("reach the sender"));
    }

    #[test]
    fn disk_and_permission_errors() {
        assert!(friendly_message("No space left on device (os error 28)").contains("free space"));
        assert!(
            friendly_message("Permission denied (os error 13)").contains("different save location")
        );
    }

    #[test]
    fn bad_ticket() {
        assert!(friendly_message("failed to parse ticket").contains("valid ticket"));
    }

    #[test]
    fn cancel_wins_over_other_words() {
        assert!(friendly_message("export cancelled: connection closed").contains("cancelled"));
    }

    #[test]
    fn unknown_errors_get_a_generic_message_not_the_raw_text() {
        let m = friendly_message("some internal explosion 0xdeadbeef");
        assert_eq!(m, "The transfer failed. Please try again.");
    }
}

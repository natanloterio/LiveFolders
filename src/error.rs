pub fn format_error(code: &str, msg: &str) -> String {
    format!("[ERROR:{}] {}", code, msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_error_produces_bracketed_prefix() {
        assert_eq!(
            format_error("TIMEOUT", "handler exceeded 30s"),
            "[ERROR:TIMEOUT] handler exceeded 30s"
        );
    }

    #[test]
    fn format_error_with_empty_message() {
        assert_eq!(format_error("SPAWN", ""), "[ERROR:SPAWN] ");
    }

    #[test]
    fn format_error_with_multiline_message() {
        let msg = "line 1\nline 2";
        assert_eq!(format_error("HANDLER", msg), "[ERROR:HANDLER] line 1\nline 2");
    }
}

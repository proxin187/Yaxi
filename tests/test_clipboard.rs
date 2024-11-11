use std::time::SystemTime;

use yaxi::clipboard::Clipboard;

#[cfg(test)]
#[cfg(feature = "clipboard")]
mod tests {
    use super::*;

    fn test_clipboard_read_text() {
        let clipboard = Clipboard::new(None).unwrap();

        let result = clipboard.get_text();

        assert!(result.is_ok());
    }

    fn test_clipboard_write_text() {
        let clipboard = Clipboard::new(None).unwrap();

        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let excepted = format!("test {}", now);

        let result = clipboard.set_text(&excepted);
        assert!(result.is_ok());
    }

    fn test_clipboard_text_consistency() {
        let clipboard = Clipboard::new(None).unwrap();

        let time = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let excepted = format!("test-{}", time);

        let result = clipboard.set_text(&excepted);
        assert!(result.is_ok());

        let text = clipboard.get_text().unwrap();
        assert_eq!(Some(excepted), text);
    }

    #[test]
    fn run() {
        test_clipboard_read_text();

        test_clipboard_write_text();

        test_clipboard_text_consistency();
    }
}

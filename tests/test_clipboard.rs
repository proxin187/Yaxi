use std::time::SystemTime;

use serial_test::serial;
use yaxi::clipboard::Clipboard;

#[cfg(test)]
#[cfg(feature = "clipboard")]
mod tests {

    use super::*;

    #[test]
    #[serial]
    fn test_clipboard_write_html() {
        let clipboard = Clipboard::new(None).unwrap();

        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let html = format!("<html><body>test {}</body></html>", now);
        let alt = Some(format!("test {}", now));

        let result = clipboard.set_html(&html, alt.as_deref());

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_clipboard_read_html() {
        let clipboard = Clipboard::new(None).unwrap();

        let result = clipboard.get_html();
        println!("{:?}", result);

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_clipboard_clear() {
        let clipboard = Clipboard::new(None).unwrap();

        let result = clipboard.get_targets();
        assert!(result.is_ok());

        let result = clipboard.clear();
        assert!(result.is_ok());

        println!("{:?}", result);
        let result = clipboard.get_targets();
        assert!(result.is_ok());
        assert_eq!(0, result.unwrap().len());
    }

    #[test]
    #[serial]
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
        assert_eq!(Some(excepted.clone()), text);
    }

    #[test]
    #[serial]
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

    #[test]
    #[serial]
    fn test_clipboard_read_text() {
        let clipboard = Clipboard::new(None).unwrap();

        let result = clipboard.get_text();
        println!("{:?}", result);

        assert!(result.is_ok());
    }
}

use std::time::SystemTime;

use yaxi::clipboard::Clipboard;


#[cfg(test)]
#[cfg(feature = "clipboard")]
mod tests {
    use super::*;

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

    fn test_clipboard_read_html() {
        let clipboard = Clipboard::new(None).unwrap();

        let result = clipboard.get_html();

        assert!(result.is_ok());
    }

    fn test_clipboard_clear() {
        let clipboard = Clipboard::new(None).unwrap();

        let result = clipboard.get_targets();
        println!("targets: {:?}", result);
        assert!(result.is_ok());

        let result = clipboard.clear();
        println!("cleared");
        assert!(result.is_ok());

        let result = clipboard.get_targets();
        println!("targets: {:?}", result);
        assert!(result.is_ok());

        assert_eq!(0, result.unwrap().len());
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

        assert_eq!(Some(excepted.clone()), text);
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

    fn test_clipboard_read_text() {
        let clipboard = Clipboard::new(None).unwrap();

        let result = clipboard.get_text();

        assert!(result.is_ok());
    }

    #[test]
    fn test_all() {
        println!("read html");
        test_clipboard_read_html();

        println!("read text");
        test_clipboard_read_text();

        // TODO: the reason why write html, write text and text consistency use a long time is
        // because they try to hand it over to the clipboard manager

        println!("write html");
        test_clipboard_write_html();

        println!("write text");
        test_clipboard_write_text();

        println!("text consistency");
        test_clipboard_text_consistency();

        println!("clear");
        test_clipboard_clear();
    }
}

use std::time::SystemTime;

use yaxi::display;


#[cfg(test)]
#[cfg(feature = "clipboard")]
mod tests {
    use super::*;

    /*
    #[test]
    fn test_clipboard_read_text() {
        let mut display = display::open_unix(0).unwrap();
        let mut clipboard = display.clipboard().unwrap();

        let result = clipboard.get_text();

        assert!(result.is_ok());
    }
    */

    // TODO: there is a problem here now
    #[test]
    fn test_clipboard_write_text() {
        let mut display = display::open_unix(0).unwrap();
        let mut clipboard = display.clipboard().unwrap();

        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let excepted = format!("test {}", now);

        let result = clipboard.set_text(&excepted);
        assert!(result.is_ok());
    }

    /*
    #[test]
    fn test_clipboard_text_consistency() {
        let mut display = display::open_unix(0).unwrap();
        let mut clipboard = display.clipboard().unwrap();

        let time = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let excepted = format!("test-{}", time);

        let result = clipboard.set_text(&excepted);
        assert!(result.is_ok());

        let text = clipboard.get_text().unwrap();
        assert_eq!(excepted, text);
    }
    */
}



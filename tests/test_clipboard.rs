use std::time::SystemTime;

use yaxi::display;

// TODO: looks like the tests arent running properly


#[cfg(test)]
#[cfg(feature = "clipboard")]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_read_text() {
        println!("read text");

        let mut display = display::open_unix(0).unwrap();
        let mut clipboard = display.clipboard().unwrap();

        println!("read text 2");

        let result = clipboard.get_text();

        println!("result: {:?}", result);

        assert!(result.is_ok());

        // TODO: it doesnt close, this is most likely because we are waiting for the thread to
        // close
        println!("result: {:?}", result);
    }

    /*
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
    */

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



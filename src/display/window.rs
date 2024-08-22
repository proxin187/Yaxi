use super::*;


pub struct Window<T> {
    stream: Stream<T>,
    id: u32,
}

impl<T> Window<T> where T: Send + Sync + Read + Write + TryClone {
    fn new(inner: T, id: u32) -> Window<T> {
        Window {
            stream: Stream::new(inner),
            id,
        }
    }

    pub fn grab_key(&mut self) {
    }
}



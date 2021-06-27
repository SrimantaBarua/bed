/// This closure takes a mutable reference to data, takes ownership of the data, and calls a
/// closure with the owned data. The closure must return data of type T, which this function then
/// assigns back to the mutable reference.
pub(crate) fn take_mut<F, T>(mut_ref: &mut T, mut f: F)
where
    F: FnMut(T) -> T,
{
    let ptr = mut_ref as *mut T;
    let t = unsafe { ptr.read() };
    let new_t = f(t);
    unsafe { ptr.write(new_t) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::{channel, Sender};
    use std::thread;

    struct TestStruct(usize, Sender<usize>);

    impl Drop for TestStruct {
        fn drop(&mut self) {
            self.1.send(self.0).unwrap();
        }
    }

    #[test]
    fn drops() {
        let (tx, rx) = channel();
        thread::spawn(move || {
            let mut data = TestStruct(1, tx.clone());
            take_mut(&mut data, |x| {
                x.1.send(x.0).unwrap();
                return TestStruct(2, tx.clone());
            });
            assert_eq!(data.0, 2);
        });
        assert_eq!(rx.recv().unwrap(), 1);
        assert_eq!(rx.recv().unwrap(), 1);
        assert_eq!(rx.recv().unwrap(), 2);
        assert!(rx.recv().is_err());
    }
}

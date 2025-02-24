use std::{ptr::NonNull, sync::atomic::AtomicUsize};

pub struct OutOfBandBuffer {
    inner: NonNull<BufferInner>,
}

struct BufferInner {
    length: AtomicUsize,
    buffer: [u8; 1024],
}

impl OutOfBandBuffer {
    pub fn new() -> Self {
        let inner = Box::new(BufferInner {
            length: Default::default(),
            buffer: [0; 1024],
        });
        Self {
            inner: NonNull::from(Box::leak(inner)),
        }
    }

    pub fn has_data(&self) -> bool {
        unsafe {
            self.inner
                .as_ref()
                .length
                .load(std::sync::atomic::Ordering::SeqCst)
                > 0
        }
    }

    pub fn read(&self) -> &[u8] {
        let buffer = unsafe { self.inner.as_ref().buffer.as_ptr() };
        let length = unsafe {
            self.inner
                .as_ref()
                .length
                .load(std::sync::atomic::Ordering::SeqCst)
        };

        unsafe { std::slice::from_raw_parts(buffer, length) }
    }

    pub fn done(&self) {
        unsafe {
            self.inner
                .as_ref()
                .length
                .store(0, std::sync::atomic::Ordering::SeqCst)
        }
    }

    pub fn write(&mut self, data: &[u8]) -> bool {
        let success = unsafe {
            self.inner
                .as_ref()
                .length
                .compare_exchange(
                    0,
                    data.len(),
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                )
                .is_ok()
        };

        if !success {
            return false;
        }

        unsafe {
            self.inner.as_mut().buffer[..data.len()].copy_from_slice(data);
        }

        true
    }
}

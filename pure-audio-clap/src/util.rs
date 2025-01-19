use std::{ffi::c_char, fmt::Write};

struct WritableCChars<'a>{
    inner: &'a mut [c_char],
    bytes_written: usize
}

impl<'a> Write for WritableCChars<'a> {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let value = s.as_bytes();
        let len = value.len();
        let available_len = self.inner.len() - self.bytes_written - 1; // 1 byte for null-termination
        if len > available_len {
            return Err(std::fmt::Error);
        }
        let inner_ptr = self.inner as *mut [c_char];
        unsafe {
            let dst = inner_ptr.cast::<u8>().add(self.bytes_written);
            core::ptr::copy_nonoverlapping(value.as_ptr(), dst, len);
            dst.add(len).write(0); // null-terminator
        }
        self.bytes_written += len;
        Ok(())
    }
}

pub trait Writable {
    fn writable(&mut self) -> impl Write;
}

impl<const N: usize> Writable for [c_char; N] {
    #[inline]
    fn writable(&mut self) -> impl Write {
        WritableCChars {
            inner: self,
            bytes_written: 0
        }
    }
}

impl Writable for [c_char] {
    fn writable(&mut self) -> impl Write {
        WritableCChars {
            inner: self,
            bytes_written: 0
        }
    }
}
use std::fmt::Display;

use super::{NetworkBuffer, ProtocolBuffer};

#[derive(Default, Clone, Copy)]
struct Ref {
    start: u32,
    length: u32,
}

pub struct HttpReq<P: ProtocolBuffer> {
    inner: P,
    data: Ref,
    method: Ref,
    path: Ref,
    version: Ref,
    headers: Vec<Ref>,
}

pub struct HttpResp {
    inner: PackedHttpResp<NetworkBuffer>,
}

pub struct PackedHttpResp<P: ProtocolBuffer> {
    inner: P,
    data: Ref,
    version: Ref,
    code: Ref,
    code_status: Ref,
    headers: Vec<Ref>,
}

const SEPARATOR: [u8; 2] = [b'\r', b'\n'];

impl<P: ProtocolBuffer> HttpReq<P> {
    pub fn into_inner(self) -> P {
        self.inner
    }

    pub fn parse(p: P) -> Self {
        let buffer = p.buf();
        let mut lines = memchr::memmem::find_iter(buffer, &SEPARATOR);
        let first_line_end = lines.next().unwrap();

        let first_line = &buffer[..first_line_end];
        let mut split = memchr::memchr_iter(b' ', first_line);
        let (method, index) = next(0, &mut split, first_line_end);
        let (path, index) = next(index, &mut split, first_line_end);
        let (version, _index) = next(index, &mut split, first_line_end);

        let mut line_index = first_line_end + SEPARATOR.len();

        let mut headers = vec![];
        for line in lines {
            let line_buf = &buffer[line_index..line];
            if line_buf.is_empty() {
                break;
            }

            let length = line - line_index;
            let r = Ref {
                start: line_index as u32,
                length: length as u32,
            };
            headers.push(r);
            line_index = line + SEPARATOR.len();
            // let sep = memchr::memchr(b':', line_buf).unwrap();
        }

        let length = buffer.len() - line_index;
        let data = Ref {
            start: line_index as u32,
            length: length as u32,
        };

        Self {
            inner: p,
            data,
            method,
            path,
            version,
            headers,
        }
    }

    pub fn method(&self) -> http::Method {
        let start = self.method.start as usize;
        let end = start + self.method.length as usize;
        http::Method::from_bytes(&self.inner.buf()[start..end]).unwrap()
    }

    pub fn path(&self) -> &str {
        self.read(self.path)
    }

    pub fn version(&self) -> &str {
        self.read(self.version)
    }

    pub fn headers(&self) -> impl Iterator<Item = &str> {
        self.headers.iter().map(|&h| self.read(h))
    }

    pub fn data(&self) -> &str {
        self.read(self.data)
    }

    fn read(&self, data_ref: Ref) -> &str {
        let start = data_ref.start as usize;
        let end = start + data_ref.length as usize;
        unsafe { std::str::from_utf8_unchecked(&self.inner.buf()[start..end]) }
    }
}

impl HttpResp {
    pub fn ok() -> Self {
        const RESPONSE: &[u8; 38] = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";

        let mut buf = NetworkBuffer::new(38);
        buf.extend_from_slice(RESPONSE);

        let inner = PackedHttpResp::parse(buf);

        Self { inner }
    }

    pub fn to_buf(self) -> NetworkBuffer {
        self.inner.inner
    }
}

impl<P: ProtocolBuffer> PackedHttpResp<P> {
    pub fn into_inner(self) -> P {
        self.inner
    }

    pub fn parse(p: P) -> Self {
        let buffer = p.buf();
        let mut lines = memchr::memmem::find_iter(buffer, &SEPARATOR);
        let first_line_end = lines.next().unwrap();

        let first_line = &buffer[..first_line_end];
        let mut split = memchr::memchr_iter(b' ', first_line);
        let (version, index) = next(0, &mut split, first_line_end);
        let (code, index) = next(index, &mut split, first_line_end);
        let (code_status, _index) = next(index, &mut split, first_line_end);

        let mut line_index = first_line_end + SEPARATOR.len();

        let mut headers = vec![];
        for line in lines {
            let line_buf = &buffer[line_index..line];
            if line_buf.is_empty() {
                break;
            }

            let length = line - line_index;
            let r = Ref {
                start: line_index as u32,
                length: length as u32,
            };
            headers.push(r);
            line_index = line + SEPARATOR.len();
            // let sep = memchr::memchr(b':', line_buf).unwrap();
        }

        let length = buffer.len() - line_index;
        let data = Ref {
            start: line_index as u32,
            length: length as u32,
        };

        Self {
            inner: p,
            data,
            version,
            code,
            code_status,
            headers,
        }
    }

    pub fn code(&self) -> &str {
        self.read(self.code)
    }

    pub fn code_status(&self) -> &str {
        self.read(self.code_status)
    }

    pub fn version(&self) -> &str {
        self.read(self.version)
    }

    pub fn headers(&self) -> impl Iterator<Item = &str> {
        self.headers.iter().map(|&h| self.read(h))
    }

    pub fn data(&self) -> &str {
        self.read(self.data)
    }

    fn read(&self, data_ref: Ref) -> &str {
        let start = data_ref.start as usize;
        let end = start + data_ref.length as usize;

        unsafe { std::str::from_utf8_unchecked(&self.inner.buf()[start..end]) }
    }
}

fn next(index: u32, reader: &mut memchr::Memchr<'_>, line_end: usize) -> (Ref, u32) {
    let next = match reader.next() {
        Some(next) => next,
        None => line_end,
    };

    let r = Ref {
        start: index,
        length: (next as u32 - index),
    };
    let index = index + r.length + 1;
    (r, index)
}

impl<P: ProtocolBuffer> Display for HttpReq<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "HttpReq")?;
        writeln!(f, "Method: {}", self.method())?;
        writeln!(f, "Path: {}", self.path())?;
        writeln!(f, "Version: {}", self.version())?;
        for header in self.headers() {
            writeln!(f, "- {}", header)?;
        }

        writeln!(f, "Data: {}", self.data())
    }
}

impl<P: ProtocolBuffer> Display for PackedHttpResp<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "HttpResp")?;
        writeln!(f, "Code: {}", self.code())?;
        writeln!(f, "Status: {}", self.code_status())?;
        writeln!(f, "Version: {}", self.version())?;
        for header in self.headers() {
            writeln!(f, "- {}", header)?;
        }

        writeln!(f, "Data: {}", self.data())
    }
}

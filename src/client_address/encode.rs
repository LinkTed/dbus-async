use crate::client_address::ClientAddress;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[inline]
fn is_optionally_escaped(c: char) -> bool {
    // *?
    // [-0-9A-Za-z_/.\]
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '/' || c == '.' || c == '\\'
}

fn to_hex(b: u8) -> char {
    match b {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
        4 => '4',
        5 => '5',
        6 => '6',
        7 => '7',
        8 => '8',
        9 => '9',
        10 => 'a',
        11 => 'b',
        12 => 'c',
        13 => 'd',
        14 => 'e',
        15 => 'f',
        b => panic!("This should not happend: {}", b),
    }
}

fn add_hex(s: &mut String, b: u8) {
    s.push('%');
    s.push(to_hex((b & 0b1111_0000) >> 4));
    s.push(to_hex(b & 0b0000_1111));
}

fn escape_unix_path(path: &str) -> String {
    let mut result = String::new();
    for c in path.chars() {
        if is_optionally_escaped(c) {
            result.push(c);
        } else {
            let mut bytes = [0; 4];
            c.encode_utf8(&mut bytes[..]);
            for b in &bytes[..c.len_utf8()] {
                add_hex(&mut result, *b);
            }
        }
    }
    result
}

impl ClientAddress {
    pub fn encode(server_addresses: &[ClientAddress]) -> String {
        let mut result = String::new();
        let mut iter = server_addresses.iter();
        // Get the first address
        match iter.next() {
            Some(server_address) => result += &server_address.to_string(),
            None => return result,
        }
        // Get the remaining addresses and add the seperator
        for server_address in iter {
            result.push(';');
            result += &server_address.to_string();
        }

        result
    }
}

impl Display for ClientAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ClientAddress::UnixPath(path) => write!(f, "unix:path={}", escape_unix_path(path)),
            ClientAddress::UnixAbstract(path) => {
                write!(f, "unix:abstract={}", escape_unix_path(path))
            }
            ClientAddress::Tcp(socket_address) => write!(
                f,
                "tcp:host={},port={}",
                socket_address.ip(),
                socket_address.port()
            ),
        }
    }
}

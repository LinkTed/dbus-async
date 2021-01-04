use crate::client_address::ClientAddress;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::num::ParseIntError;
use std::string::FromUtf8Error;
use thiserror::Error;

/// An enum representing all errors, which can occur during decoding a [client address].
///
/// [client address]: https://dbus.freedesktop.org/doc/dbus-specification.html#addresses
#[derive(Debug, Clone, Error)]
pub enum ClientAddressDecodeError {
    #[error("Could not decode unix address: {0}")]
    UnixDecodeError(#[from] UnixDecodeError),
    #[error("Could not decode socket address: {0}")]
    SocketAddressDecodeError(#[from] SocketAddressDecodeError),
    #[error("Unknown type")]
    Type,
    #[error("The address contain a none ASCII character")]
    NonAsciiChar,
}

/// An enum representing all errors, which can occur during decoding an unix path of a
/// [client address]. For example: `unix:path=/tmp/dbus/path`.
///
/// [client address]: https://dbus.freedesktop.org/doc/dbus-specification.html#addresses
#[derive(Debug, Clone, Error)]
pub enum UnixDecodeError {
    #[error("Unknown char: 0x{0:02x}")]
    UnescapeChar(u8),
    #[error("State is not Normal: {0}")]
    UnescapeState(UnescapeState),
    #[error("UTF-8: {0}")]
    UnescapeFromUtf8Error(#[from] FromUtf8Error),
    #[error("Unknown hex: {0}")]
    UnescapeHex(u8),
    #[error("Unknown type")]
    Type,
}

/// An enum representing all errors, which can occur during decoding a socket address of a
/// [client address]. For example: `tcp:host=127.0.0.1,port=30900`.
///
/// [client address]: https://dbus.freedesktop.org/doc/dbus-specification.html#addresses
#[derive(Debug, Clone, Error)]
pub enum SocketAddressDecodeError {
    #[error("The socket address has {0} element")]
    VecLen(usize),
    #[error("Host parse: {0}")]
    HostParseError(#[from] AddrParseError),
    #[error("Port parse: {0}")]
    PortParseError(#[from] ParseIntError),
    #[error("Unknown host")]
    UnknownHost,
    #[error("Unknown port")]
    UnknownPort,
    #[error("Unknown element")]
    UnknownElement,
}

/// The states of the unescape machine.
#[derive(Debug, Clone)]
pub enum UnescapeState {
    Normal,
    FirstHex,
    SecondHex(u8),
}

impl Display for UnescapeState {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            UnescapeState::Normal => write!(f, "Normal"),
            UnescapeState::FirstHex => write!(f, "First Hex"),
            UnescapeState::SecondHex(h) => write!(f, "Second Hex: 0x{:02x}", h),
        }
    }
}

#[inline]
fn is_optionally_escaped(c: u8) -> bool {
    // *?
    // [-0-9A-Za-z_/.\]
    c.is_ascii_alphanumeric() || c == b'-' || c == b'_' || c == b'/' || c == b'.' || c == b'\\'
}

fn to_hex(c: u8) -> Result<u8, UnixDecodeError> {
    match c {
        b'0' => Ok(0),
        b'1' => Ok(1),
        b'2' => Ok(2),
        b'3' => Ok(3),
        b'4' => Ok(4),
        b'5' => Ok(5),
        b'6' => Ok(6),
        b'7' => Ok(7),
        b'8' => Ok(8),
        b'9' => Ok(9),
        b'a' | b'A' => Ok(10),
        b'b' | b'B' => Ok(11),
        b'c' | b'C' => Ok(12),
        b'd' | b'D' => Ok(13),
        b'e' | b'E' => Ok(14),
        b'f' | b'F' => Ok(15),
        c => Err(UnixDecodeError::UnescapeHex(c)),
    }
}

fn decode_unix_path(path: &str) -> Result<String, UnixDecodeError> {
    let mut result: Vec<u8> = Vec::new();
    let mut decode_state = UnescapeState::Normal;
    for c in path.bytes() {
        match decode_state {
            UnescapeState::Normal => {
                if is_optionally_escaped(c) {
                    result.push(c);
                } else if c == b'%' {
                    decode_state = UnescapeState::FirstHex;
                } else {
                    return Err(UnixDecodeError::UnescapeChar(c));
                }
            }
            UnescapeState::FirstHex => {
                let first_hex = to_hex(c)?;
                decode_state = UnescapeState::SecondHex(first_hex << 4);
            }
            UnescapeState::SecondHex(first_hex) => {
                let second_hex = to_hex(c)?;
                result.push(first_hex | second_hex);
                decode_state = UnescapeState::Normal;
            }
        }
    }
    if let UnescapeState::Normal = decode_state {
        let path = String::from_utf8(result)?;
        Ok(path)
    } else {
        Err(UnixDecodeError::UnescapeState(decode_state))
    }
}

fn decode_unix(unix: &str) -> Result<ClientAddress, ClientAddressDecodeError> {
    if let Some(path) = unix.strip_prefix("path=") {
        let path = decode_unix_path(path)?;
        Ok(ClientAddress::UnixPath(path))
    } else if let Some(abstract_) = unix.strip_prefix("abstract=") {
        let abstract_ = decode_unix_path(abstract_)?;
        Ok(ClientAddress::UnixAbstract(abstract_))
    } else {
        let e = UnixDecodeError::Type;
        Err(ClientAddressDecodeError::from(e))
    }
}

fn decode_socket_address_element(
    element: &str,
    ip_addr: &mut Option<IpAddr>,
    port: &mut Option<u16>,
) -> Result<(), SocketAddressDecodeError> {
    if let Some(host) = element.strip_prefix("host=") {
        let ip_addr_element = host.parse()?;
        ip_addr.replace(ip_addr_element);
        Ok(())
    } else if let Some(port_str) = element.strip_prefix("port=") {
        let port_element = port_str.parse()?;
        port.replace(port_element);
        Ok(())
    } else {
        Err(SocketAddressDecodeError::UnknownElement)
    }
}

fn decode_socket_address(socket_address: &str) -> Result<SocketAddr, SocketAddressDecodeError> {
    let v: Vec<&str> = socket_address.splitn(3, ',').collect();
    let v_len = v.len();
    if v.len() == 2 {
        let mut ip_addr = None;
        let mut port = None;
        decode_socket_address_element(v[0], &mut ip_addr, &mut port)?;
        decode_socket_address_element(v[1], &mut ip_addr, &mut port)?;
        if let Some(ip_addr) = ip_addr {
            if let Some(port) = port {
                Ok(SocketAddr::new(ip_addr, port))
            } else {
                Err(SocketAddressDecodeError::UnknownPort)
            }
        } else {
            Err(SocketAddressDecodeError::UnknownHost)
        }
    } else {
        Err(SocketAddressDecodeError::VecLen(v_len))
    }
}

fn decode_tcp(tcp: &str) -> Result<ClientAddress, ClientAddressDecodeError> {
    let socket_address = decode_socket_address(tcp)?;
    Ok(ClientAddress::Tcp(socket_address))
}

impl ClientAddress {
    /// Decode [client addresses] separated by `;`.
    ///
    /// [client address]: https://dbus.freedesktop.org/doc/dbus-specification.html#addresses
    pub fn decode(addresses: &str) -> Result<Vec<ClientAddress>, ClientAddressDecodeError> {
        let mut result = Vec::new();
        // Split by the ;, because it can have multiple addresses separated by a ;.
        for address in addresses.split(';') {
            let address = ClientAddress::try_from(address)?;
            result.push(address)
        }
        Ok(result)
    }
}

impl TryFrom<&str> for ClientAddress {
    type Error = ClientAddressDecodeError;

    fn try_from(address: &str) -> Result<Self, Self::Error> {
        if !address.is_ascii() {
            return Err(ClientAddressDecodeError::NonAsciiChar);
        }

        if let Some(unix) = address.strip_prefix("unix:") {
            decode_unix(unix)
        } else if let Some(socket_address) = address.strip_prefix("tcp:") {
            decode_tcp(socket_address)
        } else {
            Err(ClientAddressDecodeError::Type)
        }
    }
}

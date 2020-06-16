use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::num::ParseIntError;
use std::string::FromUtf8Error;

/// This represents a DBus [server address].
///
/// [server address]: https://dbus.freedesktop.org/doc/dbus-specification.html#addresses
#[derive(Debug, Clone)]
pub enum ServerAddress {
    UnixRuntime,
    UnixPath(String),
    UnixAbstract(String),
    Tcp(SocketAddr),
}

/// An enum representing all errors, which can occur during parsing a [server address].
///
/// [server address]: https://dbus.freedesktop.org/doc/dbus-specification.html#addresses
#[derive(Debug, Clone)]
pub enum ServerAddressParseError {
    UnixParseError(UnixParseError),
    SocketAddressError(SocketAddressParseError),
    Type,
}

impl Display for ServerAddressParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ServerAddressParseError::UnixParseError(e) => {
                write!(f, "Could not parse unix address: {}", e)
            }
            ServerAddressParseError::SocketAddressError(e) => {
                write!(f, "Could not parse socket address: {}", e)
            }
            ServerAddressParseError::Type => write!(f, "Unknown type"),
        }
    }
}

impl From<UnixParseError> for ServerAddressParseError {
    fn from(e: UnixParseError) -> Self {
        ServerAddressParseError::UnixParseError(e)
    }
}

impl From<SocketAddressParseError> for ServerAddressParseError {
    fn from(e: SocketAddressParseError) -> Self {
        ServerAddressParseError::SocketAddressError(e)
    }
}

/// An enum representing all errors, which can occur during parsing an unix path of a
/// [server address]. For example: `unix:path=/tmp/dbus/path`.
///
/// [server address]: https://dbus.freedesktop.org/doc/dbus-specification.html#addresses
#[derive(Debug, Clone)]
pub enum UnixParseError {
    UnescapeChar(u8),
    UnescapeState(UnescapeState),
    UnescapeFromUtf8Error(FromUtf8Error),
    UnescapeHex(u8),
    Type,
}

impl Display for UnixParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            UnixParseError::UnescapeChar(c) => write!(f, "Unknown char: 0x{:02x}", c),
            UnixParseError::UnescapeState(state) => write!(f, "State is not Normal: {}", state),
            UnixParseError::UnescapeFromUtf8Error(e) => write!(f, "UTF-8: {}", e),
            UnixParseError::UnescapeHex(c) => write!(f, "Unknown hex: {}", c),
            UnixParseError::Type => write!(f, "Unknown type"),
        }
    }
}

impl From<FromUtf8Error> for UnixParseError {
    fn from(e: FromUtf8Error) -> Self {
        UnixParseError::UnescapeFromUtf8Error(e)
    }
}

/// An enum representing all errors, which can occur during parsing a socket address of a
/// [server address]. For example: `tcp:host=127.0.0.1,port=30900`.
///
/// [server address]: https://dbus.freedesktop.org/doc/dbus-specification.html#addresses
#[derive(Debug, Clone)]
pub enum SocketAddressParseError {
    VecLen(usize),
    HostParseError(AddrParseError),
    PortParseError(ParseIntError),
    UnknownHost,
    UnknownPort,
    UnknownElement,
}

impl Display for SocketAddressParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            SocketAddressParseError::VecLen(l) => write!(f, "The socket address has {} element", l),
            SocketAddressParseError::HostParseError(e) => write!(f, "Host parse: {}", e),
            SocketAddressParseError::PortParseError(e) => write!(f, "Port parse: {}", e),
            SocketAddressParseError::UnknownHost => write!(f, "Unknown host"),
            SocketAddressParseError::UnknownPort => write!(f, "Unknown port"),
            SocketAddressParseError::UnknownElement => write!(f, "Unknown element"),
        }
    }
}

impl From<AddrParseError> for SocketAddressParseError {
    fn from(e: AddrParseError) -> Self {
        SocketAddressParseError::HostParseError(e)
    }
}

impl From<ParseIntError> for SocketAddressParseError {
    fn from(e: ParseIntError) -> Self {
        SocketAddressParseError::PortParseError(e)
    }
}

#[inline]
fn is_optionally_escaped(c: u8) -> bool {
    // *?
    // [-0-9A-Za-z_/.\]
    c.is_ascii_alphanumeric() || c == b'-' || c == b'_' || c == b'/' || c == b'.' || c == b'\\'
}

fn to_hex(c: u8) -> Result<u8, UnixParseError> {
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
        c => Err(UnixParseError::UnescapeHex(c)),
    }
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

fn parse_unix_path(path: &str) -> Result<String, UnixParseError> {
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
                    return Err(UnixParseError::UnescapeChar(c));
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
        Err(UnixParseError::UnescapeState(decode_state))
    }
}

fn parse_unix(unix: &str) -> Result<ServerAddress, ServerAddressParseError> {
    if let Some(path) = unix.strip_prefix("path=") {
        let path = parse_unix_path(path)?;
        Ok(ServerAddress::UnixPath(path))
    } else if let Some(abstract_) = unix.strip_prefix("abstract=") {
        let abstract_ = parse_unix_path(abstract_)?;
        Ok(ServerAddress::UnixAbstract(abstract_))
    } else if unix == "runtime=yes" {
        Ok(ServerAddress::UnixRuntime)
    } else {
        let e = UnixParseError::Type;
        Err(ServerAddressParseError::from(e))
    }
}

fn parse_socket_address_element(
    element: &str,
    ip_addr: &mut Option<IpAddr>,
    port: &mut Option<u16>,
) -> Result<(), SocketAddressParseError> {
    if let Some(host) = element.strip_prefix("host=") {
        let ip_addr_element = host.parse()?;
        ip_addr.replace(ip_addr_element);
        Ok(())
    } else if let Some(port_str) = element.strip_prefix("port=") {
        let port_element = port_str.parse()?;
        port.replace(port_element);
        Ok(())
    } else {
        Err(SocketAddressParseError::UnknownElement)
    }
}

fn parse_socket_address(socket_address: &str) -> Result<SocketAddr, SocketAddressParseError> {
    let v: Vec<&str> = socket_address.splitn(3, ',').collect();
    let v_len = v.len();
    if v.len() == 2 {
        let mut ip_addr = None;
        let mut port = None;
        parse_socket_address_element(v[0], &mut ip_addr, &mut port)?;
        parse_socket_address_element(v[1], &mut ip_addr, &mut port)?;
        if let Some(ip_addr) = ip_addr {
            if let Some(port) = port {
                Ok(SocketAddr::new(ip_addr, port))
            } else {
                Err(SocketAddressParseError::UnknownPort)
            }
        } else {
            Err(SocketAddressParseError::UnknownHost)
        }
    } else {
        Err(SocketAddressParseError::VecLen(v_len))
    }
}

fn parse_tcp(tcp: &str) -> Result<ServerAddress, ServerAddressParseError> {
    let socket_address = parse_socket_address(tcp)?;
    Ok(ServerAddress::Tcp(socket_address))
}

impl TryFrom<&str> for ServerAddress {
    type Error = ServerAddressParseError;

    fn try_from(address: &str) -> Result<Self, Self::Error> {
        if let Some(unix) = address.strip_prefix("unix:") {
            parse_unix(unix)
        } else if let Some(socket_address) = address.strip_prefix("tcp:") {
            parse_tcp(socket_address)
        } else {
            Err(ServerAddressParseError::Type)
        }
    }
}

impl ServerAddress {
    /// Parse [server addresses] separated by `;`.
    ///
    /// [server address]: https://dbus.freedesktop.org/doc/dbus-specification.html#addresses
    pub fn parse(addresses: &str) -> Result<Vec<ServerAddress>, ServerAddressParseError> {
        let mut result = Vec::new();
        // Split by the ;, because it can have multiple addresses separated by a ;.
        for address in addresses.split(';') {
            let address = ServerAddress::try_from(address)?;
            result.push(address)
        }
        Ok(result)
    }
}

impl Display for ServerAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ServerAddress::UnixPath(path) => write!(f, "unix:path={}", path),
            ServerAddress::UnixAbstract(path) => write!(f, "unix:abstract={}", path),
            ServerAddress::UnixRuntime => write!(f, "unix:runtime=yes"),
            ServerAddress::Tcp(socket_address) => write!(
                f,
                "tcp:host={},port={}",
                socket_address.ip(),
                socket_address.port()
            ),
        }
    }
}

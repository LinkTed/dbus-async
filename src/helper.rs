use hex::encode;
use regex::Regex;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream as StdUnixStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream as TokioUnixStream;

/// Connect to the Unix Domain Stream socket.
async fn connect(path: &str) -> IoResult<TokioUnixStream> {
    // Connect to the Unix Domain Stream.
    let mut stream = TokioUnixStream::connect(path).await?;
    // Write a zero to the socket.
    let zero: [u8; 1] = [0; 1];
    stream.write_all(&zero[..]).await?;
    // Get the UID of the process
    let uid = unsafe { libc::getuid() };
    // Encode the UID in a hex string.
    let hex = encode(uid.to_string());
    // Authenticate to the DBus daemon.
    let cmd = "AUTH EXTERNAL ".to_owned() + &hex + "\r\n";
    stream.write_all(&cmd.into_bytes()).await?;

    // Read the response of the socket.
    let mut buf_reader = BufReader::new(&mut stream);
    let mut resp = String::new();
    buf_reader.read_line(&mut resp).await?;
    // Check if the authentication is successful.
    if !resp.starts_with("OK ") {
        return Err(IoError::new(IoErrorKind::Other, "Could not authenticate"));
    }

    let cmd = "NEGOTIATE_UNIX_FD\r\n";
    stream.write_all(cmd.as_bytes()).await?;

    resp.clear();
    let mut buf_reader = BufReader::new(&mut stream);
    buf_reader.read_line(&mut resp).await?;
    // Check if the authentication is successful.
    if resp != "AGREE_UNIX_FD\r\n" {
        return Err(IoError::new(IoErrorKind::Other, "Could not authenticate"));
    }

    // Authentication was successful.
    stream.write_all(b"BEGIN\r\n").await?;
    Ok(stream)
}

/// Get the Unix Domain Stream socket by connection to the socket defined in the
/// `DBUS_SESSION_BUS_ADDRESS` environment variable.
pub async fn get_unix_socket(path: &str) -> IoResult<TokioUnixStream> {
    lazy_static! {
        /// The regular expression for a valid `DBUS_SESSION_BUS_ADDRESS` environment variable.
        static ref UNIX_PATH_REGEX: Regex =
        Regex::new("^unix:path=([^\\x00]+[^\\x00/])$").unwrap();
    }
    // Split by the ;, because `DBUS_SESSION_BUS_ADDRESS` can have multiple paths separated
    // by a ;.
    for p in path.split(';') {
        // Check if it is a valid path.
        if let Some(c) = UNIX_PATH_REGEX.captures(&p) {
            // Try to get the path
            // The string after the first equal sign.
            if let Some(p) = c.get(1) {
                // Try to connect to the Domain socket and authenticate.
                match connect(p.as_str()).await {
                    Ok(stream) => return Ok(stream),
                    Err(e) => {
                        // It failed to try the next path
                        error!("Cannot connect to {}: {}", p.as_str(), e);
                    }
                }
            }
        }
    }
    // It could not connect to any socket
    Err(IoError::new(
        IoErrorKind::Other,
        format!("Could not open any socket in: {}", path),
    ))
}

pub fn split(unix_stream: TokioUnixStream) -> IoResult<(TokioUnixStream, TokioUnixStream)> {
    unsafe {
        let fd = libc::dup(unix_stream.as_raw_fd());
        let sink = StdUnixStream::from_raw_fd(fd);
        let stream = unix_stream;
        let sink = TokioUnixStream::from_std(sink)?;
        Ok((stream, sink))
    }
}

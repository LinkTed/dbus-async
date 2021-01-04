use dbus_async::ClientAddress;

fn decode_encode_decode(path_1: &str) {
    let server_addresses_1 = ClientAddress::decode(path_1).unwrap();
    let path_2 = ClientAddress::encode(&server_addresses_1);
    let server_addresses_2 = ClientAddress::decode(path_2.as_str()).unwrap();
    assert_eq!(server_addresses_1, server_addresses_2);
}

#[test]
fn unix_path() {
    let path = "unix:path=/tmp/dbus-test";
    decode_encode_decode(path);
}

#[test]
fn unix_abstract_path() {
    let path = "unix:abstract=/tmp/dbus-test";
    decode_encode_decode(path);
}

#[test]
fn tcp() {
    let path = "tcp:host=127.0.0.1,port=30958";
    decode_encode_decode(path);
}

#[test]
fn all() {
    let path =
        "unix:path=/tmp/dbus-test;unix:abstract=/tmp/dbus-test;tcp:host=127.0.0.1,port=30958";
    decode_encode_decode(path);
}

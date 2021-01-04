#[macro_use]
extern crate honggfuzz;
use dbus_async::ClientAddress;
use std::str::from_utf8;

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            if let Ok(path_1) = from_utf8(data) {
                let client_addresses_1 = ClientAddress::decode(path_1);
                if let Ok(client_addresses_1) = client_addresses_1 {
                    let path_2 = ClientAddress::encode(&client_addresses_1);
                    let client_addresses_2 = ClientAddress::decode(&path_2).unwrap();
                    assert_eq!(client_addresses_1, client_addresses_2);
                }
            }
        });
    }
}

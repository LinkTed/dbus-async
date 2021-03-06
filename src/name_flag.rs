bitflags! {
    /// An enum representing a [flag] for the [`RequestName`] method. see [`request_name`]
    ///
    /// [flag]: https://dbus.freedesktop.org/doc/dbus-specification.html#bus-messages-request-name
    /// [`RequestName`]: https://dbus.freedesktop.org/doc/dbus-specification.html#bus-messages-request-name
    /// [`request_name`]: crate::DBus::request_name
    pub struct DBusNameFlag: u32 {
        const ALLOW_REPLACEMENT = 0x01;
        const REPLACE_EXISTING = 0x02;
        const DO_NOT_QUEUE = 0x04;
    }
}

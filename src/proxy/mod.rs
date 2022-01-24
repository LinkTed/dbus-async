/*! <span class="stab portability" title="This is supported on crate feature `proxy` only"><code>proxy</code></span> Povides a D-Bus [`Proxy`] - the client side of RPC calls

```
use dbus_async::{DBus, proxy::{Proxy, Error}};
use dbus_message_parser::value::Value;
use std::convert::TryInto;
async fn turn_on_bt() -> Result<(),Error> {
    let (dbus, _dbus_handle) = DBus::system(false, false).await?;
    let p = Proxy::new(
        "org.bluez".try_into()?,
        "/org/bluez/hci0".try_into()?,
        &dbus
    );
    p.set_property(
        "org.bluez.Adapter1".try_into()?,
        "Powered",
        Value::Boolean(true)
    ).await?;
    p.method_call(
        "org.bluez.Adapter1".try_into()?,
        "StartDiscovery".try_into()?,
        []
    ).await?;
    Ok(())
}
```

*/

mod error;

#[cfg(feature = "introspect")]
pub mod introspect;

use crate::DBus;
use dbus_message_parser::message::{Message, MessageType};
use dbus_message_parser::value::{Bus, Interface, Member, ObjectPath, Value};
use std::collections::HashMap;
use std::convert::TryInto;

pub use error::Error;
pub type ProxyResult<T> = Result<T, Error>;

/// A struct that wraps a connection, destination and path.
///
/// A D-Bus "Proxy" is a client-side object that corresponds to a remote object on the server side.
/// Calling methods on the proxy object calls methods on the remote object.
/// Read more in the [D-Bus tutorial]
///
/// [D-Bus tutorial]: https://dbus.freedesktop.org/doc/dbus-tutorial.html#proxies
pub struct Proxy<'a> {
    destination: Bus,
    object_path: ObjectPath,
    con: &'a DBus,
}
impl<'a> Proxy<'a> {
    /// Creates a new proxy struct.
    pub fn new(destination: Bus, object_path: ObjectPath, con: &'a DBus) -> Self {
        Proxy {
            destination,
            object_path,
            con,
        }
    }

    pub fn get_object_path(&self) -> &ObjectPath {
        &self.object_path
    }

    pub fn get_destination(&self) -> &Bus {
        &self.destination
    }

    ///Call the RPC method `interface`.`method`(`args`)
    pub async fn method_call<A>(
        &self,
        interface: Interface,
        method: Member,
        args: A,
    ) -> ProxyResult<Message>
    where
        A: IntoIterator<Item = Value>,
    {
        let mut msg = Message::method_call(
            self.destination.clone(),
            self.object_path.clone(),
            interface,
            method,
        );
        for val in args {
            msg.add_value(val);
        }
        self.call(msg).await
    }

    async fn call(&self, msg: Message) -> ProxyResult<Message> {
        if log::log_enabled!(log::Level::Trace) {
            if let (Some(path), Some(ifn), Some(mem), Ok(sig)) = (
                msg.get_path(),
                msg.get_interface(),
                msg.get_member(),
                msg.get_signature(),
            ) {
                log::trace!("{}\t{}.{}({:?})", path, ifn, mem, sig);
            } else {
                log::trace!("{:?}", msg);
            }
        }
        let resp = self.con.call(msg).await?;
        log::trace!("ret: {:?} {:?}", resp.get_type(), resp.get_signature());
        if resp.get_type() == MessageType::Error {
            return Err(resp.into());
        }
        Ok(resp)
    }

    ///Get all objects and properties see [objectmanager]
    ///
    /// [objectmanager]: https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-objectmanager
    pub async fn get_managed_objects(
        &self,
    ) -> ProxyResult<HashMap<ObjectPath, HashMap<String, Value>>> {
        let msg = Message::method_call(
            self.destination.clone(),
            self.object_path.clone(),
            "org.freedesktop.DBus.ObjectManager".try_into().unwrap(),
            "GetManagedObjects".try_into().unwrap(),
        );
        let resp = self.call(msg).await?;
        //ARRAY of DICT_ENTRY<STRING,ARRAY of DICT_ENTRY<STRING,VARIANT>> interfaces_and_properties
        let mut ret = HashMap::new();
        if let Some(Value::Array(a)) = resp.get_body().get(0) {
            for r in a.as_ref() {
                if let Value::DictEntry(b) = r {
                    if let (Value::ObjectPath(k), Value::Array(vals)) = b.as_ref() {
                        let mut sub = HashMap::new();
                        for r in vals.as_ref() {
                            if let Value::DictEntry(map) = r {
                                if let (Value::String(k), v) = map.as_ref() {
                                    sub.insert(k.clone(), v.clone());
                                }
                            }
                        }
                        ret.insert(k.clone(), sub);
                    }
                }
            }
            return Ok(ret);
        }
        Err(Error::UnexpectedFormat(
            resp.get_signature().map_or(Vec::new(), |t| t.to_vec()),
        ))
    }

    ///Set a property see [properties]
    ///
    /// [properties]: https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-properties
    pub async fn set_property(
        &self,
        interface: Interface,
        property: &str,
        value: Value,
    ) -> ProxyResult<()> {
        let msg = Message::property_set(
            self.destination.clone(),
            self.object_path.clone(),
            interface,
            property,
            value,
        );
        self.call(msg).await?;
        Ok(())
    }

    ///Get a property see [properties]
    ///
    /// [properties]: https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-properties
    pub async fn get_property(
        &self,
        interface: Interface,
        property: &str,
    ) -> ProxyResult<Box<Value>> {
        let msg = Message::property_get(
            self.destination.clone(),
            self.object_path.clone(),
            interface,
            property,
        );
        let (_h, mut vals) = self.call(msg).await?.split()?;
        if vals.len() == 1 {
            if let Some(Value::Variant(v)) = vals.pop() {
                return Ok(v);
            }
        }
        Err(Error::UnexpectedFormat(
            _h.get_signature().map_or(Vec::new(), |t| t.to_vec()),
        ))
    }

    ///Get all properties see [properties]
    ///
    /// [properties]: https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-properties
    pub async fn get_properties(
        &self,
        interface: Interface,
    ) -> ProxyResult<HashMap<String, Box<Value>>> {
        let msg = Message::properties_get_all(
            self.destination.clone(),
            self.object_path.clone(),
            interface,
        );
        let (_h, mut vals) = self.call(msg).await?.split()?;

        let mut ret = HashMap::new();

        if let Some(Value::Array(a)) = vals.pop() {
            for r in a.as_ref() {
                if let Value::DictEntry(b) = r {
                    if let (Value::String(k), Value::Variant(v)) = b.as_ref() {
                        ret.insert(k.clone(), v.clone());
                    }
                }
            }
            return Ok(ret);
        }
        Err(Error::UnexpectedFormat(
            _h.get_signature().map_or(Vec::new(), |t| t.to_vec()),
        ))
    }
}

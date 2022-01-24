/*! <span class="stab portability" title="This is supported on crate feature `introspect` only"><code>introspect</code></span> Types returned by [`Proxy::introspect`].
*/
use super::{Error, Proxy, ProxyResult};
use dbus_message_parser::{message::Message, value::Value};
use serde_derive::Deserialize;
use std::convert::TryInto;

impl<'a> Proxy<'a> {
    ///<span class="stab portability" title="This is supported on crate feature `introspect` only"><code>introspect</code></span> Get a description of the object.
    /// This includes its interfaces (with signals and methods), objects below it in the object path tree, and its properties.
    /// See [introspectable]
    ///
    /// [introspectable]: https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-introspectable
    pub async fn introspect(&self) -> ProxyResult<Node> {
        let msg = Message::method_call(
            self.destination.clone(),
            self.object_path.clone(),
            "org.freedesktop.DBus.Introspectable".try_into().unwrap(),
            "Introspect".try_into().unwrap(),
        );
        let resp = self.call(msg).await?;
        if let Some(Value::String(xml)) = resp.get_body().get(0) {
            if let Ok(device_node) = serde_xml_rs::from_str(xml) {
                return Ok(device_node);
            }
        }
        Err(Error::UnexpectedFormat(
            resp.get_signature().map_or(Vec::new(), |t| t.to_vec()),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Node {
    pub name: Option<String>,
    #[serde(rename = "interface", default)]
    pub interfaces: Vec<Interface>,
    #[serde(rename = "node", default)]
    pub nodes: Vec<Node>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Interface {
    pub name: String,
    #[serde(rename = "method", default)]
    pub methods: Vec<Method>,
    #[serde(rename = "signal", default)]
    pub signals: Vec<Signal>,
    #[serde(rename = "property", default)]
    pub properties: Vec<Property>,
    #[serde(rename = "annotation", default)]
    pub annotations: Vec<Annotation>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Method {
    pub name: String,
    #[serde(rename = "arg", default)]
    pub args: Vec<MethodArg>,
    #[serde(rename = "annotation", default)]
    pub annotations: Vec<Annotation>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Signal {
    pub name: String,
    #[serde(rename = "arg", default)]
    pub args: Vec<SignalArg>,
    #[serde(rename = "annotation", default)]
    pub annotations: Vec<Annotation>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Property {
    pub name: String,
    #[serde(rename = "type")]
    pub dbustype: String,
    pub access: Access,
    #[serde(rename = "annotation", default)]
    pub annotations: Vec<Annotation>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct MethodArg {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub dbustype: String,
    #[serde(default = "default_method_arg_direction")]
    pub direction: Direction,
    #[serde(rename = "annotation", default)]
    pub annotations: Vec<Annotation>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SignalArg {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub dbustype: String,
    #[serde(default = "default_signal_arg_direction")]
    pub direction: Direction,
    #[serde(rename = "annotation", default)]
    pub annotations: Vec<Annotation>,
}

fn default_method_arg_direction() -> Direction {
    Direction::In
}

fn default_signal_arg_direction() -> Direction {
    Direction::Out
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Annotation {
    pub name: String,
    pub value: String,
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum Direction {
    #[serde(rename = "in")]
    In,
    #[serde(rename = "out")]
    Out,
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum Access {
    #[serde(rename = "readwrite")]
    ReadWrite,
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "write")]
    Write,
}

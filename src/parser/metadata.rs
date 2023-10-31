use crate::gettext;
use html5ever::tendril::TendrilSink;
use html5ever::{parse_document, ParseOpts};
use json::JsonValue;
use markup5ever_rcdom::{Node, NodeData, RcDom};
use std::default::Default;

pub struct MetaDataParser {
    pub key: String,
    pub value: Option<JsonValue>,
}

impl MetaDataParser {
    /// Create an new instance
    /// * `key` - meta's name
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            value: None,
        }
    }

    fn iter(&mut self, node: &Node) -> bool {
        match &node.data {
            NodeData::Element { name, attrs, .. } => {
                if name.local.as_ref() == "meta" {
                    let attrs = attrs.borrow();
                    let name = attrs.iter().find(|a| a.name.local.as_ref() == "name");
                    if name.is_none() {
                        return false;
                    }
                    let name = name.unwrap();
                    if name.value.as_ref() != self.key.as_str() {
                        return false;
                    }
                    let id = attrs.iter().find(|a| a.name.local.as_ref() == "id");
                    if id.is_none() {
                        return false;
                    }
                    let id = id.unwrap();
                    let mkey = format!("meta-{}", self.key.as_str());
                    let id = id.value.as_ref();
                    if id != mkey.as_str() && id != self.key.as_str() {
                        return false;
                    }
                    let c = attrs.iter().find(|a| a.name.local.as_ref() == "content");
                    if c.is_none() {
                        return false;
                    }
                    let c = c.unwrap();
                    let r = json::parse(c.value.as_ref());
                    if r.is_err() {
                        log::error!("{} {}", gettext("Failed to parse JSON:"), r.unwrap_err());
                        return false;
                    }
                    self.value = Some(r.unwrap());
                    true
                } else {
                    for c in node.children.borrow().iter() {
                        if self.iter(c) {
                            return true;
                        }
                    }
                    false
                }
            }
            _ => false,
        }
    }

    pub fn parse(&mut self, context: &str) -> bool {
        let opts = ParseOpts::default();
        let r = parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut context.as_bytes());
        let dom = match r {
            Ok(d) => d,
            Err(e) => {
                log::error!("{} {}", gettext("Failed to parse HTML:"), e.to_string());
                return false;
            }
        };
        for n in dom.document.children.borrow().iter() {
            if self.iter(n) {
                return true;
            }
        }
        false
    }
}

impl Default for MetaDataParser {
    fn default() -> Self {
        Self::new("global-data")
    }
}

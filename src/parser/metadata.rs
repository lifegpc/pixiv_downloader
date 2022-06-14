use crate::gettext;
use html_parser::Dom;
use html_parser::Node;
use json::JsonValue;
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
        match node {
            Node::Element(e) => {
                if e.name == "meta" {
                    let name = e.attributes.get("name");
                    if name.is_none() {
                        return false;
                    }
                    let name = name.unwrap();
                    if name.is_none() {
                        return false;
                    }
                    if name.as_ref().unwrap() != self.key.as_str() {
                        return false;
                    }
                    if e.id.is_none() {
                        return false;
                    }
                    let mkey = format!("meta-{}", self.key.as_str());
                    if e.id.as_ref().unwrap() != mkey.as_str() {
                        return false;
                    }
                    let c = e.attributes.get("content");
                    if c.is_none() {
                        return false;
                    }
                    let c = c.unwrap();
                    if c.is_none() {
                        return false;
                    }
                    let r = json::parse(c.as_ref().unwrap());
                    if r.is_err() {
                        println!("{} {}", gettext("Failed to parse JSON:"), r.unwrap_err());
                        return false;
                    }
                    self.value = Some(r.unwrap());
                    true
                } else {
                    for c in e.children.iter() {
                        if self.iter(c) {
                            return true;
                        }
                    }
                    false
                }
            }
            Node::Comment(_) => false,
            Node::Text(_) => false,
        }
    }

    pub fn parse(&mut self, context: &str) -> bool {
        let r = Dom::parse(context);
        if r.is_err() {
            println!("{} {}", gettext("Failed to parse HTML:"), r.unwrap_err());
            return false;
        }
        let dom = r.unwrap();
        if dom.errors.len() > 0 {
            println!("{}", gettext("Some errors occured during parsing:"));
            for i in dom.errors.iter() {
                println!("{}", i);
            }
        }
        for n in dom.children.iter() {
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

use crate::gettext;
use html_parser::Dom;
use html_parser::Node;
use json::JsonValue;

pub struct MainPageParser {
    pub value: Option<JsonValue>,
}

impl MainPageParser {
    pub fn new() -> Self {
        Self {
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
                    if name.as_ref().unwrap() != "global-data" {
                        return false;
                    }
                    if e.id.is_none() {
                        return false;
                    }
                    if e.id.as_ref().unwrap() != "meta-global-data" {
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
            Node::Comment(_) => { false }
            Node::Text(_) => { false }
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

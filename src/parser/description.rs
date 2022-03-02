use crate::gettext;
use crate::pixiv_link::remove_track;
use html_parser::Dom;
use html_parser::Node;
use std::collections::HashMap;
use std::default::Default;

/// Reprent a node
#[derive(Debug)]
struct DescriptionNode {
    /// Tag name
    pub tag: String,
    /// Output data in this node
    pub data: String,
    /// Attributes
    pub attrs: HashMap<String, String>,
}

impl Default for DescriptionNode {
    fn default() -> Self {
        Self {
            tag: String::from(""),
            data: String::from(""),
            attrs: HashMap::new(),
        }
    }
}

impl DescriptionNode {
    pub fn add_attr(&mut self, k: &str, v: &str) -> Option<String> {
        self.attrs.insert(String::from(k), String::from(v))
    }

    pub fn is_link(&self) -> bool {
        if self.tag != "a" {
            return false;
        }
        if !self.attrs.contains_key("href") {
            return false;
        }
        let href = self.attrs.get("href").unwrap();
        if href.as_str() == self.data.as_str() {
            return false;
        }
        true
    }

    pub fn to_link(&self) -> String {
        format!(
            "[{}]({})",
            self.data.as_str(),
            self.attrs.get("href").unwrap()
        )
    }
}

/// A simple HTML parser to parse description HTML
pub struct DescriptionParser {
    /// Current nodes stack
    nodes: Vec<DescriptionNode>,
    /// Output
    pub data: String,
}

impl DescriptionParser {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            data: String::from(""),
        }
    }

    pub fn iter(&mut self, node: &Node) {
        match node {
            Node::Comment(_) => {}
            Node::Text(s) => {
                if self.nodes.len() == 0 {
                    self.data += s;
                } else {
                    self.nodes.last_mut().unwrap().data += s;
                }
            }
            Node::Element(e) => {
                let tag = e.name.as_str();
                if tag == "script" || tag == "style" {
                    return;
                } else if tag == "br" {
                    if self.nodes.len() == 0 {
                        self.data += "\n";
                    } else {
                        self.nodes.last_mut().unwrap().data += "\n";
                    }
                    return;
                }
                let mut node = DescriptionNode::default();
                node.tag = tag.to_string();
                if tag == "a" {
                    let href = e.attributes.get("href");
                    if href.is_some() {
                        let href = href.unwrap();
                        if href.is_some() {
                            let link = remove_track(href.as_ref().unwrap());
                            node.add_attr("href", link.as_str());
                        }
                    }
                }
                self.nodes.push(node);
                for n in e.children.iter() {
                    self.iter(n);
                }
                let node = self.nodes.pop().unwrap();
                let s = if node.is_link() {
                    node.to_link()
                } else {
                    node.data
                };
                if self.nodes.len() == 0 {
                    self.data += s.as_str();
                } else {
                    self.nodes.last_mut().unwrap().data += s.as_str();
                }
            }
        }
    }

    pub fn parse(&mut self, desc: &str) -> Result<(), ()> {
        let r = Dom::parse(desc);
        if r.is_err() {
            println!("{} {}", gettext("Failed to parse HTML:"), r.unwrap_err());
            return Err(());
        }
        let dom = r.unwrap();
        if dom.errors.len() > 0 {
            println!("{}", gettext("Some errors occured during parsing:"));
            for i in dom.errors.iter() {
                println!("{}", i);
            }
        }
        for node in dom.children.iter() {
            self.iter(node)
        }
        if self.nodes.len() != 0 {
            println!(
                "{} {:?}",
                gettext("There are some nodes still in stack:"),
                self.nodes
            );
            return Err(());
        }
        Ok(())
    }
}

pub fn parse_description(desc: &str) -> Option<String> {
    let mut p = DescriptionParser::new();
    match p.parse(desc) {
        Ok(_) => Some(p.data),
        Err(_) => None,
    }
}

#[test]
fn test_parse_description() {
    assert_eq!(
        Some(String::from("a [example](https://a.com)")),
        parse_description("a <a href=\"https://a.com\">example</a>")
    );
    assert_eq!(
        Some(String::from("a https://a.com")),
        parse_description("a <a href=\"https://a.com\">https://a.com</a>")
    );
    assert_eq!(
        Some(String::from("a [a\n[bc](a.com)d](b.com)\ndata")),
        parse_description("a <a href=\"b.com\">a<br/><a href=\"a.com\">bc</a>d</a><br>data")
    );
    assert_eq!(
        Some(String::from("https://a.com")),
        parse_description("<a href=\"/jump.php?https%3A%2F%2Fa.com\">https://a.com</a>")
    )
}

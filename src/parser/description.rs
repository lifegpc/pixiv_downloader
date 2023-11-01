use crate::error::PixivDownloaderError;
use crate::gettext;
use crate::pixiv_link::remove_track;
use html5ever::tendril::TendrilSink;
use html5ever::{parse_document, ParseOpts};
use markup5ever_rcdom::{Node, NodeData, RcDom};
use percent_encoding::{percent_encode, AsciiSet, NON_ALPHANUMERIC};
use std::collections::HashMap;
use std::default::Default;

const URLENCODE: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b':')
    .remove(b'/')
    .remove(b'?')
    .remove(b'#')
    .remove(b'[')
    .remove(b']')
    .remove(b'@')
    .remove(b'!')
    .remove(b'$')
    .remove(b'&')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')')
    .remove(b'*')
    .remove(b'+')
    .remove(b',')
    .remove(b';')
    .remove(b'=')
    .remove(b'%')
    .remove(b' ')
    .remove(b'.');

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

    pub fn is_em(&self) -> bool {
        self.tag == "em" || self.tag == "i"
    }

    pub fn is_headline(&self) -> bool {
        match self.tag.as_str() {
            "h1" => true,
            "h2" => true,
            "h3" => true,
            "h4" => true,
            "h5" => true,
            "h6" => true,
            _ => false,
        }
    }

    pub fn is_link(&self, md_mode: bool) -> bool {
        if self.tag != "a" {
            return false;
        }
        if !self.attrs.contains_key("href") {
            return false;
        }
        let href = self.attrs.get("href").unwrap();
        if !md_mode && href.as_str() == self.data.as_str() {
            return false;
        }
        true
    }

    pub fn is_paragraph(&self) -> bool {
        self.tag == "p" || self.tag == "paragraph"
    }

    pub fn is_strong(&self) -> bool {
        self.tag == "strong" || self.tag == "b"
    }

    pub fn to_em(&self) -> String {
        format!("*{}*", self.data.as_str())
    }

    pub fn to_headline(&self) -> String {
        let mut s = String::from("#");
        let level = self.tag.chars().last().unwrap().to_digit(10).unwrap();
        for _ in 1..level {
            s += "#";
        }
        s += " ";
        s += self.data.as_str();
        s += "\n";
        s
    }

    pub fn to_link(&self, ascii: bool) -> String {
        let href = self.attrs.get("href").unwrap();
        let href = if ascii {
            percent_encode(href.as_bytes(), URLENCODE).to_string()
        } else {
            href.clone()
        };
        format!("[{}]({})", self.data.as_str(), href)
    }

    pub fn to_paragraph(&self) -> String {
        let mut s = self.data.clone();
        s += "\n\n";
        s
    }

    pub fn to_strong(&self) -> String {
        format!("**{}**", self.data.as_str())
    }
}

pub struct DescriptionParserBuilder {
    /// Markdown mode
    md_mode: bool,
    /// Ensure link is ASCII
    _ensure_link_ascii: bool,
}

#[allow(dead_code)]
impl DescriptionParserBuilder {
    pub fn new(md_mode: bool) -> Self {
        Self {
            md_mode,
            _ensure_link_ascii: false,
        }
    }

    /// Ensure link is ASCII
    pub fn ensure_link_ascii(mut self) -> Self {
        self._ensure_link_ascii = true;
        self
    }

    pub fn build(self) -> DescriptionParser {
        DescriptionParser::from(self)
    }
}

/// A simple HTML parser to parse description HTML
pub struct DescriptionParser {
    /// Current nodes stack
    nodes: Vec<DescriptionNode>,
    /// Output
    pub data: String,
    /// Options
    opts: DescriptionParserBuilder,
}

impl DescriptionParser {
    pub fn new(md_mode: bool) -> Self {
        Self {
            nodes: Vec::new(),
            data: String::from(""),
            opts: DescriptionParserBuilder::new(md_mode),
        }
    }

    pub fn iter(&mut self, node: &Node) {
        match &node.data {
            NodeData::Text { contents } => {
                let s = contents.borrow().to_string();
                if self.nodes.len() == 0 {
                    self.data += &s;
                } else {
                    self.nodes.last_mut().unwrap().data += &s;
                }
            }
            NodeData::Element { name, attrs, .. } => {
                let tag = name.local.to_string();
                if tag == "script" || tag == "style" {
                    return;
                } else if tag == "br" {
                    let br = if self.opts.md_mode { "  \n" } else { "\n" };
                    if self.nodes.len() == 0 {
                        self.data += br;
                    } else {
                        self.nodes.last_mut().unwrap().data += br;
                    }
                    return;
                }
                let mut nod = DescriptionNode::default();
                nod.tag = tag.to_string();
                let attrs = attrs.borrow();
                if tag == "a" {
                    let href = attrs.iter().find(|k| k.name.local.to_string() == "href");
                    if href.is_some() {
                        let href = href.unwrap().value.to_string();
                        let link = remove_track(href);
                        nod.add_attr("href", link.as_str());
                    }
                }
                self.nodes.push(nod);
                for n in node.children.borrow().iter() {
                    self.iter(n);
                }
                let node = self.nodes.pop().unwrap();
                let mut is_paragraph = false;
                let s = if node.is_link(self.opts.md_mode) {
                    node.to_link(self.opts._ensure_link_ascii)
                } else if self.opts.md_mode && node.is_headline() {
                    node.to_headline()
                } else if self.opts.md_mode && node.is_paragraph() {
                    is_paragraph = true;
                    node.to_paragraph()
                } else if self.opts.md_mode && node.is_strong() {
                    node.to_strong()
                } else if self.opts.md_mode && node.is_em() {
                    node.to_em()
                } else {
                    node.data
                };
                if self.nodes.len() == 0 {
                    while self.opts.md_mode && is_paragraph && !self.data.ends_with("\n\n") {
                        self.data += "\n";
                    }
                    self.data += s.as_str();
                } else {
                    let n = self.nodes.last_mut().unwrap();
                    while self.opts.md_mode && is_paragraph && !n.data.ends_with("\n\n") {
                        n.data += "\n";
                    }
                    n.data += s.as_str();
                }
            }
            _ => {}
        }
    }

    pub fn parse<S: AsRef<str> + ?Sized>(&mut self, desc: &S) -> Result<(), PixivDownloaderError> {
        let opts = ParseOpts::default();
        let r = parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut desc.as_ref().as_bytes());
        let dom = match r {
            Ok(d) => d,
            Err(e) => {
                return Err(
                    format!("{} {}", gettext("Failed to parse HTML:"), e.to_string()).into(),
                )
            }
        };
        for node in dom.document.children.borrow().iter() {
            self.iter(node)
        }
        if self.nodes.len() != 0 {
            return Err(format!(
                "{} {:?}",
                gettext("There are some nodes still in stack:"),
                self.nodes
            )
            .into());
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn builder(md_mode: bool) -> DescriptionParserBuilder {
        DescriptionParserBuilder::new(md_mode)
    }
}

impl From<DescriptionParserBuilder> for DescriptionParser {
    fn from(opts: DescriptionParserBuilder) -> Self {
        Self {
            nodes: Vec::new(),
            data: String::from(""),
            opts,
        }
    }
}

pub fn parse_description<S: AsRef<str> + ?Sized>(desc: &S) -> Option<String> {
    let mut p = DescriptionParser::new(false);
    match p.parse(desc) {
        Ok(_) => Some(p.data),
        Err(e) => {
            log::error!("{}", e);
            None
        }
    }
}

pub fn convert_description_to_md<S: AsRef<str> + ?Sized>(
    desc: &S,
) -> Result<String, PixivDownloaderError> {
    let mut p = DescriptionParser::new(true);
    p.parse(desc)?;
    Ok(p.data)
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
        Some(String::from("a [a\n](b.com)[bc](a.com)d\ndata")),
        parse_description("a <a href=\"b.com\">a<br/><a href=\"a.com\">bc</a>d</a><br>data")
    );
    assert_eq!(
        Some(String::from("https://a.com")),
        parse_description("<a href=\"/jump.php?https%3A%2F%2Fa.com\">https://a.com</a>")
    )
}

#[test]
fn test_convert_description_to_md() {
    assert_eq!(
        String::from("test  \n[https://a.com](https://a.com)"),
        convert_description_to_md(
            "test<br/><a href=\"/jump.php?https%3A%2F%2Fa.com\">https://a.com</a>"
        )
        .unwrap()
    );
    assert_eq!(
        String::from("# He\n## Be\ntest"),
        convert_description_to_md("<h1>He</h1><h2>Be</h2>test").unwrap()
    );
    assert_eq!(
        String::from("D\n\nHe\n\nBe\n\ntest"),
        convert_description_to_md("D<p>He</p><p>Be</p>test").unwrap()
    );
    assert_eq!(
        String::from("# Head\nD\n\nHe\n\nBe\n\nt***e**s*t\n\n[Link](https://a.com)\n\n"),
        convert_description_to_md("<h1>Head</h1>D<p>He</p><p>Be</p>t<em><strong>e</strong>s</em>t<p><a href=\"/jump.php?https%3A%2F%2Fa.com\">Link</a></p>").unwrap()
    );
}

#[test]
fn test_ensure_link_ascii() {
    let mut p = DescriptionParser::builder(true).ensure_link_ascii().build();
    p.parse("<a href=\"https://test:pass@www.test.com/ad/测试?p=1&t=*\">测试<a>")
        .unwrap();
    assert_eq!(
        String::from("[测试](https://test:pass@www.test.com/ad/%E6%B5%8B%E8%AF%95?p=1&t=*)"),
        p.data
    );
}

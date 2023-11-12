use crate::error::PixivDownloaderError;
use html5ever::tendril::TendrilSink;
use html5ever::{parse_document, ParseOpts, QualName};
use markup5ever_rcdom::{Node, NodeData, RcDom};

#[derive(Clone, Debug, PartialEq)]
/// Message Entity Type
enum MessageEntityType {
    Url,
    /// Bold text
    Bold,
    /// Italic text
    Italic,
    /// Underlined text
    Underline,
    /// Strikethrough text
    Strikethrough,
    /// Spoiler text
    Spoiler,
    /// Monowidth string
    Code,
    /// Monowidth block
    Pre {
        /// Programming language of the entity text
        language: Option<String>,
    },
    /// clickable text URLs
    TextLink {
        url: String,
    },
    /// Inline custom emoji stickers
    CustomEmoji {
        /// Unique identifier for the custom emoji
        custom_emoji_id: String,
    },
}

impl MessageEntityType {
    pub fn is_equal(&self, s: &str) -> bool {
        match s {
            "b" | "strong" => matches!(self, MessageEntityType::Bold),
            "i" | "em" => matches!(self, MessageEntityType::Italic),
            "u" | "ins" => matches!(self, MessageEntityType::Underline),
            "s" | "strike" | "del" => matches!(self, MessageEntityType::Strikethrough),
            "span" => matches!(self, MessageEntityType::Spoiler),
            "tg-spoiler" => matches!(self, MessageEntityType::Spoiler),
            "a" => matches!(self, MessageEntityType::TextLink { .. }),
            "tg-emoji" => matches!(self, MessageEntityType::CustomEmoji { .. }),
            "code" => matches!(self, MessageEntityType::Code),
            "pre" => matches!(self, MessageEntityType::Pre { .. }),
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Message Entity
struct MessageEntity {
    /// Type of the entity
    pub typ: MessageEntityType,
    /// Offset in UTF-16 code units to the start of the entity
    pub offset: usize,
    /// Length of the entity in UTF-16 code units
    pub length: usize,
}

pub struct TextSpliterBuilder {
    /// Maximum length of the message text
    pub _max_length: usize,
    /// Whether to scan links in the message text
    pub scan_link: bool,
}

impl TextSpliterBuilder {
    /// Disable scanning links in the message text
    pub fn disable_scan_link(mut self) -> Self {
        self.scan_link = false;
        self
    }

    /// Set maximum length of the message text
    pub fn max_length(mut self, max_length: usize) -> Self {
        self._max_length = max_length;
        self
    }

    pub fn build(self) -> TextSpliter {
        TextSpliter::from(self)
    }
}

impl Default for TextSpliterBuilder {
    fn default() -> Self {
        Self {
            _max_length: 4096,
            scan_link: true,
        }
    }
}

enum Stack {
    Entity(MessageEntity),
    Other(QualName),
}

impl Stack {
    pub fn is_equal(&self, s: &str) -> bool {
        match self {
            Stack::Entity(entity) => entity.typ.is_equal(s),
            Stack::Other(name) => name.local.as_ref() == s,
        }
    }
}

pub struct TextSpliter {
    entities: Vec<MessageEntity>,
    text: String,
    stacks: Vec<Stack>,
    opts: TextSpliterBuilder,
}

impl TextSpliter {
    pub fn builder() -> TextSpliterBuilder {
        TextSpliterBuilder::default()
    }

    fn is_conflict_with_link_entities(&self, offset: usize, length: usize) -> bool {
        for entity in &self.entities {
            if !matches!(entity.typ, MessageEntityType::TextLink { .. }) {
                continue;
            }
            let esize = entity.offset + entity.length;
            if offset >= entity.offset && offset <= esize {
                return true;
            }
            let eesize = offset + length;
            if eesize >= entity.offset && eesize <= esize {
                return true;
            }
            if offset <= entity.offset && eesize >= esize {
                return true;
            }
        }
        false
    }

    fn iter(&mut self, node: &Node) {
        match &node.data {
            NodeData::Text { contents } => {
                self.text.push_str(&contents.borrow());
            }
            NodeData::Element { name, attrs, .. } => {
                let local_name = name.local.as_ref();
                match local_name {
                    "br" => {
                        self.text.push('\n');
                        return;
                    }
                    "b" | "strong" => {
                        self.stacks.push(Stack::Entity(MessageEntity {
                            typ: MessageEntityType::Bold,
                            offset: self.text_len(),
                            length: 0,
                        }));
                    }
                    "i" | "em" => {
                        self.stacks.push(Stack::Entity(MessageEntity {
                            typ: MessageEntityType::Italic,
                            offset: self.text_len(),
                            length: 0,
                        }));
                    }
                    "u" | "ins" => {
                        self.stacks.push(Stack::Entity(MessageEntity {
                            typ: MessageEntityType::Underline,
                            offset: self.text_len(),
                            length: 0,
                        }));
                    }
                    "s" | "strike" | "del" => {
                        self.stacks.push(Stack::Entity(MessageEntity {
                            typ: MessageEntityType::Strikethrough,
                            offset: self.text_len(),
                            length: 0,
                        }));
                    }
                    "span" => {
                        if let Some(class) = attrs
                            .borrow()
                            .iter()
                            .find(|attr| attr.name.local.as_ref() == "class")
                        {
                            if class.value.as_ref() == "tg-spoiler" {
                                self.stacks.push(Stack::Entity(MessageEntity {
                                    typ: MessageEntityType::Spoiler,
                                    offset: self.text_len(),
                                    length: 0,
                                }));
                            }
                        } else {
                            self.stacks.push(Stack::Other(name.clone()));
                        }
                    }
                    "tg-spoiler" => {
                        self.stacks.push(Stack::Entity(MessageEntity {
                            typ: MessageEntityType::Spoiler,
                            offset: self.text_len(),
                            length: 0,
                        }));
                    }
                    "a" => {
                        if let Some(href) = attrs
                            .borrow()
                            .iter()
                            .find(|attr| attr.name.local.as_ref() == "href")
                        {
                            self.stacks.push(Stack::Entity(MessageEntity {
                                typ: MessageEntityType::TextLink {
                                    url: href.value.to_string(),
                                },
                                offset: self.text_len(),
                                length: 0,
                            }));
                        } else {
                            self.stacks.push(Stack::Other(name.clone()));
                        }
                    }
                    "tg-emoji" => {
                        if let Some(emoji_id) = attrs
                            .borrow()
                            .iter()
                            .find(|attr| attr.name.local.as_ref() == "emoji-id")
                        {
                            self.stacks.push(Stack::Entity(MessageEntity {
                                typ: MessageEntityType::CustomEmoji {
                                    custom_emoji_id: emoji_id.value.to_string(),
                                },
                                offset: self.text_len(),
                                length: 0,
                            }));
                        } else {
                            self.stacks.push(Stack::Other(name.clone()));
                        }
                    }
                    "code" => {
                        let need_replace = if let Some(last) = self.stacks.last_mut() {
                            match &last {
                                Stack::Entity(entity) => match entity.typ {
                                    MessageEntityType::Pre { .. } => {
                                        entity.offset == self.text_len()
                                    }
                                    _ => false,
                                },
                                _ => false,
                            }
                        } else {
                            false
                        };
                        if need_replace {
                            self.stacks.pop();
                            self.stacks.push(Stack::Entity(MessageEntity {
                                typ: MessageEntityType::Pre {
                                    language: attrs
                                        .borrow()
                                        .iter()
                                        .find(|attr| attr.name.local.as_ref() == "class")
                                        .map(|attr| attr.value.to_string()),
                                },
                                offset: self.text_len(),
                                length: 0,
                            }));
                            self.stacks.push(Stack::Other(name.clone()));
                        } else {
                            self.stacks.push(Stack::Entity(MessageEntity {
                                typ: MessageEntityType::Code,
                                offset: self.text_len(),
                                length: 0,
                            }));
                        }
                    }
                    "pre" => {
                        self.stacks.push(Stack::Entity(MessageEntity {
                            typ: MessageEntityType::Pre { language: None },
                            offset: self.text_len(),
                            length: 0,
                        }));
                    }
                    _ => {
                        self.stacks.push(Stack::Other(name.clone()));
                    }
                }
                for i in node.children.borrow().iter() {
                    self.iter(i);
                }
                if let Some(i) = self
                    .stacks
                    .iter()
                    .rev()
                    .position(|s| s.is_equal(local_name))
                {
                    let entity = self.stacks.remove(self.stacks.len() - i - 1);
                    if let Stack::Entity(mut entity) = entity {
                        entity.length = self.text_len() - entity.offset;
                        self.entities.push(entity);
                    }
                } else {
                    log::warn!("Unknown tag: {}", local_name);
                }
            }
            _ => {}
        }
    }

    pub fn parse<S: AsRef<str> + ?Sized>(&mut self, text: &S) -> Result<(), PixivDownloaderError> {
        let opts = ParseOpts::default();
        let dom = parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut text.as_ref().replace("\n", "<br>").as_bytes())?;
        for node in dom.document.children.borrow().iter() {
            self.iter(node);
        }
        self.sort();
        if self.opts.scan_link {
            self.scan_link();
        }
        self.sort();
        Ok(())
    }

    fn scan_link(&mut self) {
        let mut offset = 0;
        while let Some(i) = self.text[offset..]
            .find("http://")
            .or_else(|| self.text[offset..].find("https://"))
        {
            let mut length = 0;
            for c in self.text[offset + i..].chars() {
                if c != ' ' && c != '\n' && c != '\r' && c != '\t' {
                    length += 1;
                } else {
                    break;
                }
            }
            let boffset = self.text[..offset + i].encode_utf16().count();
            let tlen = self.text[offset + i..offset + i + length].encode_utf16().count();
            if length > 0 && !self.is_conflict_with_link_entities(boffset, tlen) {
                self.entities.push(MessageEntity {
                    typ: MessageEntityType::Url,
                    offset: boffset,
                    length: tlen,
                });
            }
            offset += i + length;
        }
    }


    fn sort(&mut self) {
        self.entities.sort_by(|a, b| {
            a.offset
                .cmp(&b.offset)
                .then_with(|| a.length.cmp(&b.length))
        });
    }

    #[inline]
    fn text_len(&self) -> usize {
        self.text.encode_utf16().count()
    }
}

impl Default for TextSpliter {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl From<TextSpliterBuilder> for TextSpliter {
    fn from(opts: TextSpliterBuilder) -> Self {
        Self {
            entities: Vec::new(),
            text: String::new(),
            stacks: Vec::new(),
            opts,
        }
    }
}

#[test]
fn test_parse() {
    let mut spliter = TextSpliter::default();
    spliter.parse("<b>bold</b><a href=\"https://example.com\">exa</a><span class=\"tg-spoiler\">Spoiler</span><b>Bold<i>italicBold</i></b> <a>link</a><code>code</code><pre>code block</pre><pre><code class=\"language-shell\">/usr/bin/sh</code></pre><tg-emoji emoji-id=\"536\">👍</tg-emoji><tg-spoiler>Sp</tg-spoiler>\nhttps://www.pixiv.net\n<a href=\"https://b.com\">https://www.pixiv.net</a>\nhttps://www.pixiv<a href=\"https://www.pixiv.net\">.net</a>\nhttps://www.<strong>pixiv</strong>.net\n<u>下划线</u>").unwrap();
    assert_eq!(spliter.text, "boldexaSpoilerBolditalicBold linkcodecode block/usr/bin/sh👍Sp\nhttps://www.pixiv.net\nhttps://www.pixiv.net\nhttps://www.pixiv.net\nhttps://www.pixiv.net\n下划线");
    assert_eq!(
        spliter.entities,
        vec![
            MessageEntity {
                typ: MessageEntityType::Bold,
                offset: 0,
                length: 4,
            },
            MessageEntity {
                typ: MessageEntityType::TextLink {
                    url: "https://example.com".to_string(),
                },
                offset: 4,
                length: 3,
            },
            MessageEntity {
                typ: MessageEntityType::Spoiler,
                offset: 7,
                length: 7,
            },
            MessageEntity {
                typ: MessageEntityType::Bold,
                offset: 14,
                length: 14,
            },
            MessageEntity {
                typ: MessageEntityType::Italic,
                offset: 18,
                length: 10,
            },
            MessageEntity {
                typ: MessageEntityType::Code,
                offset: 33,
                length: 4,
            },
            MessageEntity {
                typ: MessageEntityType::Pre { language: None },
                offset: 37,
                length: 10,
            },
            MessageEntity {
                typ: MessageEntityType::Pre {
                    language: Some("language-shell".to_string()),
                },
                offset: 47,
                length: 11,
            },
            MessageEntity {
                typ: MessageEntityType::CustomEmoji {
                    custom_emoji_id: "536".to_string(),
                },
                offset: 58,
                length: 2,
            },
            MessageEntity {
                typ: MessageEntityType::Spoiler,
                offset: 60,
                length: 2,
            },
            MessageEntity {
                typ: MessageEntityType::Url,
                offset: 63,
                length: 21,
            },
            MessageEntity {
                typ: MessageEntityType::TextLink {
                    url: "https://b.com".to_string(),
                },
                offset: 85,
                length: 21,
            },
            MessageEntity {
                typ: MessageEntityType::TextLink {
                    url: "https://www.pixiv.net".to_string(),
                },
                offset: 124,
                length: 4,
            },
            MessageEntity {
                typ: MessageEntityType::Url,
                offset: 129,
                length: 21,
            },
            MessageEntity {
                typ: MessageEntityType::Bold,
                offset: 141,
                length: 5,
            },
            MessageEntity {
                typ: MessageEntityType::Underline,
                offset: 151,
                length: 3,
            },
        ]
    );
}

#[test]
fn test_parse2() {
    let mut spliter = TextSpliter::builder().disable_scan_link().build();
    spliter.parse("<b>中文</b> https://www.pixiv.net").unwrap();
    assert_eq!(spliter.text, "中文 https://www.pixiv.net");
    assert_eq!(
        spliter.entities,
        vec![MessageEntity {
            typ: MessageEntityType::Bold,
            offset: 0,
            length: 2,
        },]
    );
}

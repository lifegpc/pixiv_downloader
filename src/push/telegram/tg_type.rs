use crate::formdata::{FormDataBody, FormDataPart};
use derive_builder::Builder;
use derive_more::From;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, PartialOrd, From, Deserialize, Serialize)]
#[serde(untagged)]
/// Unique identifier for the target chat or username of the target channel
pub enum ChatId {
    /// Unique id for chat
    Id(i64),
    /// Username of the target channel, `@channelusername`
    Username(String),
}

impl FromStr for ChatId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<i64>() {
            Ok(i) => Ok(ChatId::Id(i)),
            Err(_) => {
                if s.starts_with("@") {
                    Ok(ChatId::Username(s.to_owned()))
                } else {
                    Err(String::from("Failed to parse chat id."))
                }
            }
        }
    }
}

impl ToString for ChatId {
    fn to_string(&self) -> String {
        match self {
            ChatId::Id(i) => format!("{}", i),
            ChatId::Username(s) => s.to_owned(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Deserialize, Serialize)]
/// Formatting options
/// See https://core.telegram.org/bots/api#formatting-options
pub enum ParseMode {
    /// https://core.telegram.org/bots/api#markdownv2-style
    MarkdownV2,
    /// https://core.telegram.org/bots/api#html-style
    HTML,
    /// https://core.telegram.org/bots/api#markdown-style
    Markdown,
}

impl AsRef<str> for ParseMode {
    fn as_ref(&self) -> &str {
        match self {
            ParseMode::HTML => "HTML",
            ParseMode::Markdown => "Markdown",
            ParseMode::MarkdownV2 => "MarkdownV2",
        }
    }
}

#[derive(Builder, Clone, Debug, Default, PartialEq, PartialOrd, Deserialize, Serialize)]
#[builder(setter(strip_option))]
#[builder(default)]
/// Describes the options used for link preview generation.
/// See https://core.telegram.org/bots/api#linkpreviewoptions
pub struct LinkPreviewOptions {
    /// Optional. True, if the link preview is disabled
    #[serde(skip_serializing_if = "Option::is_none")]
    is_disabled: Option<bool>,
    /// Optional. URL to use for the link preview.
    /// If empty, then the first URL found in the message text will be used
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    /// Optional. True, if the media in the link preview is supposed to be shrunk;
    /// ignored if the URL isn't explicitly specified or media size change isn't supported for the preview
    #[serde(skip_serializing_if = "Option::is_none")]
    prefer_small_media: Option<bool>,
    /// Optional. True, if the media in the link preview is supposed to be enlarged;
    /// ignored if the URL isn't explicitly specified or media size change isn't supported for the preview
    #[serde(skip_serializing_if = "Option::is_none")]
    prefer_large_media: Option<bool>,
    /// Optional. True, if the link preview must be shown above the message text;
    /// otherwise, the link preview will be shown below the message text
    #[serde(skip_serializing_if = "Option::is_none")]
    show_above_text: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum BotApiResult<T>
where
    T: Clone + Serialize,
{
    Ok {
        ok: bool,
        result: T,
    },
    Failed {
        ok: bool,
        error_code: i64,
        description: String,
    },
}

impl<T> BotApiResult<T>
where
    T: Clone + Serialize,
{
    pub fn unwrap(self) -> T {
        match self {
            Self::Ok { result, .. } => result,
            Self::Failed {
                description,
                error_code,
                ..
            } => panic!("{} ({})", description, error_code),
        }
    }

    pub fn to_result(self) -> Result<T, String> {
        match self {
            Self::Ok { result, .. } => Ok(result),
            Self::Failed {
                description,
                error_code,
                ..
            } => Err(format!("{} ({})", description, error_code)),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// This object represents a message.
/// https://core.telegram.org/bots/api#message
pub struct Message {
    /// Unique message identifier inside this chat
    pub message_id: i64,
    /// Optional. Unique identifier of a message thread to which the message belongs;
    /// for supergroups only
    pub message_thread_id: Option<i64>,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
pub struct ReplyParameters {
    /// Identifier of the message that will be replied to in the current chat,
    /// or in the chat chat_id if it is specified
    message_id: i64,
    /// Optional. If the message to be replied to is from a different chat,
    /// unique identifier for the chat or username of the channel (in the format `@channelusername`).
    /// Not supported for messages sent on behalf of a business account.
    #[builder(default, setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    chat_id: Option<ChatId>,
    /// Optional. Pass True if the message should be sent even if the specified message
    /// to be replied to is not found. Always False for replies in another chat or forum topic.
    /// Always True for messages sent on behalf of a business account.
    #[builder(setter(strip_option))]
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    allow_sending_without_reply: Option<bool>,
    /// Optional. Quoted part of the message to be replied to; 0-1024 characters after entities parsing.
    /// The quote must be an exact substring of the message to be replied to,
    /// including bold, italic, underline, strikethrough, spoiler, and custom_emoji entities.
    /// The message will fail to send if the quote isn't found in the original message.
    #[builder(default, setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    quote: Option<String>,
    /// Optional. Mode for parsing entities in the quote. See formatting options for more details.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    quote_parse_mode: Option<ParseMode>,
    /// Optional. Position of the quote in the original message in UTF-16 code units
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    quote_position: Option<i64>,
}

#[derive(Debug, derive_more::From)]
/// Represents the contents of a file
pub enum InputFile {
    /// URL
    URL(String),
    /// File data
    Content(FormDataPart),
}

impl InputFile {
    pub async fn get_size(&self) -> Result<Option<u64>, std::io::Error> {
        match self {
            Self::URL(_) => Ok(None),
            Self::Content(c) => match c.body() {
                FormDataBody::Data(d) => Ok(Some(d.len() as u64)),
                FormDataBody::File(f) => {
                    let m = tokio::fs::metadata(f).await?;
                    Ok(Some(m.len()))
                }
            },
        }
    }
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(setter(into))]
/// Represents an audio file to be treated as music to be sent.
pub struct InputMediaAudio {
    /// File to send. Pass a file_id to send a file that exists on the Telegram servers (recommended),
    /// pass an HTTP URL for Telegram to get a file from the Internet,
    /// or pass “attach://<file_attach_name>” to upload a new one using
    /// multipart/form-data under <file_attach_name> name.
    media: String,
    /// Optional. Thumbnail of the file sent; can be ignored if thumbnail generation for the file
    /// is supported server-side. The thumbnail should be in JPEG format and less than 200 kB in size.
    /// A thumbnail's width and height should not exceed 320. Ignored if the file is not uploaded
    /// using multipart/form-data. Thumbnails can't be reused and can be only uploaded as a new file,
    /// so you can pass “attach://<file_attach_name>” if the thumbnail was uploaded using
    /// multipart/form-data under <file_attach_name>.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail: Option<String>,
    /// Optional. Caption of the audio to be sent, 0-1024 characters after entities parsing
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    caption: Option<String>,
    /// Optional. Mode for parsing entities in the audio caption. See formatting options for more details.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<ParseMode>,
    /// Optional. Duration of the audio in seconds
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<i64>,
    /// Optional. Performer of the audio
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    performer: Option<String>,
    /// Optional. Title of the audio
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(setter(into))]
/// Represents a general file to be sent.
pub struct InputMediaDocument {
    /// File to send. Pass a file_id to send a file that exists on the Telegram servers (recommended),
    /// pass an HTTP URL for Telegram to get a file from the Internet,
    /// or pass “attach://<file_attach_name>” to upload a new one using
    /// multipart/form-data under <file_attach_name> name.
    media: String,
    /// Optional. Thumbnail of the file sent; can be ignored if thumbnail generation for the file
    /// is supported server-side. The thumbnail should be in JPEG format and less than 200 kB in size.
    /// A thumbnail's width and height should not exceed 320. Ignored if the file is not uploaded
    /// using multipart/form-data. Thumbnails can't be reused and can be only uploaded as a new file,
    /// so you can pass “attach://<file_attach_name>” if the thumbnail was uploaded using
    /// multipart/form-data under <file_attach_name>.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail: Option<String>,
    /// Optional. Caption of the document to be sent, 0-1024 characters after entities parsing
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    caption: Option<String>,
    /// Optional. Mode for parsing entities in the document caption.
    /// See formatting options for more details.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<ParseMode>,
    /// Optional. Disables automatic server-side content type detection for files uploaded
    /// using multipart/form-data. Always True, if the document is sent as part of an album.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    disable_content_type_detection: Option<bool>,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(setter(into))]
/// Represents a photo to be sent.
pub struct InputMediaPhoto {
    /// File to send. Pass a file_id to send a file that exists on the Telegram servers (recommended),
    /// pass an HTTP URL for Telegram to get a file from the Internet, or pass
    /// “attach://<file_attach_name>” to upload a new one using multipart/form-data
    /// under <file_attach_name> name.
    media: String,
    /// Optional. Caption of the photo to be sent, 0-1024 characters after entities parsing
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    caption: Option<String>,
    /// Optional. Mode for parsing entities in the photo caption. See formatting options for more details.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<ParseMode>,
    /// Optional. Pass True, if the caption must be shown above the message media
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    show_caption_above_media: Option<bool>,
    /// Optional. Pass True if the photo needs to be covered with a spoiler animations
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    has_spoiler: Option<bool>,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(setter(into))]
/// Represents a video to be sent.
pub struct InputMediaVideo {
    /// File to send. Pass a file_id to send a file that exists on the Telegram servers (recommended),
    /// pass an HTTP URL for Telegram to get a file from the Internet, or pass
    /// “attach://<file_attach_name>” to upload a new one using multipart/form-data under
    /// <file_attach_name> name.
    media: String,
    /// Optional. Thumbnail of the file sent; can be ignored if thumbnail generation for the file
    /// is supported server-side. The thumbnail should be in JPEG format and less than 200 kB in size.
    /// A thumbnail's width and height should not exceed 320. Ignored if the file is not uploaded
    /// using multipart/form-data. Thumbnails can't be reused and can be only uploaded as a new file,
    /// so you can pass “attach://<file_attach_name>” if the thumbnail was uploaded using
    /// multipart/form-data under <file_attach_name>.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail: Option<String>,
    /// Optional. Caption of the video to be sent, 0-1024 characters after entities parsing
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    caption: Option<String>,
    /// Optional. Mode for parsing entities in the video caption. See formatting options for more details.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<ParseMode>,
    /// Optional. Pass True, if the caption must be shown above the message media
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    show_caption_above_media: Option<bool>,
    /// Optional. Video width
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<i64>,
    /// Optional. Video height
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<i64>,
    /// Optional. Video duration in seconds
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<i64>,
    /// Optional. Pass True if the uploaded video is suitable for streaming
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    supports_streaming: Option<bool>,
    /// Optional. Pass True if the video needs to be covered with a spoiler animation
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    has_spoiler: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, derive_more::From)]
#[serde(rename_all = "camelCase", tag = "type")]
/// This object represents the content of a media message to be sent.
pub enum InputMedia {
    Audio(InputMediaAudio),
    Document(InputMediaDocument),
    Photo(InputMediaPhoto),
    Video(InputMediaVideo),
}

#[test]
fn test_chat_id() {
    assert_eq!(
        serde_json::from_str::<ChatId>("32").unwrap(),
        ChatId::Id(32)
    );
}

#[test]
fn test_parse_mode() {
    assert_eq!(
        serde_json::from_str::<ParseMode>("\"MarkdownV2\"").unwrap(),
        ParseMode::MarkdownV2
    );
}

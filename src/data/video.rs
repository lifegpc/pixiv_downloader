use crate::avdict::AVDict;
use crate::avdict::AVDictError;
use crate::data::data::PixivData;
use crate::parser::description::parse_description;

pub fn get_video_metadata(data: &PixivData) -> Result<AVDict, AVDictError> {
    let mut d = AVDict::new();
    if data.title.is_some() {
        let t = data.title.as_ref().unwrap();
        d.set("title", t, None)?;
    }
    if data.author.is_some() {
        let au = data.author.as_ref().unwrap();
        d.set("artist", au, None)?;
    }
    if data.description.is_some() {
        let odesc = data.description.as_ref().unwrap();
        let desc = parse_description(odesc);
        let des = match &desc {
            Some(d) => { d }
            None => { odesc }
        };
        d.set("comment", des, None)?;
    }
    Ok(d)
}

use crate::data::data::PixivData;
use crate::exif::ExifByteOrder;
use crate::exif::ExifData;
use crate::exif::ExifImage;
use crate::exif::ExifKey;
use crate::exif::ExifTypeID;
use crate::exif::ExifValue;
use crate::parser::description::parse_description;
use std::convert::TryFrom;
use std::ffi::OsStr;
use utf16string::LittleEndian;
use utf16string::WString;

fn add_image_id(data: &mut ExifData, d: &PixivData) -> Result<(), ()> {
    let link = d.id.to_link();
    let key = ExifKey::try_from("Exif.Image.ImageID")?;
    let mut value = ExifValue::try_from(ExifTypeID::AsciiString)?;
    value.read(link.as_bytes(), None)?;
    data.add(&key, &value)?;
    Ok(())
}

fn add_image_title(data: &mut ExifData, d: &PixivData) -> Result<(), ()> {
    if d.title.is_none() {
        return Ok(());
    }
    let title = d.title.as_ref().unwrap();
    let key = ExifKey::try_from("Exif.Image.ImageDescription")?;
    let mut value = ExifValue::try_from(ExifTypeID::AsciiString)?;
    value.read(title.as_bytes(), None)?;
    data.add(&key, &value)?;
    let key = ExifKey::try_from("Exif.Image.XPTitle")?;
    let mut value = ExifValue::try_from(ExifTypeID::BYTE)?;
    let s: WString<LittleEndian> = WString::from(title);
    value.read(s.as_bytes(), None)?;
    data.add(&key, &value)?;
    Ok(())
}

fn add_image_author(data: &mut ExifData, d: &PixivData) -> Result<(), ()> {
    if d.author.is_none() {
        return Ok(());
    }
    let author = d.author.as_ref().unwrap();
    let key = ExifKey::try_from("Exif.Image.XPAuthor")?;
    let mut value = ExifValue::try_from(ExifTypeID::BYTE)?;
    let s: WString<LittleEndian> = WString::from(author);
    value.read(s.as_bytes(), None)?;
    data.add(&key, &value)?;
    let key = ExifKey::try_from("Exif.Image.Artist")?;
    let mut value = ExifValue::try_from(ExifTypeID::AsciiString)?;
    value.read(author.as_bytes(), None)?;
    data.add(&key, &value)?;
    Ok(())
}

fn add_image_comment(data: &mut ExifData, d: &PixivData) -> Result<(), ()> {
    if d.description.is_none() {
        return Ok(());
    }
    let desc = parse_description(d.description.as_ref().unwrap());
    let desc = if desc.is_some() {
        desc.as_ref().unwrap()
    } else {
        d.description.as_ref().unwrap()
    };
    let key = ExifKey::try_from("Exif.Image.XPComment")?;
    let mut value = ExifValue::try_from(ExifTypeID::BYTE)?;
    let s: WString<LittleEndian> = WString::from(desc);
    value.read(s.as_bytes(), None)?;
    data.add(&key, &value)?;
    Ok(())
}

fn add_image_page(data: &mut ExifData, page: u16) -> Result<(), ()> {
    let key = ExifKey::try_from("Exif.Image.PageNumber")?;
    let mut value = ExifValue::try_from(ExifTypeID::UShort)?;
    value.read(&page.to_le_bytes(), Some(ExifByteOrder::Little))?;
    data.add(&key, &value)?;
    Ok(())
}

pub fn add_exifdata_to_image<S: AsRef<OsStr> + ?Sized>(file_name: &S, data: &PixivData, page: u16) -> Result<(), ()> {
    let mut f = ExifImage::new(file_name)?;
    let mut d = ExifData::new()?;
    add_image_id(&mut d, data)?;
    add_image_title(&mut d, data)?;
    add_image_author(&mut d, data)?;
    add_image_comment(&mut d, data)?;
    add_image_page(&mut d, page)?;
    f.set_exif_data(&d)?;
    f.write_metadata()?;
    Ok(())
}

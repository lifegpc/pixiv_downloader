use crate::exif::ExifByteOrder;
use crate::exif::ExifData;
use crate::exif::ExifImage;
use crate::exif::ExifKey;
use crate::exif::ExifTypeID;
use crate::exif::ExifValue;
use crate::ext::try_err::TryErr;
use crate::parser::description::parse_description;
use proc_macros::call_parent_data_source_fun;
use proc_macros::define_exif_data_source;
use std::ffi::OsStr;
use utf16string::LittleEndian;
use utf16string::WString;

pub trait ExifDataSource {
    define_exif_data_source!("src/data/exif_data_source.json");
}

impl<T: ExifDataSource> ExifDataSource for std::sync::Arc<T> {
    call_parent_data_source_fun!("src/data/exif_data_source.json", **self,);
}

impl<T: ExifDataSource> ExifDataSource for Option<T> {
    call_parent_data_source_fun!(
        "src/data/exif_data_source.json",
        match self {
            Some(data) => data,
            None => {
                return None;
            }
        },
    );
}

fn add_image_id<D: ExifDataSource>(data: &mut ExifData, d: &D) -> Result<(), ()> {
    let link = match d.image_id() {
        Some(link) => link,
        None => return Ok(()),
    };
    let key = ExifKey::try_from("Exif.Image.ImageID")?;
    let mut value = ExifValue::try_from(ExifTypeID::AsciiString)?;
    value.read(link.as_bytes(), None)?;
    data.add(&key, &value)?;
    Ok(())
}

fn add_image_title<D: ExifDataSource>(data: &mut ExifData, d: &D) -> Result<(), ()> {
    let title = match d.image_title() {
        Some(title) => title,
        None => return Ok(()),
    };
    let key = ExifKey::try_from("Exif.Image.ImageDescription")?;
    let mut value = ExifValue::try_from(ExifTypeID::AsciiString)?;
    value.read(title.as_bytes(), None)?;
    data.add(&key, &value)?;
    let key = ExifKey::try_from("Exif.Image.XPTitle")?;
    let mut value = ExifValue::try_from(ExifTypeID::BYTE)?;
    let s: WString<LittleEndian> = WString::from(title.as_str());
    value.read(s.as_bytes(), None)?;
    data.add(&key, &value)?;
    Ok(())
}

fn add_image_author<D: ExifDataSource>(data: &mut ExifData, d: &D) -> Result<(), ()> {
    let author = match d.image_author() {
        Some(author) => author,
        None => {
            return Ok(());
        }
    };
    let key = ExifKey::try_from("Exif.Image.XPAuthor")?;
    let mut value = ExifValue::try_from(ExifTypeID::BYTE)?;
    let s: WString<LittleEndian> = WString::from(author.as_str());
    value.read(s.as_bytes(), None)?;
    data.add(&key, &value)?;
    let key = ExifKey::try_from("Exif.Image.Artist")?;
    let mut value = ExifValue::try_from(ExifTypeID::AsciiString)?;
    value.read(author.as_bytes(), None)?;
    data.add(&key, &value)?;
    Ok(())
}

fn add_image_comment<D: ExifDataSource>(data: &mut ExifData, d: &D) -> Result<(), ()> {
    let odesc = match d.image_comment() {
        Some(desc) => desc,
        None => {
            return Ok(());
        }
    };
    let desc = match parse_description(odesc.as_str()) {
        Some(desc) => desc,
        None => odesc,
    };
    let key = ExifKey::try_from("Exif.Image.XPComment")?;
    let mut value = ExifValue::try_from(ExifTypeID::BYTE)?;
    let s: WString<LittleEndian> = WString::from(desc.as_str());
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

pub fn add_exifdata_to_image<S: AsRef<OsStr> + ?Sized, D: ExifDataSource>(
    file_name: &S,
    data: &D,
    page: u16,
) -> Result<(), ()> {
    let mut f = ExifImage::new(file_name)?;
    // # TODO: do duplicate check
    // f.read_metadata()?;
    let mut d = f.exif_data().try_err(())?.to_owned();
    add_image_id(&mut d, data)?;
    add_image_title(&mut d, data)?;
    add_image_author(&mut d, data)?;
    add_image_comment(&mut d, data)?;
    add_image_page(&mut d, page)?;
    f.set_exif_data(&d)?;
    f.write_metadata()?;
    Ok(())
}

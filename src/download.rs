use crate::data::data::PixivData;
#[cfg(feature = "exif")]
use crate::data::exif::add_exifdata_to_image;
use crate::data::json::JSONDataFile;
use crate::gettext;
use crate::pixiv_link::PixivID;
use crate::pixiv_web::PixivWebClient;
use crate::utils::get_file_name_from_url;
use crate::webclient::WebClient;
use crate::Main;
use json::JsonValue;
use std::path::PathBuf;

impl Main {
    pub fn download(&mut self) -> i32 {
        let mut pw = PixivWebClient::new(&self);
        if !pw.init() {
            println!("{}", gettext("Failed to initialize pixiv web api client."));
            return 1;
        }
        pw.check_login();
        if !pw.logined() {
            println!(
                "{}",
                gettext("Warning: Web api client not logined, some future may not work.")
            );
        }
        for id in self.cmd.as_ref().unwrap().ids.iter() {
            match id {
                PixivID::Artwork(id) => {
                    let r = self.download_artwork(&mut pw, id.clone());
                    if r != 0 {
                        return r;
                    }
                }
            }
        }
        0
    }

    pub fn download_artwork(&self, pw: &mut PixivWebClient, id: u64) -> i32 {
        let re = pw.get_artwork(id);
        if re.is_none() {
            return 1;
        }
        let re = re.unwrap();
        let pages = (&re["illust"][format!("{}", id).as_str()]["pageCount"]).as_u64();
        if pages.is_none() {
            println!("{}", gettext("Failed to get page count."));
            return 1;
        }
        let pages = pages.unwrap();
        let mut pages_data: Option<JsonValue> = None;
        if pages > 1 {
            pages_data = pw.get_illust_pages(id);
        }
        if pages > 1 && pages_data.is_none() {
            println!("{}", gettext("Failed to get pages' data."));
            return 1;
        }
        let base = PathBuf::from(".");
        let json_file = base.join(format!("{}.json", id));
        let mut datas = PixivData::new(id).unwrap();
        datas.from_web_page_data(&re, true);
        let json_data = JSONDataFile::from(&datas);
        if !json_data.save(&json_file) {
            println!("{}", gettext("Failed to save metadata to JSON file."));
            return 1;
        }
        if pages_data.is_some() {
            let pages_data = pages_data.as_ref().unwrap();
            for page in pages_data.members() {
                let link = &page["urls"]["original"];
                if !link.is_string() {
                    println!("{}", gettext("Failed to get original picture's link."));
                    return 1;
                }
                let link = link.as_str().unwrap();
                let file_name = get_file_name_from_url(link);
                if file_name.is_none() {
                    println!("{} {}", gettext("Failed to get file name from url:"), link);
                    return 1;
                }
                let file_name = file_name.unwrap();
                let file_name = base.join(file_name);
                let r = pw.download_image(link);
                if r.is_none() {
                    println!("{} {}", gettext("Failed to download image:"), link);
                    return 1;
                }
                let r = r.unwrap();
                let re = WebClient::download_stream(&file_name, r, pw.helper.overwrite());
                if re.is_err() {
                    println!("{} {}", gettext("Failed to download image:"), link);
                    return 1;
                }
                println!(
                    "{} {} -> {}",
                    gettext("Downloaded image:"),
                    link,
                    file_name.to_str().unwrap_or("(null)")
                );
                #[cfg(feature = "exif")]
                {
                    if add_exifdata_to_image(&file_name, &datas).is_err() {
                        println!(
                            "{} {}",
                            gettext("Failed to add exif data to image:"),
                            file_name.to_str().unwrap_or("(null)")
                        );
                    }
                }
            }
        } else {
            let link = (&re["illust"][format!("{}", id)]["urls"]["original"]).as_str();
            if link.is_none() {
                println!("{}", gettext("Failed to get original picture's link."));
                return 1;
            }
            let link = link.unwrap();
            let file_name = get_file_name_from_url(link);
            if file_name.is_none() {
                println!("{} {}", gettext("Failed to get file name from url:"), link);
                return 1;
            }
            let file_name = file_name.unwrap();
            let file_name = base.join(file_name);
            let r = pw.download_image(link);
            if r.is_none() {
                println!("{} {}", gettext("Failed to download image:"), link);
                return 1;
            }
            let r = r.unwrap();
            let re = WebClient::download_stream(&file_name, r, pw.helper.overwrite());
            if re.is_err() {
                println!("{} {}", gettext("Failed to download image:"), link);
                return 1;
            }
            println!(
                "{} {} -> {}",
                gettext("Downloaded image:"),
                link,
                file_name.to_str().unwrap_or("(null)")
            );
            #[cfg(feature = "exif")]
            {
                if add_exifdata_to_image(&file_name, &datas).is_err() {
                    println!(
                        "{} {}",
                        gettext("Failed to add exif data to image:"),
                        file_name.to_str().unwrap_or("(null)")
                    );
                }
            }
        }
        0
    }
}

#[cfg(feature = "avdict")]
use crate::avdict::AVDict;
use crate::data::data::PixivData;
#[cfg(feature = "exif")]
use crate::data::exif::add_exifdata_to_image;
use crate::data::json::JSONDataFile;
#[cfg(feature = "ugoira")]
use crate::data::video::get_video_metadata;
use crate::gettext;
use crate::opthelper::OptHelper;
use crate::pixiv_link::PixivID;
use crate::pixiv_web::PixivWebClient;
#[cfg(feature = "ugoira")]
use crate::ugoira::{UgoiraFrames, convert_ugoira_to_mp4};
use crate::utils::ask_need_overwrite;
use crate::utils::get_file_name_from_url;
use crate::webclient::WebClient;
use crate::Main;
use indicatif::MultiProgress;
use json::JsonValue;
use spin_on::spin_on;
use std::path::PathBuf;
use std::sync::Arc;

impl Main {
    pub fn download(&mut self) -> i32 {
        let pw = Arc::new(PixivWebClient::new(&self));
        if !pw.init() {
            println!("{}", gettext("Failed to initialize pixiv web api client."));
            return 1;
        }
        if !pw.check_login() {
            return 1;
        }
        if !pw.logined() {
            println!(
                "{}",
                gettext("Warning: Web api client not logined, some future may not work.")
            );
        }
        for id in self.cmd.as_ref().unwrap().ids.iter() {
            match id {
                PixivID::Artwork(id) => {
                    let r = self.download_artwork(Arc::clone(&pw), id.clone());
                    if r != 0 {
                        return r;
                    }
                }
            }
        }
        0
    }

    pub async fn download_artwork_page(pw: Arc<PixivWebClient>, page: JsonValue, np: u16, progress_bars: Arc<MultiProgress>, datas: Arc<PixivData>, base: Arc<PathBuf>, helper: Arc<OptHelper>) -> i32 {
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
        if file_name.exists() {
            match helper.overwrite() {
                Some(overwrite) => {
                    if !overwrite {
                        #[cfg(feature = "exif")]
                        {
                            if helper.update_exif() {
                                if add_exifdata_to_image(&file_name, &datas, np).is_err() {
                                    println!(
                                        "{} {}",
                                        gettext("Failed to add exif data to image:"),
                                        file_name.to_str().unwrap_or("(null)")
                                    );
                                }
                            }
                        }
                        return 0;
                    }
                }
                None => {
                    if !ask_need_overwrite(file_name.to_str().unwrap()) {
                        return 0;
                    }
                }
            }
        }
        let r;
        {
            r = pw.adownload_image(link).await;
            if r.is_none() {
                println!("{} {}", gettext("Failed to download image:"), link);
                return 1;
            }
        }
        let r = r.unwrap();
        let re = WebClient::adownload_stream(&file_name, r, &helper, Some(progress_bars)).await;
        if re.is_err() {
            println!("{} {}", gettext("Failed to download image:"), link);
            return 1;
        }
        #[cfg(feature = "exif")]
        {
            if add_exifdata_to_image(&file_name, &datas, np).is_err() {
                println!(
                    "{} {}",
                    gettext("Failed to add exif data to image:"),
                    file_name.to_str().unwrap_or("(null)")
                );
            }
        }
        0
    }

    pub fn download_artwork(&self, pw: Arc<PixivWebClient>, id: u64) -> i32 {
        let mut re = None;
        let pages;
        let mut ajax_ver = true;
        let helper = Arc::new(pw.helper.clone());
        if helper.use_webpage() {
            re = pw.get_artwork(id);
            if re.is_some() {
                ajax_ver = false;
            }
        }
        if re.is_none() {
            re = pw.get_artwork_ajax(id);
        }
        if re.is_none() {
            return 1;
        }
        let re = re.unwrap();
        if ajax_ver {
            pages = (&re["pageCount"]).as_u64();
        } else {
            pages = (&re["illust"][format!("{}", id).as_str()]["pageCount"]).as_u64();
        }
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
        let base = Arc::new(PathBuf::from("."));
        let json_file = base.join(format!("{}.json", id));
        let mut datas = PixivData::new(id, (*helper).clone()).unwrap();
        if ajax_ver {
            datas.from_web_page_ajax_data(&re, true);
        } else {
            datas.from_web_page_data(&re, true);
        }
        let datas = Arc::new(datas);
        let json_data = JSONDataFile::from(Arc::clone(&datas));
        if !json_data.save(&json_file) {
            println!("{}", gettext("Failed to save metadata to JSON file."));
            return 1;
        }
        let illust_type = if ajax_ver {
            (&re["illustType"]).as_i64()
        } else {
            (&re["illust"][format!("{}", id).as_str()]["illustType"]).as_i64()
        };
        if illust_type.is_some() {
            let illust_type = illust_type.unwrap();
            match illust_type {
                0 => { }
                2 => {
                    let ugoira_data = pw.get_ugoira(id);
                    if ugoira_data.is_none() {
                        println!("{}", gettext("Failed to get ugoira's data."));
                        return 1;
                    }
                    let ugoira_data = ugoira_data.unwrap();
                    let src = (&ugoira_data["originalSrc"]).as_str();
                    if src.is_none() {
                        println!("{}", gettext("Can not find source link for ugoira."));
                        return 1;
                    }
                    let src = src.unwrap();
                    let file_name = get_file_name_from_url(src);
                    if file_name.is_none() {
                        println!("{} {}", gettext("Failed to get file name from url:"), src);
                        return 1;
                    }
                    let file_name = file_name.unwrap();
                    let file_name = base.join(file_name);
                    let dw = if file_name.exists() {
                        match helper.overwrite() {
                            Some(overwrite) => { overwrite }
                            None => { ask_need_overwrite(file_name.to_str().unwrap()) }
                        }
                    } else {
                        true
                    };
                    if dw {
                        let r = pw.download_image(src);
                        if r.is_none() {
                            println!("{} {}", gettext("Failed to download ugoira:"), src);
                            return 1;
                        }
                        let r = r.unwrap();
                        let re = WebClient::download_stream(&file_name, r, &helper);
                        if re.is_err() {
                            println!("{} {}", gettext("Failed to download ugoira:"), src);
                            return 1;
                        }
                        println!(
                            "{} {} -> {}",
                            gettext("Downloaded ugoira:"),
                            src,
                            file_name.to_str().unwrap_or("(null)")
                        );
                    }
                    #[cfg(feature = "ugoira")]
                    {
                        let metadata = match get_video_metadata(Arc::clone(&datas).as_ref()) {
                            Ok(m) => { m }
                            Err(e) => {
                                println!("{} {}", gettext("Warning: Failed to generate video's metadata:"), e);
                                AVDict::new()
                            }
                        };
                        let options = AVDict::new();
                        match UgoiraFrames::from_json(&ugoira_data["frames"]) {
                            Ok(frames) => {
                                let output_file_name = base.join(format!("{}.mp4", id));
                                let re = convert_ugoira_to_mp4(&file_name, &output_file_name, &frames, 60f32, &options, &metadata);
                                if re.is_err() {
                                    println!("{} {}", gettext("Failed to convert from ugoira to mp4 video file:"), re.unwrap_err());
                                    return 1;
                                }
                                println!("{}", gettext("Converted <src> -> <dest>").replace("<src>", file_name.to_str().unwrap_or("(null)")).replace("<dest>", output_file_name.to_str().unwrap_or("(null)")).as_str());
                            }
                            Err(e) => {
                                println!("{} {}", gettext("Failed to parse frames:"), e);
                                return 1;
                            }
                        }
                    }
                    return 0;
                }
                _ => { println!("{} {}", gettext("Warning: Unknown illust type:"), illust_type) }
            }
        } else {
            println!("{}", gettext("Warning: Failed to get illust's type."));
        }
        if pages_data.is_some() && helper.download_multiple_images() {
            let mut np = 0u16;
            let pages_data = pages_data.as_ref().unwrap();
            let progress_bars = Arc::new(MultiProgress::new());
            let mut tasks = Vec::new();
            for page in pages_data.members() {
                let f = tokio::spawn(Self::download_artwork_page(Arc::clone(&pw), page.clone(), np, Arc::clone(&progress_bars), Arc::clone(&datas), Arc::clone(&base), Arc::clone(&helper)));
                tasks.push(f);
                np += 1;
            }
            let mut re = 0;
            for task in tasks {
                let r = spin_on(task);
                re |= r.unwrap_or(1);
            }
            return re;
        }
        else if pages_data.is_some() {
            #[cfg(feature = "exif")]
            let mut np = 0u16;
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
                if file_name.exists() {
                    match helper.overwrite() {
                        Some(overwrite) => {
                            if !overwrite {
                                #[cfg(feature = "exif")]
                                {
                                    if helper.update_exif() {
                                        if add_exifdata_to_image(&file_name, &datas, np).is_err() {
                                            println!(
                                                "{} {}",
                                                gettext("Failed to add exif data to image:"),
                                                file_name.to_str().unwrap_or("(null)")
                                            );
                                        }
                                    }
                                    np += 1;
                                }
                                continue;
                            }
                        }
                        None => {
                            if !ask_need_overwrite(file_name.to_str().unwrap()) {
                                continue;
                            }
                        }
                    }
                }
                let r = pw.download_image(link);
                if r.is_none() {
                    println!("{} {}", gettext("Failed to download image:"), link);
                    return 1;
                }
                let r = r.unwrap();
                let re = WebClient::download_stream(&file_name, r, &helper);
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
                    if add_exifdata_to_image(&file_name, &datas, np).is_err() {
                        println!(
                            "{} {}",
                            gettext("Failed to add exif data to image:"),
                            file_name.to_str().unwrap_or("(null)")
                        );
                    }
                    np += 1;
                }
            }
        } else {
            let link = if ajax_ver {
                (&re["urls"]["original"]).as_str()
            } else {
                (&re["illust"][format!("{}", id)]["urls"]["original"]).as_str()
            };
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
            if file_name.exists() {
                let overwrite = match helper.overwrite() {
                    Some(overwrite) => {
                        overwrite
                    }
                    None => {
                        ask_need_overwrite(file_name.to_str().unwrap())
                    }
                };
                if !overwrite {
                    #[cfg(feature = "exif")]
                    if helper.update_exif() {
                        if add_exifdata_to_image(&file_name, &datas, 0).is_err() {
                            println!(
                                "{} {}",
                                gettext("Failed to add exif data to image:"),
                                file_name.to_str().unwrap_or("(null)")
                            );
                        }
                    }
                    return 0;
                }
            }
            let r = pw.download_image(link);
            if r.is_none() {
                println!("{} {}", gettext("Failed to download image:"), link);
                return 1;
            }
            let r = r.unwrap();
            let re = WebClient::download_stream(&file_name, r, &helper);
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
                if add_exifdata_to_image(&file_name, &datas, 0).is_err() {
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

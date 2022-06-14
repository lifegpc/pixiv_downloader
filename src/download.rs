#[cfg(feature = "avdict")]
use crate::avdict::AVDict;
use crate::data::data::PixivData;
#[cfg(feature = "exif")]
use crate::data::exif::add_exifdata_to_image;
use crate::data::json::JSONDataFile;
#[cfg(feature = "ugoira")]
use crate::data::video::get_video_metadata;
use crate::downloader::Downloader;
use crate::downloader::DownloaderResult;
use crate::downloader::LocalFile;
use crate::error::PixivDownloaderError;
use crate::concat_pixiv_downloader_error;
use crate::ext::try_err::TryErr;
use crate::gettext;
use crate::opthelper::get_helper;
use crate::pixiv_link::PixivID;
use crate::pixiv_web::PixivWebClient;
#[cfg(feature = "ugoira")]
use crate::ugoira::{UgoiraFrames, convert_ugoira_to_mp4};
use crate::utils::ask_need_overwrite;
use crate::utils::get_file_name_from_url;
use crate::webclient::WebClient;
use crate::Main;
use indicatif::MultiProgress;
use indicatif::ProgressStyle;
use json::JsonValue;
use reqwest::IntoUrl;
use spin_on::spin_on;
use std::path::PathBuf;
use std::sync::Arc;

impl Main {
    pub async fn download(&mut self) -> i32 {
        let pw = Arc::new(PixivWebClient::new());
        if !pw.init() {
            println!("{}", gettext("Failed to initialize pixiv web api client."));
            return 1;
        }
        if !pw.check_login().await {
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
                    let r = self.download_artwork(Arc::clone(&pw), id.clone()).await;
                    let r = if r.is_ok() {
                        0
                    } else {
                        println!("{} {}", gettext("Failed to download artwork:"), r.unwrap_err());
                        1
                    };
                    if r != 0 {
                        return r;
                    }
                }
            }
        }
        0
    }

    /// Download artwork link
    /// * `link` - Link
    /// * `np` - Number of page in artworks
    /// * `progress_bars` - Multiple progress bars
    /// * `datas` - The artwork's data
    /// * `base` - The directory of the target
    pub async fn download_artwork_link<L: IntoUrl + Clone>(link: L, np: u16, progress_bars: Option<Arc<MultiProgress>>, datas: Arc<PixivData>, base: Arc<PathBuf>) -> Result<(), PixivDownloaderError> {
        let file_name = get_file_name_from_url(link.clone()).try_err(format!("{} {}", gettext("Failed to get file name from url:"), link.as_str()))?;
        let file_name = base.join(file_name);
        let helper = get_helper();
        let downloader = Downloader::<LocalFile>::new(link, json::object!{"referer": "https://www.pixiv.net/"}, Some(&file_name), helper.overwrite())?;
        match downloader {
            DownloaderResult::Ok(d) => {
                if helper.use_progress_bar() {
                    let style = ProgressStyle::default_bar()
                        .template(helper.progress_bar_template().as_ref()).unwrap()
                        .progress_chars("#>-");
                    match progress_bars {
                        Some(b) => {
                            d.enable_progress_bar(style, Some(&b));
                        }
                        None => {
                            d.enable_progress_bar(style, None);
                        }
                    }
                    d.download();
                    d.join().await?;
                    if d.is_downloaded() {
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
                    }
                }
            }
            DownloaderResult::Canceled => {
                #[cfg(feature = "exif")]
                {
                    if helper.update_exif() && file_name.exists() {
                        if add_exifdata_to_image(&file_name, &datas, np).is_err() {
                            println!(
                                "{} {}",
                                gettext("Failed to add exif data to image:"),
                                file_name.to_str().unwrap_or("(null)")
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn download_artwork(&self, pw: Arc<PixivWebClient>, id: u64) -> Result<(), PixivDownloaderError> {
        let mut re = None;
        let pages;
        let mut ajax_ver = true;
        let helper = get_helper();
        if helper.use_webpage() {
            re = pw.get_artwork(id).await;
            if re.is_some() {
                ajax_ver = false;
            }
        }
        if re.is_none() {
            re = pw.get_artwork_ajax(id).await;
        }
        let re = re.try_err(gettext("Failed to get artwork's data."))?;
        if ajax_ver {
            pages = (&re["pageCount"]).as_u64();
        } else {
            pages = (&re["illust"][format!("{}", id).as_str()]["pageCount"]).as_u64();
        }
        let pages = pages.try_err(gettext("Failed to get page count."))?;
        let mut pages_data: Option<JsonValue> = None;
        if pages > 1 {
            pages_data = pw.get_illust_pages(id).await;
        }
        if pages > 1 && pages_data.is_none() {
            return Err(PixivDownloaderError::from(gettext("Failed to get pages' data.")));
        }
        let base = Arc::new(PathBuf::from("."));
        let json_file = base.join(format!("{}.json", id));
        let mut datas = PixivData::new(id).unwrap();
        if ajax_ver {
            datas.from_web_page_ajax_data(&re, true);
        } else {
            datas.from_web_page_data(&re, true);
        }
        let datas = Arc::new(datas);
        let json_data = JSONDataFile::from(Arc::clone(&datas));
        if !json_data.save(&json_file) {
            return Err(PixivDownloaderError::from(gettext("Failed to save metadata to JSON file.")));
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
                    let ugoira_data = pw.get_ugoira(id).await.try_err(gettext("Failed to get ugoira's data."))?;
                    let src = (&ugoira_data["originalSrc"]).as_str().try_err(gettext("Can not find source link for ugoira."))?;
                    let file_name = get_file_name_from_url(src).try_err(format!("{} {}", gettext("Failed to get file name from url:"), src))?;
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
                        let r = pw.download_image(src).await.try_err(format!("{} {}", gettext("Failed to download ugoira:"), src))?;
                        WebClient::download_stream(&file_name, r).try_err(format!("{} {}", gettext("Failed to download ugoira:"), src))?;
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
                        let frames =  UgoiraFrames::from_json(&ugoira_data["frames"])?;
                        let output_file_name = base.join(format!("{}.mp4", id));
                        convert_ugoira_to_mp4(&file_name, &output_file_name, &frames, 60f32, &options, &metadata)?;
                        println!("{}", gettext("Converted <src> -> <dest>").replace("<src>", file_name.to_str().unwrap_or("(null)")).replace("<dest>", output_file_name.to_str().unwrap_or("(null)")).as_str());
                    }
                    return Ok(());
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
            let mut re: Result<(), PixivDownloaderError> = Ok(());
            for page in pages_data.members() {
                let url = page["urls"]["original"].as_str();
                if url.is_none() {
                    concat_pixiv_downloader_error!(re, Err::<(), &str>(gettext("Failed to get original picture's link.")));
                    continue;
                }
                let f = tokio::spawn(Self::download_artwork_link(url.unwrap().to_owned(), np, Some(Arc::clone(&progress_bars)), Arc::clone(&datas), Arc::clone(&base)));
                tasks.push(f);
                np += 1;
            }
            for task in tasks {
                let r = spin_on(task);
                let r = match r {
                    Ok(r) => { r }
                    Err(e) => {
                        Err(PixivDownloaderError::from(e))
                    }
                };
                concat_pixiv_downloader_error!(re, r);
            }
            return re;
        }
        else if pages_data.is_some() {
            #[cfg(feature = "exif")]
            let mut np = 0u16;
            let pages_data = pages_data.as_ref().unwrap();
            for page in pages_data.members() {
                let link = page["urls"]["original"].as_str().try_err(gettext("Failed to get original picture's link."))?;
                let file_name = get_file_name_from_url(link).try_err(format!("{} {}", gettext("Failed to get file name from url:"), link))?;
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
                let r = pw.download_image(link).await.try_err(format!("{} {}", gettext("Failed to download image:"), link))?;
                WebClient::download_stream(&file_name, r).try_err(format!("{} {}", gettext("Failed to download image:"), link))?;
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
            }.try_err(gettext("Failed to get original picture's link."))?;
            let file_name = get_file_name_from_url(link).try_err(format!("{} {}", gettext("Failed to get file name from url:"), link))?;
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
                    return Ok(());
                }
            }
            let r = pw.download_image(link).await.try_err(format!("{} {}", gettext("Failed to download image:"), link))?;
            WebClient::download_stream(&file_name, r).try_err(format!("{} {}", gettext("Failed to download image:"), link))?;
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
        Ok(())
    }
}

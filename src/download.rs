#[cfg(feature = "avdict")]
use crate::avdict::AVDict;
use crate::concat_pixiv_downloader_error;
use crate::data::data::PixivData;
#[cfg(feature = "exif")]
use crate::data::exif::add_exifdata_to_image;
use crate::data::fanbox::FanboxData;
use crate::data::json::JSONDataFile;
#[cfg(feature = "ugoira")]
use crate::data::video::get_video_metadata;
use crate::data::video::get_video_metas;
#[cfg(feature = "db")]
use crate::db::open_and_init_database;
use crate::downloader::Downloader;
use crate::downloader::DownloaderHelper;
use crate::downloader::DownloaderResult;
use crate::downloader::LocalFile;
use crate::error::PixivDownloaderError;
use crate::ext::try_err::TryErr;
use crate::ext::try_err::TryErr4;
use crate::fanbox::article::block::FanboxArticleBlock;
use crate::fanbox::article::url_embed::FanboxArticleUrlEmbed;
use crate::fanbox::check::CheckUnknown;
use crate::fanbox::creator::FanboxCreator;
use crate::fanbox::creator::FanboxProfileItem;
use crate::fanbox::post::FanboxPost;
use crate::fanbox_api::FanboxClient;
use crate::gettext;
use crate::opthelper::get_helper;
use crate::pixiv_app::PixivAppClient;
use crate::pixiv_link::FanboxPostID;
use crate::pixiv_link::PixivID;
use crate::pixiv_web::PixivWebClient;
use crate::task_manager::get_progress_bar;
use crate::task_manager::TaskManager;
use crate::ugoira::convert_ugoira_to_mp4_subprocess;
#[cfg(feature = "ugoira")]
use crate::ugoira::{convert_ugoira_to_mp4, UgoiraFrames};
use crate::utils::get_file_name_from_url;
use crate::Main;
use indicatif::MultiProgress;
use json::JsonValue;
use proc_macros::print_error;
use wreq::IntoUrl;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

impl Main {
    pub async fn download(&mut self) -> i32 {
        let pw = Arc::new(PixivWebClient::new());
        let fc = Arc::new(FanboxClient::new());
        #[cfg(feature = "db")]
        let db = Arc::new(print_error!(
            gettext("Failed to open database:"),
            open_and_init_database(get_helper().db()).await,
            0
        ));
        #[cfg(not(feature = "db"))]
        let ac = PixivAppClient::new();
        #[cfg(feature = "db")]
        let ac = PixivAppClient::with_db(Some(db));
        let tasks = TaskManager::new_post();
        let download_multiple_posts = get_helper().download_multiple_posts();
        for id in self.cmd.as_ref().unwrap().ids.iter() {
            match id {
                PixivID::Artwork(id) => {
                    tasks
                        .add_task(download_artwork(ac.clone(), Arc::clone(&pw), id.clone()))
                        .await;
                    if !download_multiple_posts {
                        tasks.join().await;
                    }
                }
                PixivID::FanboxPost(id) => {
                    if !fc.is_inited() {
                        let helper = get_helper();
                        if !fc.init(helper.cookies()) {
                            log::error!("{}", gettext("Failed to initialize fanbox api client."));
                            return 1;
                        }
                        if !fc.check_login().await {
                            return 1;
                        }
                        if !fc.logined() {
                            log::warn!("{}", gettext("Warning: Fanbox client is not logged in."));
                        }
                    }
                    tasks
                        .add_task(download_fanbox_post(Arc::clone(&fc), id.clone()))
                        .await;
                    if !download_multiple_posts {
                        tasks.join().await;
                    }
                }
                PixivID::FanboxCreator(id) => {
                    if !fc.is_inited() {
                        let helper = get_helper();
                        if !fc.init(helper.cookies()) {
                            log::error!("{}", gettext("Failed to initialize fanbox api client."));
                            return 1;
                        }
                        if !fc.check_login().await {
                            return 1;
                        }
                        if !fc.logined() {
                            log::warn!("{}", gettext("Warning: Fanbox client is not logged in."));
                        }
                    }
                    tasks
                        .add_task(download_fanbox_creator_info(
                            Arc::clone(&fc),
                            id.to_owned(),
                            None,
                            None,
                        ))
                        .await;
                    if !download_multiple_posts {
                        tasks.join().await;
                    }
                }
            }
        }
        let mut re = 0;
        tasks.join().await;
        let tasks = tasks.take_finished_tasks();
        for task in tasks {
            let result = match task.await {
                Ok(result) => result,
                Err(e) => Err(PixivDownloaderError::from(e)),
            };
            match result {
                Ok(_) => {}
                Err(e) => {
                    log::error!("{} {}", gettext("Failed to download post:"), e);
                    re = 1;
                }
            }
        }
        re
    }

    pub async fn download_files(&mut self) -> i32 {
        let tasks = TaskManager::<Result<(), PixivDownloaderError>>::default();
        let helper = get_helper();
        let base = Arc::new(PathBuf::from(helper.download_base()));
        let enable_multi_progress_bar = helper.enable_multi_progress_bar();
        for url in self.cmd.as_ref().unwrap().urls.as_ref().unwrap().iter() {
            let url = url.clone();
            let base = Arc::clone(&base);
            tasks
                .add_task(async move {
                    let d = DownloaderHelper::builder(url)?.build();
                    download_file(
                        d,
                        if enable_multi_progress_bar {
                            Some(get_progress_bar())
                        } else {
                            None
                        },
                        base.clone(),
                    )
                    .await
                })
                .await;
        }
        let mut re = 0;
        tasks.join().await;
        let tasks = tasks.take_finished_tasks();
        for task in tasks {
            let result = match task.await {
                Ok(result) => result,
                Err(e) => Err(PixivDownloaderError::from(e)),
            };
            match result {
                Ok(_) => {}
                Err(e) => {
                    log::error!("{} {}", gettext("Failed to download url:"), e);
                    re = 1;
                }
            }
        }
        re
    }
}

/// Download artwork link
/// * `link` - Link
/// * `np` - Number of page in artworks
/// * `progress_bars` - Multiple progress bars
/// * `datas` - The artwork's data
/// * `base` - The directory of the target
pub async fn download_artwork_link<L: IntoUrl + Clone>(
    link: L,
    np: u16,
    progress_bars: Option<Arc<MultiProgress>>,
    datas: Arc<PixivData>,
    base: Arc<PathBuf>,
) -> Result<(), PixivDownloaderError> {
    let file_name = get_file_name_from_url(link.clone()).try_err(format!(
        "{} {}",
        gettext("Failed to get file name from url:"),
        link.as_str()
    ))?;
    let file_name = base.join(file_name);
    let helper = get_helper();
    let downloader = Downloader::<LocalFile>::new(
        link,
        json::object! {"referer": "https://www.pixiv.net/"},
        Some(&file_name),
        helper.overwrite(),
    )?;
    match downloader {
        DownloaderResult::Ok(d) => {
            d.handle_options(&helper, progress_bars);
            d.download();
            d.join().await?;
            if d.is_downloaded() {
                #[cfg(feature = "exif")]
                {
                    if add_exifdata_to_image(&file_name, &datas, np).is_err() {
                        log::warn!(
                            "{} {}",
                            gettext("Failed to add exif data to image:"),
                            file_name.to_str().unwrap_or("(null)")
                        );
                    }
                }
            } else if d.is_panic() {
                return Err(PixivDownloaderError::from(
                    d.get_panic()
                        .try_err(gettext("Failed to get error message."))?,
                ));
            }
        }
        DownloaderResult::Canceled => {
            #[cfg(feature = "exif")]
            {
                if helper.update_exif() && file_name.exists() {
                    if add_exifdata_to_image(&file_name, &datas, np).is_err() {
                        log::warn!(
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

pub async fn download_artwork(
    ac: PixivAppClient,
    pw: Arc<PixivWebClient>,
    id: u64,
) -> Result<(), PixivDownloaderError> {
    let helper = get_helper();
    let app_ok = helper.refresh_token().is_some();
    if app_ok && helper.use_app_api() {
        if let Err(e) = download_artwork_app(ac, pw.clone(), id).await {
            if e.is_not_found() {
                return Err(e);
            }
            log::warn!("{}{}", gettext("Warning: Failed to download artwork with app api, trying to download with web api: "), e);
            download_artwork_web(pw.clone(), id).await?;
        }
    } else if app_ok {
        if let Err(_) = download_artwork_web(pw.clone(), id).await {
            download_artwork_app(ac, pw.clone(), id).await?;
        }
    } else {
        download_artwork_web(pw, id).await?;
    }
    Ok(())
}

pub async fn download_artwork_ugoira(
    pw: Arc<PixivWebClient>,
    id: u64,
    base: Arc<PathBuf>,
    datas: Arc<PixivData>,
) -> Result<(), PixivDownloaderError> {
    let helper = get_helper();
    let ugoira_data = pw
        .get_ugoira(id)
        .await
        .try_err(gettext("Failed to get ugoira's data."))?;
    let src = (&ugoira_data["originalSrc"])
        .as_str()
        .try_err(gettext("Can not find source link for ugoira."))?;
    let dh = DownloaderHelper::builder(src)?
        .headers(json::object! { "referer": "https://www.pixiv.net/" })
        .build();
    let tasks = TaskManager::default();
    tasks
        .add_task(download_file(
            dh,
            if helper.enable_multi_progress_bar() {
                Some(get_progress_bar())
            } else {
                None
            },
            Arc::clone(&base),
        ))
        .await;
    tasks.join().await;
    let mut tasks = tasks.take_finished_tasks();
    let task = tasks.get_mut(0).try_err(gettext("No finished task."))?;
    task.await??;
    #[cfg(feature = "ugoira")]
    let use_cli = helper.ugoira_cli();
    #[cfg(not(feature = "ugoira"))]
    let use_cli = true;
    if use_cli {
        if let Some(ubase) = helper.ugoira() {
            let file_name = get_file_name_from_url(src).try_err(format!(
                "{} {}",
                gettext("Failed to get file name from url:"),
                src
            ))?;
            let file_name = base.join(file_name);
            let metadata = get_video_metas(&datas.clone());
            let frames_file_name = base.join(format!("{}_frames.json", id));
            std::fs::write(
                &frames_file_name,
                json::stringify((&ugoira_data["frames"]).clone()),
            )
            .try_err4(gettext("Failed to write frames info to file:"))?;
            let output_file_name = base.join(format!("{}.mp4", id));
            convert_ugoira_to_mp4_subprocess(
                &ubase,
                &file_name,
                &output_file_name,
                &frames_file_name,
                helper.ugoira_max_fps(),
                metadata,
                helper.force_yuv420p(),
                helper.x264_crf(),
                Some(helper.x264_profile()),
            )
            .await?;
            log::info!(
                "{}",
                gettext("Converted <src> -> <dest>")
                    .replace("<src>", file_name.to_str().unwrap_or("(null)"))
                    .replace("<dest>", output_file_name.to_str().unwrap_or("(null)"))
                    .as_str()
            );
            return Ok(());
        }
    }
    #[cfg(feature = "ugoira")]
    {
        let file_name = get_file_name_from_url(src).try_err(format!(
            "{} {}",
            gettext("Failed to get file name from url:"),
            src
        ))?;
        let file_name = base.join(file_name);
        let metadata = match get_video_metadata(Arc::clone(&datas).as_ref()) {
            Ok(m) => m,
            Err(e) => {
                log::warn!(
                    "{} {}",
                    gettext("Warning: Failed to generate video's metadata:"),
                    e
                );
                AVDict::new()
            }
        };
        let mut options = AVDict::new();
        if helper.force_yuv420p() {
            options.set("force_yuv420p", "1", None)?;
        }
        let profile = helper.x264_profile();
        if !profile.is_auto() {
            options.set("profile", profile.as_str(), None)?;
        }
        match helper.x264_crf() {
            Some(crf) => {
                options.set("crf", format!("{}", crf), None)?;
            }
            None => {}
        }
        let frames_file_name = base.join(format!("{}_frames.json", id));
        std::fs::write(
            &frames_file_name,
            json::stringify((&ugoira_data["frames"]).clone()),
        )
        .try_err4(gettext("Failed to write frames info to file:"))?;
        let frames = UgoiraFrames::from_json(&ugoira_data["frames"])?;
        let output_file_name = base.join(format!("{}.mp4", id));
        convert_ugoira_to_mp4(
            &file_name,
            &output_file_name,
            &frames,
            helper.ugoira_max_fps(),
            &options,
            &metadata,
        )?;
        log::info!(
            "{}",
            gettext("Converted <src> -> <dest>")
                .replace("<src>", file_name.to_str().unwrap_or("(null)"))
                .replace("<dest>", output_file_name.to_str().unwrap_or("(null)"))
                .as_str()
        );
    }
    return Ok(());
}

pub async fn download_artwork_app(
    ac: PixivAppClient,
    pw: Arc<PixivWebClient>,
    id: u64,
) -> Result<(), PixivDownloaderError> {
    let data = ac.get_illust_details(id).await?;
    let helper = get_helper();
    log::debug!("{:#?}", data);
    match crate::pixivapp::check::CheckUnknown::check_unknown(&data) {
        Ok(_) => {}
        Err(e) => {
            log::warn!(
                "{} {}",
                gettext("Warning: Post info contains unknown data:"),
                e
            );
        }
    }
    let base = Arc::new(PathBuf::from(helper.download_base()));
    let json_file = base.join(format!("{}.json", id));
    let mut datas = PixivData::new(id).unwrap();
    datas.from_app_illust(&data);
    let mut web_used = false;
    if data.caption_is_empty() && helper.use_web_description() {
        if let Some(data) = pw.get_artwork_ajax(id).await {
            web_used = true;
            if let Some(desc) = data["description"]
                .as_str()
                .or_else(|| data["illustComment"].as_str())
            {
                datas.description = Some(desc.to_owned());
            }
        }
    }
    if helper.add_history() && !web_used {
        if let Err(e) = ac.add_illust_to_browsing_history(vec![id]).await {
            log::warn!(
                "{} {}",
                gettext("Warning: Failed to add artwork to history:"),
                e
            );
        }
    }
    let datas = Arc::new(datas);
    let json_data = JSONDataFile::from(Arc::clone(&datas));
    if !json_data.save(&json_file) {
        return Err(PixivDownloaderError::from(gettext(
            "Failed to save metadata to JSON file.",
        )));
    }
    let illust_type = data.typ();
    match illust_type {
        Some(illust_type) => match illust_type {
            "ugoira" => {
                return download_artwork_ugoira(pw, id, base, datas).await;
            }
            _ => {}
        },
        None => {
            log::warn!("{}", gettext("Warning: Failed to get illust's type."));
        }
    }
    let page_count = data
        .page_count()
        .ok_or(gettext("Failed to get page count."))?;
    if page_count > 1 && helper.download_multiple_files() {
        let mut np = 0u16;
        let tasks = TaskManager::default();
        let mut re: Result<(), PixivDownloaderError> = Ok(());
        for page in data.meta_pages() {
            let url = match page.original() {
                Some(url) => url.to_owned(),
                None => {
                    concat_pixiv_downloader_error!(
                        re,
                        Err::<(), &str>(gettext("Failed to get original picture's link."))
                    );
                    continue;
                }
            };
            tasks
                .add_task(download_artwork_link(
                    url,
                    np,
                    if helper.enable_multi_progress_bar() {
                        Some(get_progress_bar())
                    } else {
                        None
                    },
                    Arc::clone(&datas),
                    Arc::clone(&base),
                ))
                .await;
            np += 1;
        }
        tasks.join().await;
        let tasks = tasks.take_finished_tasks();
        for task in tasks {
            let r = task.await;
            let r = match r {
                Ok(r) => r,
                Err(e) => Err(PixivDownloaderError::from(e)),
            };
            concat_pixiv_downloader_error!(re, r);
        }
        return re;
    } else if page_count > 1 {
        let mut np = 0u16;
        let tasks = TaskManager::default();
        for page in data.meta_pages() {
            let link = page
                .original()
                .ok_or(gettext("Failed to get original picture's link."))?;
            tasks
                .add_task(download_artwork_link(
                    link.to_owned(),
                    np,
                    if helper.enable_multi_progress_bar() {
                        Some(get_progress_bar())
                    } else {
                        None
                    },
                    Arc::clone(&datas),
                    Arc::clone(&base),
                ))
                .await;
            tasks.join().await;
            np += 1;
        }
        let mut re = Ok(());
        let tasks = tasks.take_finished_tasks();
        for task in tasks {
            let r = task.await;
            let r = match r {
                Ok(r) => r,
                Err(e) => Err(PixivDownloaderError::from(e)),
            };
            concat_pixiv_downloader_error!(re, r);
        }
        return re;
    } else {
        let link = data
            .original_image_url()
            .ok_or(gettext("Failed to get original picture's link."))?;
        let tasks = TaskManager::default();
        tasks
            .add_task(download_artwork_link(
                link.to_owned(),
                0,
                if helper.enable_multi_progress_bar() {
                    Some(get_progress_bar())
                } else {
                    None
                },
                Arc::clone(&datas),
                Arc::clone(&base),
            ))
            .await;
        tasks.join().await;
        let mut tasks = tasks.take_finished_tasks();
        let task = tasks.get_mut(0).try_err(gettext("No tasks finished."))?;
        task.await??;
    }
    Ok(())
}

pub async fn download_artwork_web(
    pw: Arc<PixivWebClient>,
    id: u64,
) -> Result<(), PixivDownloaderError> {
    if !pw.is_login_checked() {
        if !pw.check_login().await {
            log::error!("{}", gettext("Failed to check login status."));
        } else {
            if !pw.logined() {
                log::warn!(
                    "{}",
                    gettext("Warning: Web api client not logged in, some future may not work.")
                );
            }
        }
    }
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
        return Err(PixivDownloaderError::from(gettext(
            "Failed to get pages' data.",
        )));
    }
    let base = Arc::new(PathBuf::from(helper.download_base()));
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
        return Err(PixivDownloaderError::from(gettext(
            "Failed to save metadata to JSON file.",
        )));
    }
    let illust_type = if ajax_ver {
        (&re["illustType"]).as_i64()
    } else {
        (&re["illust"][format!("{}", id).as_str()]["illustType"]).as_i64()
    };
    if illust_type.is_some() {
        let illust_type = illust_type.unwrap();
        match illust_type {
            0 => {} // Normal illust
            1 => {} // Manga illust
            2 => {
                return download_artwork_ugoira(pw, id, base, datas).await;
            }
            _ => {
                log::warn!(
                    "{} {}",
                    gettext("Warning: Unknown illust type:"),
                    illust_type
                )
            }
        }
    } else {
        log::warn!("{}", gettext("Warning: Failed to get illust's type."));
    }
    if pages_data.is_some() && helper.download_multiple_files() {
        let mut np = 0u16;
        let pages_data = pages_data.as_ref().unwrap();
        let tasks = TaskManager::default();
        let mut re: Result<(), PixivDownloaderError> = Ok(());
        for page in pages_data.members() {
            let url = page["urls"]["original"].as_str();
            if url.is_none() {
                concat_pixiv_downloader_error!(
                    re,
                    Err::<(), &str>(gettext("Failed to get original picture's link."))
                );
                continue;
            }
            tasks
                .add_task(download_artwork_link(
                    url.unwrap().to_owned(),
                    np,
                    if helper.enable_multi_progress_bar() {
                        Some(get_progress_bar())
                    } else {
                        None
                    },
                    Arc::clone(&datas),
                    Arc::clone(&base),
                ))
                .await;
            np += 1;
        }
        tasks.join().await;
        let tasks = tasks.take_finished_tasks();
        for task in tasks {
            let r = task.await;
            let r = match r {
                Ok(r) => r,
                Err(e) => Err(PixivDownloaderError::from(e)),
            };
            concat_pixiv_downloader_error!(re, r);
        }
        return re;
    } else if pages_data.is_some() {
        let mut np = 0u16;
        let pages_data = pages_data.as_ref().unwrap();
        let tasks = TaskManager::default();
        for page in pages_data.members() {
            let link = page["urls"]["original"]
                .as_str()
                .try_err(gettext("Failed to get original picture's link."))?;
            tasks
                .add_task(download_artwork_link(
                    link.to_owned(),
                    np,
                    if helper.enable_multi_progress_bar() {
                        Some(get_progress_bar())
                    } else {
                        None
                    },
                    Arc::clone(&datas),
                    Arc::clone(&base),
                ))
                .await;
            tasks.join().await;
            np += 1;
        }
        let mut re = Ok(());
        let tasks = tasks.take_finished_tasks();
        for task in tasks {
            let r = task.await;
            let r = match r {
                Ok(r) => r,
                Err(e) => Err(PixivDownloaderError::from(e)),
            };
            concat_pixiv_downloader_error!(re, r);
        }
        return re;
    } else {
        let link = if ajax_ver {
            (&re["urls"]["original"]).as_str()
        } else {
            (&re["illust"][format!("{}", id)]["urls"]["original"]).as_str()
        }
        .try_err(gettext("Failed to get original picture's link."))?;
        let tasks = TaskManager::default();
        tasks
            .add_task(download_artwork_link(
                link.to_owned(),
                0,
                if helper.enable_multi_progress_bar() {
                    Some(get_progress_bar())
                } else {
                    None
                },
                Arc::clone(&datas),
                Arc::clone(&base),
            ))
            .await;
        tasks.join().await;
        let mut tasks = tasks.take_finished_tasks();
        let task = tasks.get_mut(0).try_err(gettext("No tasks finished."))?;
        task.await??;
    }
    Ok(())
}

/// Download a  file link
/// * `dh` - Link and other informations
/// * `progress_bars` - Multiple progress bars
/// * `base` - The directory of the target
pub async fn download_file(
    dh: DownloaderHelper,
    progress_bars: Option<Arc<MultiProgress>>,
    base: Arc<PathBuf>,
) -> Result<(), PixivDownloaderError> {
    let helper = get_helper();
    match dh.download_local(helper.overwrite(), &*base)? {
        DownloaderResult::Ok(d) => {
            d.handle_options(&helper, progress_bars);
            d.download();
            d.join().await?;
            if d.is_panic() {
                return Err(PixivDownloaderError::from(
                    d.get_panic()
                        .try_err(gettext("Failed to get error message."))?,
                ));
            }
        }
        DownloaderResult::Canceled => {}
    }
    Ok(())
}

/// Download a fanbox image link
/// * `dh` - Link and other informations
/// * `np` - Number of page
/// * `progress_bars` - Multiple progress bars
/// * `datas` - The artwork's data
/// * `base` - The directory of the target
/// * `total_page` - The total count of pages
pub async fn download_fanbox_image(
    dh: DownloaderHelper,
    np: u16,
    progress_bars: Option<Arc<MultiProgress>>,
    datas: Arc<FanboxData>,
    base: Arc<PathBuf>,
    total_page: u16,
) -> Result<(), PixivDownloaderError> {
    let mut ndh = dh.clone();
    let helper = get_helper();
    if helper.fanbox_page_number() {
        let len = format!("{}", total_page).len();
        let basep = match &datas.id {
            PixivID::Artwork(a) => format!("{}", a),
            PixivID::FanboxCreator(f) => format!("{}", f),
            PixivID::FanboxPost(p) => format!("{}", p.post_id),
        };
        let mut nps = format!("{}", np + 1);
        while nps.len() < len {
            nps = String::from("0") + &nps;
        }
        let ofn = ndh
            .get_local_file_path(&*base)
            .try_err(gettext("Failed to get file name from url."))?;
        let ext = ofn
            .extension()
            .map_or("jpg", |v| v.to_str().unwrap_or("jpg"));
        ndh.set_file_name(&format!("{}_{}.{}", basep, nps, ext));
    }
    let file_name = ndh
        .get_local_file_path(&*base)
        .try_err(gettext("Failed to get file name from url."))?;
    match ndh.download_local(helper.overwrite(), &*base)? {
        DownloaderResult::Ok(d) => {
            d.handle_options(&helper, progress_bars);
            d.download();
            d.join().await?;
            if d.is_downloaded() {
                #[cfg(feature = "exif")]
                {
                    if add_exifdata_to_image(&file_name, &datas, np).is_err() {
                        log::warn!(
                            "{} {}",
                            gettext("Failed to add exif data to image:"),
                            file_name.to_str().unwrap_or("(null)")
                        );
                    }
                }
            } else if d.is_panic() {
                return Err(PixivDownloaderError::from(
                    d.get_panic()
                        .try_err(gettext("Failed to get error message."))?,
                ));
            }
        }
        DownloaderResult::Canceled => {
            #[cfg(feature = "exif")]
            {
                if helper.update_exif() && file_name.exists() {
                    if add_exifdata_to_image(&file_name, &datas, np).is_err() {
                        log::warn!(
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

pub async fn download_fanbox_post(
    fc: Arc<FanboxClient>,
    id: FanboxPostID,
) -> Result<(), PixivDownloaderError> {
    let post = fc
        .get_post_info(id.post_id)
        .await
        .try_err(gettext("Failed to get post info."))?;
    let helper = get_helper();
    log::debug!("{:#?}", post);
    match post.check_unknown() {
        Ok(_) => {}
        Err(e) => {
            log::warn!(
                "{} {}",
                gettext("Warning: Post info contains unknown data:"),
                e
            );
        }
    }
    if post
        .is_restricted()
        .try_err(gettext("Failed to check the post is restricted or not."))?
    {
        log::warn!("{}", gettext("Warning: This article is restricted."));
        // #TODO allow to continue
        return Ok(());
    }
    let base = Arc::new(
        PathBuf::from(helper.download_base())
            .join(&id.creator_id)
            .join(format!("{}", id.post_id)),
    );
    let json_file = base.join("data.json");
    let data = FanboxData::new(id, &post).try_err("Failed to create data file.")?;
    let data_file = JSONDataFile::from(&data);
    if !base.exists() {
        match create_dir_all(&*base) {
            Ok(_) => {}
            Err(e) => {
                if !base.exists() {
                    return Err(PixivDownloaderError::from(e));
                }
            }
        }
    }
    data_file
        .save(&json_file)
        .try_err(gettext("Failed to save post data to file."))?;
    let tasks = TaskManager::default();
    let ptasks = TaskManager::new_post();
    let mut re = Ok(());
    let download_multiple_files = helper.download_multiple_files();
    match post {
        FanboxPost::Article(article) => {
            let article = Arc::new(article);
            let body = article.body();
            let image_map = body
                .image_map()
                .try_err(gettext("Failed to get image map from article."))?;
            let blocks = body
                .blocks()
                .try_err(gettext("Failed to get blocks from article."))?;
            let url_embed_map = body
                .url_embed_map()
                .try_err(gettext("Failed to get embed url map from article."))?;
            let file_map = body
                .file_map()
                .ok_or(gettext("Failed to get file map from article."))?;
            let mut np = 0;
            let total_page = image_map.len() as u16;
            let mut datas = data.clone();
            #[cfg(feature = "exif")]
            datas.exif_data.replace(Box::new(Arc::clone(&article)));
            let datas = Arc::new(datas);
            for i in blocks {
                match i {
                    FanboxArticleBlock::Image(img) => {
                        let img = image_map
                            .get_image(
                                img.image_id()
                                    .try_err(gettext("Failed to get image id from block."))?,
                            )
                            .try_err(gettext("Failed get image information from image map."))?;
                        let dh = img
                            .download_original_url()?
                            .try_err(gettext("Can not get original url for image"))?;
                        tasks
                            .add_task(download_fanbox_image(
                                dh,
                                np,
                                if helper.enable_multi_progress_bar() {
                                    Some(get_progress_bar())
                                } else {
                                    None
                                },
                                Arc::clone(&datas),
                                Arc::clone(&base),
                                total_page,
                            ))
                            .await;
                        if !download_multiple_files {
                            tasks.join().await;
                        }
                        np += 1;
                    }
                    FanboxArticleBlock::UrlEmbed(u) => {
                        let embed_url = url_embed_map
                            .get_url_embed(
                                u.url_embed_id()
                                    .try_err(gettext("Failed to get embed url id from block"))?,
                            )
                            .try_err(gettext("Failed to get embed url from url embed map."))?;
                        match embed_url {
                            FanboxArticleUrlEmbed::FanboxCreator(creator) => {
                                let profile = creator
                                    .profile()
                                    .try_err(gettext("Failed to get creator's profile."))?;
                                let id = profile
                                    .creator_id()
                                    .try_err("Failed to get creator's id.")?;
                                match ptasks
                                    .add_task_else_run_local(download_fanbox_creator_info(
                                        Arc::clone(&fc),
                                        id.to_owned(),
                                        Some(profile),
                                        None,
                                    ))
                                    .await
                                {
                                    Some(r) => {
                                        concat_pixiv_downloader_error!(re, r);
                                    }
                                    None => {
                                        if !download_multiple_files {
                                            ptasks.join().await;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    FanboxArticleBlock::File(f) => {
                        let file = file_map
                            .get_file(
                                f.file_id()
                                    .try_err(gettext("Failed to get file id from block."))?,
                            )
                            .try_err(gettext("Failed to get file from file map."))?;
                        let dh = file
                            .download_url()?
                            .ok_or(gettext("Failed to get download url from file information."))?;
                        tasks
                            .add_task(download_file(
                                dh,
                                if helper.enable_multi_progress_bar() {
                                    Some(get_progress_bar())
                                } else {
                                    None
                                },
                                Arc::clone(&base),
                            ))
                            .await;
                        if !download_multiple_files {
                            tasks.join().await;
                        }
                    }
                    _ => {}
                }
            }
        }
        FanboxPost::File(file) => {
            let body = file
                .body()
                .try_err(gettext("Failed to get the body of file post."))?;
            let files = body
                .files()
                .try_err(gettext("Failed to get files from file post."))?;
            for f in files.iter() {
                let dh = f
                    .download_url()?
                    .try_err(gettext("Failed to get url of the file."))?;
                tasks
                    .add_task(download_file(
                        dh,
                        if helper.enable_multi_progress_bar() {
                            Some(get_progress_bar())
                        } else {
                            None
                        },
                        Arc::clone(&base),
                    ))
                    .await;
                if !download_multiple_files {
                    tasks.join().await;
                }
            }
        }
        FanboxPost::Image(img) => {
            let img = Arc::new(img);
            let body = img
                .body()
                .try_err(gettext("Failed to get the body of image post."))?;
            let images = body
                .images()
                .try_err(gettext("Failed to get images from the image post."))?;
            let mut np = 0;
            let total_page = images.len() as u16;
            let mut datas = data.clone();
            #[cfg(feature = "exif")]
            datas.exif_data.replace(Box::new(Arc::clone(&img)));
            let datas = Arc::new(datas);
            for img in images.iter() {
                let dh = img
                    .download_original_url()?
                    .try_err(gettext("Can not get original url for image"))?;
                tasks
                    .add_task(download_fanbox_image(
                        dh,
                        np,
                        if helper.enable_multi_progress_bar() {
                            Some(get_progress_bar())
                        } else {
                            None
                        },
                        Arc::clone(&datas),
                        Arc::clone(&base),
                        total_page,
                    ))
                    .await;
                if !download_multiple_files {
                    tasks.join().await;
                }
                np += 1;
            }
        }
        FanboxPost::Text(t) => {
            let text = t
                .text()
                .ok_or(gettext("Failed to get text from text post."))?;
            let text_file = base.join("data.txt");
            let mut f = File::create(&text_file)?;
            f.write_all(text.as_bytes())?;
        }
        FanboxPost::Unknown(_) => {
            return Err(PixivDownloaderError::from(gettext(
                "Unrecognized post type.",
            )));
        }
    }
    tasks.join().await;
    let tasks = tasks.take_finished_tasks();
    for task in tasks {
        let r = task.await;
        let r = match r {
            Ok(r) => r,
            Err(e) => Err(PixivDownloaderError::from(e)),
        };
        concat_pixiv_downloader_error!(re, r);
    }
    ptasks.join().await;
    let ptasks = ptasks.take_finished_tasks();
    for task in ptasks {
        let r = task.await;
        let r = match r {
            Ok(r) => r,
            Err(e) => Err(PixivDownloaderError::from(e)),
        };
        concat_pixiv_downloader_error!(re, r);
    }
    re
}

pub async fn download_fanbox_creator_info(
    fc: Arc<FanboxClient>,
    id: String,
    data: Option<FanboxCreator>,
    base: Option<PathBuf>,
) -> Result<(), PixivDownloaderError> {
    let data = match data {
        Some(data) => {
            let cid = data
                .creator_id()
                .try_err(gettext("Failed to get creator's id."))?;
            if id == cid {
                Some(data)
            } else {
                None
            }
        }
        None => None,
    };
    let data = match data {
        Some(data) => data,
        None => fc
            .get_creator(&id)
            .await
            .try_err(gettext("Failed to get creator's information."))?,
    };
    let data = Arc::new(data);
    let helper = get_helper();
    log::debug!("{:#?}", data);
    match data.check_unknown() {
        Ok(_) => {}
        Err(e) => {
            log::warn!(
                "{} {}",
                gettext("Warning: Creator's info contains unknown data:"),
                e
            );
            return Ok(());
        }
    }
    let mut fdata = FanboxData::new(PixivID::FanboxCreator(id.clone()), &*data)
        .try_err("Failed to create data file.")?;
    let base = match base {
        Some(base) => Arc::new(base),
        None => Arc::new(PathBuf::from(".").join(&id)),
    };
    let json_file = base.join("creator.json");
    let data_file = JSONDataFile::from(&fdata);
    if !base.exists() {
        match create_dir_all(&*base) {
            Ok(_) => {}
            Err(e) => {
                if !base.exists() {
                    return Err(PixivDownloaderError::from(e));
                }
            }
        }
    }
    data_file
        .save(&json_file)
        .try_err(gettext("Failed to save post data to file."))?;
    let tasks = TaskManager::default();
    #[cfg(feature = "exif")]
    fdata.exif_data.replace(Box::new(Arc::clone(&data)));
    let fdata = Arc::new(fdata);
    let download_multiple_files = helper.download_multiple_files();
    let mut np = 0u16;
    let mut total_page = 0;
    match data.download_cover_image_url()? {
        Some(_) => {
            total_page += 1;
        }
        None => {}
    }
    total_page += data.profile_items()?.len() as u16;
    {
        match data.download_cover_image_url()? {
            Some(dh) => {
                tasks
                    .add_task(download_fanbox_image(
                        dh,
                        np,
                        if helper.enable_multi_progress_bar() {
                            Some(get_progress_bar())
                        } else {
                            None
                        },
                        Arc::clone(&fdata),
                        Arc::clone(&base),
                        total_page,
                    ))
                    .await;
                if !download_multiple_files {
                    tasks.join().await;
                }
                np += 1;
            }
            None => {}
        }
    }
    for i in data.profile_items()?.deref() {
        match i {
            FanboxProfileItem::Image(img) => {
                let dh = img
                    .download_image_url()?
                    .try_err(gettext("Can not get image url."))?;
                tasks
                    .add_task(download_fanbox_image(
                        dh,
                        np,
                        if helper.enable_multi_progress_bar() {
                            Some(get_progress_bar())
                        } else {
                            None
                        },
                        Arc::clone(&fdata),
                        Arc::clone(&base),
                        total_page,
                    ))
                    .await;
                if !download_multiple_files {
                    tasks.join().await;
                }
                np += 1;
            }
            FanboxProfileItem::Unknown(_) => {
                return Err(PixivDownloaderError::from(gettext(
                    "Unrecognized profile item type.",
                )));
            }
        }
    }
    tasks.join().await;
    let mut re = Ok(());
    let tasks = tasks.take_finished_tasks();
    for task in tasks {
        let r = task.await;
        let r = match r {
            Ok(r) => r,
            Err(e) => Err(PixivDownloaderError::from(e)),
        };
        concat_pixiv_downloader_error!(re, r);
    }
    re
}

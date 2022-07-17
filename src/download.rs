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
use crate::downloader::Downloader;
use crate::downloader::DownloaderHelper;
use crate::downloader::DownloaderResult;
use crate::downloader::LocalFile;
use crate::error::PixivDownloaderError;
use crate::ext::any::AsAny;
use crate::ext::try_err::TryErr;
use crate::fanbox::article::block::FanboxArticleBlock;
use crate::fanbox::check::CheckUnknown;
use crate::fanbox::post::FanboxPost;
use crate::fanbox_api::FanboxClient;
use crate::gettext;
use crate::opthelper::get_helper;
use crate::pixiv_link::FanboxPostID;
use crate::pixiv_link::PixivID;
use crate::pixiv_web::PixivWebClient;
use crate::task_manager::get_progress_bar;
use crate::task_manager::TaskManager;
#[cfg(feature = "ugoira")]
use crate::ugoira::{convert_ugoira_to_mp4, UgoiraFrames};
use crate::utils::get_file_name_from_url;
use crate::Main;
use indicatif::MultiProgress;
use json::JsonValue;
use reqwest::IntoUrl;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinHandle;

impl Main {
    pub async fn download(&mut self) -> i32 {
        let pw = Arc::new(PixivWebClient::new());
        let fc = Arc::new(FanboxClient::new());
        for id in self.cmd.as_ref().unwrap().ids.iter() {
            match id {
                PixivID::Artwork(id) => {
                    if !pw.is_inited() {
                        if !pw.init() {
                            println!("{}", gettext("Failed to initialize pixiv web api client."));
                            return 1;
                        }
                        if !pw.check_login().await {
                            return 1;
                        }
                        if !pw.logined() {
                            println!("{}", gettext("Warning: Web api client not logined, some future may not work."));
                        }
                    }
                    let r = self.download_artwork(Arc::clone(&pw), id.clone()).await;
                    let r = if r.is_ok() {
                        0
                    } else {
                        println!(
                            "{} {}",
                            gettext("Failed to download artwork:"),
                            r.unwrap_err()
                        );
                        1
                    };
                    if r != 0 {
                        return r;
                    }
                }
                PixivID::FanboxPost(id) => {
                    if !fc.is_inited() {
                        let helper = get_helper();
                        if !fc.init(helper.cookies()) {
                            println!("{}", gettext("Failed to initialize fanbox api client."));
                            return 1;
                        }
                        if !fc.check_login().await {
                            return 1;
                        }
                        if !fc.logined() {
                            println!("{}", gettext("Warning: Fanbox client is not logined."));
                        }
                        let r = self.download_fanbox_post(Arc::clone(&fc), id.clone()).await;
                        let r = match r {
                            Ok(_) => 0,
                            Err(e) => {
                                println!("{} {}", gettext("Failed to download post:"), e);
                                1
                            }
                        };
                        if r != 0 {
                            return r;
                        }
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
                            println!(
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

    pub async fn download_artwork(
        &self,
        pw: Arc<PixivWebClient>,
        id: u64,
    ) -> Result<(), PixivDownloaderError> {
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
                        .add_task(Self::download_file(
                            dh,
                            Some(get_progress_bar()),
                            Arc::clone(&base),
                        ))
                        .await;
                    tasks.join().await;
                    let mut tasks = tasks.take_finished_tasks();
                    let task = tasks.get_mut(0).try_err(gettext("No finished task."))?;
                    let task = task.as_any_mut();
                    let task = task
                        .downcast_mut::<JoinHandle<Result<(), PixivDownloaderError>>>()
                        .try_err("Failed to downcast task.")?;
                    task.await??;
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
                                println!(
                                    "{} {}",
                                    gettext("Warning: Failed to generate video's metadata:"),
                                    e
                                );
                                AVDict::new()
                            }
                        };
                        let options = AVDict::new();
                        let frames = UgoiraFrames::from_json(&ugoira_data["frames"])?;
                        let output_file_name = base.join(format!("{}.mp4", id));
                        convert_ugoira_to_mp4(
                            &file_name,
                            &output_file_name,
                            &frames,
                            60f32,
                            &options,
                            &metadata,
                        )?;
                        println!(
                            "{}",
                            gettext("Converted <src> -> <dest>")
                                .replace("<src>", file_name.to_str().unwrap_or("(null)"))
                                .replace("<dest>", output_file_name.to_str().unwrap_or("(null)"))
                                .as_str()
                        );
                    }
                    return Ok(());
                }
                _ => {
                    println!(
                        "{} {}",
                        gettext("Warning: Unknown illust type:"),
                        illust_type
                    )
                }
            }
        } else {
            println!("{}", gettext("Warning: Failed to get illust's type."));
        }
        if pages_data.is_some() && helper.download_multiple_files() {
            let mut np = 0u16;
            let pages_data = pages_data.as_ref().unwrap();
            let progress_bars = get_progress_bar();
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
                    .add_task(Self::download_artwork_link(
                        url.unwrap().to_owned(),
                        np,
                        Some(Arc::clone(&progress_bars)),
                        Arc::clone(&datas),
                        Arc::clone(&base),
                    ))
                    .await;
                np += 1;
            }
            tasks.join().await;
            let tasks = tasks.take_finished_tasks();
            for mut task in tasks {
                let t = task.as_any_mut();
                if let Some(task) = t.downcast_mut::<JoinHandle<Result<(), PixivDownloaderError>>>()
                {
                    let r = task.await;
                    let r = match r {
                        Ok(r) => r,
                        Err(e) => Err(PixivDownloaderError::from(e)),
                    };
                    concat_pixiv_downloader_error!(re, r);
                }
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
                    .add_task(Self::download_artwork_link(
                        link.to_owned(),
                        np,
                        Some(get_progress_bar()),
                        Arc::clone(&datas),
                        Arc::clone(&base),
                    ))
                    .await;
                tasks.join().await;
                np += 1;
            }
            let mut re = Ok(());
            let tasks = tasks.take_finished_tasks();
            for mut task in tasks {
                let t = task.as_any_mut();
                if let Some(task) = t.downcast_mut::<JoinHandle<Result<(), PixivDownloaderError>>>()
                {
                    let r = task.await;
                    let r = match r {
                        Ok(r) => r,
                        Err(e) => Err(PixivDownloaderError::from(e)),
                    };
                    concat_pixiv_downloader_error!(re, r);
                }
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
                .add_task(Self::download_artwork_link(
                    link.to_owned(),
                    0,
                    Some(get_progress_bar()),
                    Arc::clone(&datas),
                    Arc::clone(&base),
                ))
                .await;
            tasks.join().await;
            let mut tasks = tasks.take_finished_tasks();
            let task = tasks.get_mut(0).try_err(gettext("No tasks finished."))?;
            let task = task.as_any_mut();
            let task = task
                .downcast_mut::<JoinHandle<Result<(), PixivDownloaderError>>>()
                .try_err("Failed to downcast the result.")?;
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
    pub async fn download_fanbox_image(
        dh: DownloaderHelper,
        np: u16,
        progress_bars: Option<Arc<MultiProgress>>,
        datas: Arc<FanboxData>,
        base: Arc<PathBuf>,
    ) -> Result<(), PixivDownloaderError> {
        let helper = get_helper();
        let file_name = dh
            .get_local_file_path(&*base)
            .try_err(gettext("Failed to get file name from url."))?;
        match dh.download_local(helper.overwrite(), &*base)? {
            DownloaderResult::Ok(d) => {
                d.handle_options(&helper, progress_bars);
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

    pub async fn download_fanbox_post(
        &self,
        fc: Arc<FanboxClient>,
        id: FanboxPostID,
    ) -> Result<(), PixivDownloaderError> {
        let post = fc
            .get_post_info(id.post_id)
            .await
            .try_err(gettext("Failed to get post info."))?;
        let helper = get_helper();
        if helper.verbose() {
            println!("{:#?}", post);
        }
        match post.check_unknown() {
            Ok(_) => {}
            Err(e) => {
                println!(
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
            println!("{}", gettext("Warning: This article is restricted."));
            // #TODO allow to continue
            return Ok(());
        }
        let base = Arc::new(PathBuf::from(format!("./{}/{}", id.creator_id, id.post_id)));
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
                let mut np = 0;
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
                                .add_task(Self::download_fanbox_image(
                                    dh,
                                    np,
                                    Some(get_progress_bar()),
                                    Arc::clone(&datas),
                                    Arc::clone(&base),
                                ))
                                .await;
                            if !download_multiple_files {
                                tasks.join().await;
                            }
                            np += 1;
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
                        .add_task(Self::download_file(
                            dh,
                            Some(get_progress_bar()),
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
                let mut datas = data.clone();
                #[cfg(feature = "exif")]
                datas.exif_data.replace(Box::new(Arc::clone(&img)));
                let datas = Arc::new(datas);
                for img in images.iter() {
                    let dh = img
                        .download_original_url()?
                        .try_err(gettext("Can not get original url for image"))?;
                    tasks
                        .add_task(Self::download_fanbox_image(
                            dh,
                            np,
                            Some(get_progress_bar()),
                            Arc::clone(&datas),
                            Arc::clone(&base),
                        ))
                        .await;
                    if !download_multiple_files {
                        tasks.join().await;
                    }
                    np += 1;
                }
            }
            FanboxPost::Unknown(_) => {
                return Err(PixivDownloaderError::from(gettext(
                    "Unrecognized post type.",
                )));
            }
        }
        tasks.join().await;
        let mut re = Ok(());
        let tasks = tasks.take_finished_tasks();
        for mut task in tasks {
            let task = task.as_any_mut();
            if let Some(task) = task.downcast_mut::<JoinHandle<Result<(), PixivDownloaderError>>>()
            {
                let r = task.await;
                let r = match r {
                    Ok(r) => r,
                    Err(e) => Err(PixivDownloaderError::from(e)),
                };
                concat_pixiv_downloader_error!(re, r);
            }
        }
        re
    }
}

use crate::error::PixivDownloaderError;
use crate::ext::subprocess::PopenAsyncExt;
use crate::ext::try_err::TryErr4;
use crate::get_helper;
use std::{ffi::OsStr, io::Read};
use subprocess::{ExitStatus, Popen, PopenConfig, Redirection};

pub const MAX_PHOTO_SIZE: u64 = 10485760;

pub async fn check_ffprobe<S: AsRef<str> + ?Sized>(path: &S) -> Result<bool, PixivDownloaderError> {
    let mut p = Popen::create(
        &[path.as_ref(), "-h"],
        PopenConfig {
            stdin: Redirection::None,
            stdout: Redirection::Pipe,
            stderr: Redirection::Pipe,
            ..PopenConfig::default()
        },
    )
    .try_err4("Failed to create popen: ")?;
    p.communicate(None)?;
    let re = p.async_wait().await;
    Ok(match re {
        ExitStatus::Exited(o) => o == 0,
        _ => false,
    })
}

pub struct SupportedImage {
    pub supported: bool,
    pub size_too_big: bool,
}

impl SupportedImage {
    pub fn new(supported: bool, size_too_big: bool) -> Self {
        Self {
            supported,
            size_too_big,
        }
    }
}

pub async fn get_image_size<S: AsRef<OsStr> + ?Sized, P: AsRef<OsStr> + ?Sized>(
    ffprobe: &S,
    file: &P,
) -> Result<(i64, i64), PixivDownloaderError> {
    let argv = [
        ffprobe.as_ref().to_owned(),
        "-v".into(),
        "error".into(),
        "-select_streams".into(),
        "v:0".into(),
        "-show_entries".into(),
        "stream=width,height".into(),
        "-of".into(),
        "csv=s=x:p=0".into(),
        file.as_ref().to_owned(),
    ];
    let mut p = Popen::create(
        &argv,
        PopenConfig {
            stdin: Redirection::None,
            stdout: Redirection::Pipe,
            stderr: Redirection::None,
            ..PopenConfig::default()
        },
    )
    .try_err4("Failed to create popen: ")?;
    let re = p.async_wait().await;
    if !re.success() {
        log::error!(target: "telegram_image", "Failed to get image size for {}: {:?}.", file.as_ref().to_string_lossy(), re);
        match &mut p.stdout {
            Some(f) => {
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                let s = String::from_utf8_lossy(&buf);
                log::info!(target: "telegram_image", "Ffprobe output: {}", s);
            }
            None => {}
        }
        return Err(PixivDownloaderError::from("Failed to get image size."));
    }
    let s = match &mut p.stdout {
        Some(f) => {
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            String::from_utf8_lossy(&buf).into_owned()
        }
        None => {
            log::warn!(target: "telegram_image", "No output for ffprobe.");
            return Err(PixivDownloaderError::from("No output for ffprobe."));
        }
    };
    log::debug!(target: "telegram_image", "Ffprobe output: {}", s);
    let s: Vec<_> = s.trim().split('x').collect();
    if s.len() != 2 {
        return Err(PixivDownloaderError::from("Too many output for ffprobe."));
    }
    Ok((s[0].parse()?, s[1].parse()?))
}

pub async fn is_supported_image<S: AsRef<OsStr> + ?Sized>(
    path: &S,
) -> Result<SupportedImage, PixivDownloaderError> {
    let helper = get_helper();
    let ffprobe = helper.ffprobe().unwrap_or(String::from("ffprobe"));
    let re = check_ffprobe(&ffprobe).await?;
    if !re {
        return Err(PixivDownloaderError::from("ffprobe seems not works."));
    }
    let (width, height) = get_image_size(&ffprobe, path).await?;
    let w = width as f64;
    let h = height as f64;
    Ok(if w / h >= 20.0 || h / w >= 20.0 {
        SupportedImage::new(false, false)
    } else if width + height >= 10000 {
        SupportedImage::new(false, true)
    } else {
        let meta = tokio::fs::metadata(path.as_ref()).await?;
        if meta.len() >= MAX_PHOTO_SIZE {
            SupportedImage::new(false, true)
        } else {
            SupportedImage::new(true, false)
        }
    })
}

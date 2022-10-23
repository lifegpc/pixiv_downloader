# pixiv downloader
A pixiv downloader written in Rust.
## Features
* Write exif metatata to picture.
* Merge ugoira(GIF) pictures to video files.
### TODO
See [issues](https://github.com/lifegpc/pixiv_downloader/issues) or [projects](https://github.com/lifegpc/pixiv_downloader/projects)
### all
Enable all unconflicted features, this will enable [`exif`](#exif) and [`ugoira`](#ugoira).
### exif
Enable exif support.  
#### Notice 
* [Exiv2](https://exiv2.org/) is needed. If exiv2 library is not included in system library path. Make sure correct `CMAKE_PREFIX_PATH` is set.
* If you are build on windows system. You need apply patches in [exif/patchs](exif/patchs) folder to make sure Exiv2 support UTF-8 encoding path.
### ugoira
Enable the feature that merge ugoira(GIF) pictures(ZIP file) to video files(MP4 file).
#### Notice
* [libzip](https://libzip.org/) and [FFmpeg](https://ffmpeg.org/) is needed. If these libraries are not included in system library path. Make sure `CMAKE_PREFIX_PATH` and `PKG_CONFIG_PATH` are seted.
* FFmpeg library should be linked with [libX264](https://www.videolan.org/developers/x264.html). Other H.264 encoder may works.

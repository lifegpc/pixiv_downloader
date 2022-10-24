# pixiv downloader
A pixiv downloader written in Rust.
## Features
* Write exif metatata to picture.
* Merge ugoira(GIF) pictures to video files.
### TODO
See [issues](https://github.com/lifegpc/pixiv_downloader/issues) or [projects](https://github.com/lifegpc/pixiv_downloader/projects)
## Rust features flags
### all
Enable all unconflicted features, this will enable [`db_all`](#db_all), [`exif`](#exif) and [`ugoira`](#ugoira).
### db
Enable database support, at least one implement is needed.
### db_all
Enable database support with all implement. This will enable [`db`](#db) and [`db_sqlite`](#db_sqlite).
### db_sqlite
Enable database support with sqlite3.
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
## OpenSSL
Due to schannel not works so well, OpenSSL is needed on Windows system. You may need specify some environment variables to make sure OpenSSL is found. (See [openssl](https://lifegpc.github.io/pixiv_downloader/openssl/#manual) for more information.)

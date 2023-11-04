xtr --package-name pixiv_downloader --package-version "0.0.1" proc_macros/proc_macros.rs src/main.rs -o Language/pixiv_downloader.pot
msgmerge -U --lang=zh_CN Language/pixiv_downloader.zh_CN.po Language/pixiv_downloader.pot

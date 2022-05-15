# The spec for .pd file
Numbers are encoded in little endian.
## Fixed size part
| Offset | Size | Name | Type | Description |
|:------:|:----:|:----:|:----:|:-----------:|
| 0 | 4 | magic_word | Bytes | Should always be `5044FFFF` |
| 4 | 2 | version | Bytes | Version number, the first byte is major version, the second byte is minor version |
| 6 | 4 | file_name_len | u32 | The size of the file name |
| 10 | 1 | status | [Enum(u8)](#status) | The status of the downloaded file. |
| 11 | 1 | type | [Enum(u8)](#type) | The type of the downloader. |
| 12 | 8 | file_size | u64 | The target size of the file. If unknown, set this to 0. |
| 20 | 8 | downloaded_file_size | u64 | The size of the downloaded data. |
| 28 | 4 | part_size | u32 | The size of the each part. Ignored in single thread mode. |
## Non-fixed size part
| Size | Name | Type | Description |
|:----:|:----:|:----:|:-----------:|
| file_name_len | file_name | String | The file name encoded in UTF-8. |
| - | part_datas | [Bytes](#part-status) | The status of the each part. Ignored in single thread mode. |
## Part status
| Offset(Bit) | Size(Bit) | Name | Type | Description |
|:-----------:|:---------:|:----:|:----:|:-----------:|
| 0 | 2 | status | [Enum(u8)](#part-status-1) | The current status of this part. |
| 2 | 30 | downloaded_size | u32 | The size of the downloaded data in this part. |
## Enum
### status
| Number | Name | Description |
|:------:|:----:|:-----------:|
| 0 | started | The download is already started but the target size is unknown. |
| 1 | downloading | The download is started and the tagret size is known. |
| 2 | downloaded | The download is completed. |
### type
| Number | Name | Description |
|:------:|:----:|:-----------:|
| 0 | single_thread | Download in single thread mode. |
| 1 | multi_thread | Download in multiple thread mode. |
### part status
| Number | Name | Description |
|:------:|:----:|:-----------:|
| 0 | waited | The download of this part is waited. |
| 1 | downloading | The download of this part is started. |
| 2 | downloaded | The download of this part is completed. |

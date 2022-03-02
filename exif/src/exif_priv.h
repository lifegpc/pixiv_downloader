#ifndef _EXIF_EXIF_PRIV_H
#define _EXIF_EXIF_PRIV_H
#include "exiv2/exiv2.hpp"
typedef struct ExifImage {
    Exiv2::Image::UniquePtr image;
} ExifImage;
typedef struct ExifKey {
    Exiv2::ExifKey* key = nullptr;
} ExifKey;
typedef struct ExifValue {
    Exiv2::Value::UniquePtr value;
} ExifValue;
typedef struct ExifData {
    Exiv2::ExifData data;
} ExifData;
typedef struct ExifDatum {
    Exiv2::Exifdatum* data;
} ExifDatum;
#endif

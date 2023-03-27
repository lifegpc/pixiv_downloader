#ifndef _EXIF_EXIF_PRIV_H
#define _EXIF_EXIF_PRIV_H
#include "exiv2/exiv2.hpp"
typedef struct ExifImage {
    Exiv2::Image::UniquePtr image;
    Exiv2::ExifData* exif_data_ref;
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
    Exiv2::Exifdatum data;
} ExifDatum;
typedef struct ExifDataItor {
    Exiv2::ExifData* ref;
    Exiv2::ExifMetadata::const_iterator itor;
    Exiv2::ExifMetadata::const_iterator end;
} ExifDataItor;
typedef struct ExifDataMutItor {
    Exiv2::ExifData* ref;
    Exiv2::ExifMetadata::iterator itor;
    Exiv2::ExifMetadata::iterator end;
} ExifDataMutItor;
#endif

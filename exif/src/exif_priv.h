#ifndef _EXIF_EXIF_PRIV_H
#define _EXIF_EXIF_PRIV_H
#include "exiv2/exiv2.hpp"
typedef struct ExifDataRef {
    Exiv2::ExifData* data = nullptr;
} ExifDataRef;
typedef struct ExifImage {
    Exiv2::Image::UniquePtr image;
    ExifDataRef exif_data_ref;
} ExifImage;
typedef struct ExifKey {
    Exiv2::ExifKey* key = nullptr;
} ExifKey;
typedef struct ExifValueRef {
    Exiv2::Value* value;
} ExifValueRef;
typedef struct ExifValue {
    Exiv2::Value::UniquePtr value;
    ExifValueRef ref;
} ExifValue;
typedef struct ExifData {
    Exiv2::ExifData data;
    ExifDataRef ref;
} ExifData;
typedef struct ExifDatumRef {
    Exiv2::Exifdatum* data;
    ExifValueRef ref;
} ExifDatumRef;
typedef struct ExifDatum {
    Exiv2::Exifdatum data;
    ExifDatumRef ref;
} ExifDatum;
typedef struct ExifDataItor {
    ExifDataRef ref;
    ExifDatumRef ref2;
    Exiv2::ExifMetadata::const_iterator begin;
    Exiv2::ExifMetadata::const_iterator itor;
    Exiv2::ExifMetadata::const_iterator end;
} ExifDataItor;
#endif

#ifndef _EXIF_H
#define _EXIF_H
#ifdef __cplusplus
extern "C" {
#endif
#include <stddef.h>
#include <stdint.h>
/// <div rustbindgen opaque></div>
typedef struct ExifImage ExifImage;
/// <div rustbindgen opaque></div>
typedef struct ExifKey ExifKey;
/// <div rustbindgen opaque></div>
typedef struct ExifValue ExifValue;
/// <div rustbindgen opaque></div>
typedef struct ExifData ExifData;
/// <div rustbindgen opaque></div>
typedef struct ExifDatum ExifDatum;
/// <div rustbindgen opaque></div>
typedef struct ExifDataRef ExifDataRef;
#if defined _WIN32 && defined WIN32_DLL
#if BUILD_DLL
#define EXIF_API __declspec(dllexport)
#else
#define EXIF_API __declspec(dllimport)
#endif
#else
#define EXIF_API
#endif

EXIF_API ExifImage* create_exif_image(const char* path);
EXIF_API const ExifDataRef* exif_image_get_exif_data(ExifImage* image);
EXIF_API int exif_image_set_exif_data(ExifImage* image, ExifData* data);
EXIF_API int exif_image_write_metadata(ExifImage* image);
EXIF_API void free_exif_image(ExifImage* img);
EXIF_API ExifKey* exif_create_key_by_key(const char* key);
EXIF_API ExifKey* exif_create_key_by_id(uint16_t id, const char* group_name);
EXIF_API char* exif_get_key_key(ExifKey* key);
EXIF_API char* exif_get_key_family_name(ExifKey* key);
EXIF_API char* exif_get_key_group_name(ExifKey* key);
EXIF_API char* exif_get_key_tag_name(ExifKey* key);
EXIF_API uint16_t exif_get_key_tag_tag(ExifKey* key);
EXIF_API char* exif_get_key_tag_label(ExifKey* key);
EXIF_API char* exif_get_key_tag_desc(ExifKey* key);
EXIF_API int exif_get_key_default_type_id(ExifKey* key);
EXIF_API void exif_free(void* v);
EXIF_API void exif_free_key(ExifKey* key);
EXIF_API ExifValue* exif_create_value(int type_id);
EXIF_API int exif_get_value_type_id(ExifValue* value);
EXIF_API long exif_get_value_count(ExifValue* value);
EXIF_API long exif_get_value_size(ExifValue* value);
EXIF_API long exif_get_value_size_data_area(ExifValue *value);
EXIF_API int exif_value_read(ExifValue* value, const uint8_t* bytes, long len, int byte_order);
EXIF_API int exif_get_value_ok(ExifValue* value);
EXIF_API char* exif_value_to_string(ExifValue* value, size_t* len);
EXIF_API char* exif_value_to_string2(ExifValue* value, size_t* len, long i);
EXIF_API int64_t exif_value_to_int64(ExifValue* value, long i);
EXIF_API ExifData* exif_data_new();
EXIF_API int exif_data_add(ExifData* d, ExifKey* key, ExifValue* value);
EXIF_API int exif_data_clear(ExifData* d);
EXIF_API const ExifDataRef* exif_data_get_ref(ExifData* d);
EXIF_API ExifData* exif_data_ref_clone(ExifDataRef* d);
EXIF_API int exif_data_ref_is_empty(ExifDataRef* d);
EXIF_API long exif_data_ref_get_count(ExifDataRef* d);
EXIF_API void exif_free_value(ExifValue* value);
EXIF_API void exif_free_data(ExifData* d);
EXIF_API void exif_free_datum(ExifDatum* d);
#ifdef __cplusplus
}
#endif
#endif

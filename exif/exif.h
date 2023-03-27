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
/// <div rustbindgen opaque></div>
typedef struct ExifDataItor ExifDataItor;
/// <div rustbindgen opaque></div>
typedef struct ExifDatumRef ExifDatumRef;
/// <div rustbindgen opaque></div>
typedef struct ExifValueRef ExifValueRef;
/// <div rustbindgen opaque></div>
typedef struct ExifDataMutItor ExifDataMutItor;
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
EXIF_API ExifDataRef* exif_image_get_exif_data(ExifImage* image);
EXIF_API int exif_image_read_metadata(ExifImage* image);
EXIF_API int exif_image_set_exif_data(ExifImage* image, ExifData* data);
EXIF_API int exif_image_write_metadata(ExifImage* image);
EXIF_API void free_exif_image(ExifImage* img);
EXIF_API ExifKey* exif_create_key_by_key(const char* key);
EXIF_API ExifKey* exif_create_key_by_id(uint16_t id, const char* group_name);
EXIF_API ExifKey* exif_create_key_by_another(ExifKey* key);
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
EXIF_API ExifValueRef* exif_value_get_ref(ExifValue* value);
EXIF_API int exif_get_value_type_id(ExifValueRef* value);
EXIF_API size_t exif_get_value_count(ExifValueRef* value);
EXIF_API size_t exif_get_value_size(ExifValueRef* value);
EXIF_API size_t exif_get_value_size_data_area(ExifValueRef *value);
EXIF_API int exif_value_read(ExifValueRef* value, const uint8_t* bytes, long len, int byte_order);
EXIF_API int exif_get_value_ok(ExifValueRef* value);
EXIF_API char* exif_value_to_string(ExifValueRef* value, size_t* len);
EXIF_API char* exif_value_to_string2(ExifValueRef* value, size_t* len, size_t i);
EXIF_API int64_t exif_value_to_int64(ExifValueRef* value, size_t i);
EXIF_API ExifValue* exif_value_ref_clone(ExifValueRef* value);
EXIF_API ExifData* exif_data_new();
EXIF_API int exif_data_ref_add(ExifDataRef* d, ExifKey* key, ExifValueRef* value);
EXIF_API int exif_data_ref_clear(ExifDataRef* d);
EXIF_API ExifDataRef* exif_data_get_ref(ExifData* d);
EXIF_API ExifData* exif_data_ref_clone(ExifDataRef* d);
EXIF_API int exif_data_ref_is_empty(ExifDataRef* d);
EXIF_API long exif_data_ref_get_count(ExifDataRef* d);
EXIF_API void exif_data_ref_sort_by_key(ExifDataRef* d);
EXIF_API void exif_data_ref_sort_by_tag(ExifDataRef* d);
EXIF_API ExifDataItor* exif_data_ref_iter(ExifDataRef* d);
EXIF_API ExifDataMutItor* exif_data_ref_iter_mut(ExifDataRef* d);
EXIF_API ExifDatumRef* exif_data_itor_next(ExifDataItor* itor);
EXIF_API ExifDatumRef* exif_data_itor_next_back(ExifDataItor* itor);
EXIF_API ExifDatumRef* exif_data_mutitor_next(ExifDataMutItor* itor);
EXIF_API ExifDatumRef* exif_data_mutitor_next_back(ExifDataMutItor* itor);
EXIF_API char* exif_datum_key(ExifDatumRef* d);
EXIF_API ExifValueRef* exif_datum_value(ExifDatumRef *d);
EXIF_API void exif_datum_set_value(ExifDatumRef* d, ExifValueRef* v);
EXIF_API void exif_free_value(ExifValue* value);
EXIF_API void exif_free_data(ExifData* d);
EXIF_API void exif_free_data_itor(ExifDataItor* itor);
EXIF_API void exif_free_data_mutitor(ExifDataMutItor* itor);
#ifdef __cplusplus
}
#endif
#endif

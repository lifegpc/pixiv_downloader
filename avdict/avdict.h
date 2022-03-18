#ifndef _AVDICT_H
#define _AVDICT_H
#include <stdint.h>
#ifndef BUILD_AVDICT
#define AV_DICT_MATCH_CASE 1
#define AV_DICT_IGNORE_SUFFIX 2
#define AV_DICT_DONT_STRDUP_KEY 4
#define AV_DICT_DONT_STRDUP_VAL 8
#define AV_DICT_DONT_OVERWRITE 16
#define AV_DICT_APPEND 32
#define AV_DICT_MULTIKEY 64
#endif
/// <div rustbindgen opaque></div>
typedef struct AVDict AVDict;
typedef struct AVDictEntry {
    char* key;
    char* value;
} AVDictEntry;
AVDictEntry* avdict_get(const AVDict* m, const char* key, const AVDictEntry* prev, int flags);
int avdict_count(const AVDict* m);
int avdict_set(AVDict** pm, const char* key, const char* value, int flags);
int avdict_copy(AVDict** dst, const AVDict* src, int flags);
void avdict_free(AVDict** m);
void avdict_mfree(void* data);
char* avdict_get_errmsg(int code);
int avdict_set_int(AVDict** pm, const char* key, int64_t value, int flags);
int avdict_parse_string(AVDict** pm, const char* str, const char* key_val_sep, const char* pairs_sep, int flags);
void avdict_avfree(void* data);
int avdict_get_string(const AVDict* m, char** buffer, const char key_val_sep, const char pairs_sep);
#endif

#include "../avdict.h"
#include "libavutil/avutil.h"
#include "libavutil/dict.h"

#include <malloc.h>

struct AVDictionary {
    int count;
    AVDictionaryEntry* elems;
};

AVDictEntry* avdict_get(const AVDict* m, const char* key, const AVDictEntry* prev, int flags) {
    return (AVDictEntry*)av_dict_get((const AVDictionary*)m, key, (const AVDictionaryEntry*)prev, flags);
}

int avdict_count(const AVDict* m) {
    return av_dict_count((const AVDictionary*)m);
}

int avdict_set(AVDict** pm, const char* key, const char* value, int flags) {
    return av_dict_set((AVDictionary**)pm, key, value, flags);
}

int avdict_copy(AVDict** dst, const AVDict* src, int flags) {
    return av_dict_copy((AVDictionary**)dst, (const AVDictionary*)src, flags);
}

void avdict_free(AVDict** m) {
    av_dict_free((AVDictionary**)m);
}

void avdict_mfree(void* data) {
    if (!data) return;
    free(data);
}

char* avdict_get_errmsg(int code) {
    char* msg = malloc(AV_ERROR_MAX_STRING_SIZE);
    if (!msg) return NULL;
    return av_make_error_string(msg, AV_ERROR_MAX_STRING_SIZE, code);
}

int avdict_set_int(AVDict** pm, const char* key, int64_t value, int flags) {
    return av_dict_set_int((AVDictionary**)pm, key, value, flags);
}

int avdict_parse_string(AVDict** pm, const char* str, const char* key_val_sep, const char* pairs_sep, int flags) {
    return av_dict_parse_string((AVDictionary**)pm, str, key_val_sep, pairs_sep, flags);
}

void avdict_avfree(void* data) {
    if (!data) return;
    av_free(data);
}

int avdict_get_string(const AVDict* m, char** buffer, const char key_val_sep, const char pairs_sep) {
    return av_dict_get_string((const AVDictionary*)m, buffer, key_val_sep, pairs_sep);
}

#include "exif.h"
#include "exif_priv.h"
#include "cpp2c.h"

#include <malloc.h>
#include <string.h>
#include "fileop.h"

#define string2char cpp2c::string2char

ExifImage* create_exif_image(const char* path) {
    if (!path) return nullptr;
    if (!fileop::exists(path)) return nullptr;
    auto img = new ExifImage;
    try {
        img->image = Exiv2::ImageFactory::open(path);
    } catch (std::exception& e) {
        printf("%s\n", e.what());
        goto end;
    }
    return img;
end:
    delete img;
    return nullptr;
}

ExifDataRef* exif_image_get_exif_data(ExifImage* image) {
    if (!image || !image->image) return nullptr;
    image->exif_data_ref.data = &image->image->exifData();
    return &image->exif_data_ref;
}

int exif_image_read_metadata(ExifImage* image) {
    if (!image) return 1;
    try {
        image->image->readMetadata();
    } catch (std::exception& e) {
        printf("%s\n", e.what());
        return 1;
    }
    return 0;
}

int exif_image_set_exif_data(ExifImage* image, ExifData* data) {
    if (!image || !data) return 1;
    image->image->setExifData(data->data);
    return 0;
}

int exif_image_write_metadata(ExifImage* image) {
    if (!image) return 1;
    try {
        image->image->writeMetadata();
    } catch (std::exception& e) {
        printf("%s\n", e.what());
        return 1;
    }
    return 0;
}

void free_exif_image(ExifImage* img) {
    if (!img) return;
    delete img;
}

ExifKey* exif_create_key_by_key(const char* key) {
    if (!key) return nullptr;
    ExifKey* k = new ExifKey;
    try {
        k->key = new Exiv2::ExifKey(key);
    } catch (std::exception& e) {
        printf("%s\n", e.what());
        goto end;
    }
    return k;
end:
    if (k->key) delete k->key;
    delete k;
    return nullptr;
}

ExifKey* exif_create_key_by_id(uint16_t id, const char* group_name) {
    if (!group_name) return nullptr;
    ExifKey* k = new ExifKey;
    try {
        k->key = new Exiv2::ExifKey(id, group_name);
    } catch (std::exception& e) {
        printf("%s\n", e.what());
        goto end;
    }
    return k;
end:
    if (k->key) delete k->key;
    delete k;
    return nullptr;
}

char* exif_get_key_key(ExifKey* key) {
    if (!key || !key->key) return nullptr;
    auto s = key->key->key();
    char* re = nullptr;
    if (!string2char(s, re)) return nullptr;
    return re;
}

char* exif_get_key_family_name(ExifKey* key) {
    if (!key || !key->key) return nullptr;
    auto s = key->key->familyName();
    char* re = nullptr;
    if (!string2char(s, re)) return nullptr;
    return re;
}

char* exif_get_key_group_name(ExifKey* key) {
    if (!key || !key->key) return nullptr;
    auto s = key->key->groupName();
    char* re = nullptr;
    if (!string2char(s, re)) return nullptr;
    return re;
}

char* exif_get_key_tag_name(ExifKey* key) {
    if (!key || !key->key) return nullptr;
    auto s = key->key->tagName();
    char* re = nullptr;
    if (!string2char(s, re)) return nullptr;
    return re;
}

uint16_t exif_get_key_tag_tag(ExifKey* key) {
    if (!key || !key->key) return -1;
    return key->key->tag();
}

char* exif_get_key_tag_label(ExifKey* key) {
    if (!key || !key->key) return nullptr;
    auto s = key->key->tagLabel();
    char* re = nullptr;
    if (!string2char(s, re)) return nullptr;
    return re;
}

char* exif_get_key_tag_desc(ExifKey* key) {
    if (!key || !key->key) return nullptr;
    auto s = key->key->tagDesc();
    char* re = nullptr;
    if (!string2char(s, re)) return nullptr;
    return re;
}

int exif_get_key_default_type_id(ExifKey* key) {
    if (!key || !key->key) return -1;
    return key->key->defaultTypeId();
}

void exif_free(void* v) {
    free(v);
}

void exif_free_key(ExifKey* key) {
    if (!key) return;
    if (key->key) delete key->key;
    delete key;
}

ExifValue* exif_create_value(int type_id) {
    ExifValue* v = new ExifValue;
    try {
        auto t = static_cast<Exiv2::TypeId>(type_id);
        v->value = Exiv2::Value::create(t);
    } catch (std::exception& e) {
        printf("%s\n", e.what());
        goto end;
    }
    return v;
end:
    if (v) delete v;
    return nullptr;
}

int exif_get_value_type_id(ExifValue* value) {
    if (!value) return -1;
    return value->value->typeId();
}

long exif_get_value_count(ExifValue* value) {
    if (!value) return -1;
    return value->value->count();
}

long exif_get_value_size(ExifValue* value) {
    if (!value) return -1;
    return value->value->size();
}

long exif_get_value_size_data_area(ExifValue* value) {
    if (!value) return -1;
    return value->value->sizeDataArea();
}

int exif_value_read(ExifValue* value, const uint8_t* bytes, long len, int byte_order) {
    if (!value || !bytes) return -1;
    return value->value->read(bytes, len, static_cast<Exiv2::ByteOrder>(byte_order));
}

int exif_get_value_ok(ExifValue* value) {
    if (!value) return 0;
    return value->value->ok() ? 1 : 0;
}

char* exif_value_to_string(ExifValue* value, size_t* len) {
    if (!value || !len) return nullptr;
    auto s = value->value->toString();
    *len = s.size();
    char* tmp = nullptr;
    if (!string2char(s, tmp)) return nullptr;
    return tmp;
}

char* exif_value_to_string2(ExifValue* value, size_t* len, long i) {
    if (!value || !len) return nullptr;
    auto s = value->value->toString(i);
    *len = s.size();
    char* tmp = nullptr;
    if (!string2char(s, tmp)) return nullptr;
    return tmp;
}

int64_t exif_value_to_int64(ExifValue* value, long i) {
    if (!value) return -1;
    return value->value->toInt64(i);
}

ExifData* exif_data_new() {
    return new ExifData;
}

int exif_data_ref_add(ExifDataRef* d, ExifKey* key, ExifValue* value) {
    if (!d || !d->data || !key || !value || !key->key) return 0;
    d->data->add(*key->key, value->value.get());
    return 1;
}

int exif_data_ref_clear(ExifDataRef* d) {
    if (!d || !d->data) return 0;
    d->data->clear();
    return 1;
}

ExifDataRef* exif_data_get_ref(ExifData* d) {
    if (!d) return nullptr;
    d->ref.data = &d->data;
    return &d->ref;
}

ExifData* exif_data_ref_clone(ExifDataRef* d) {
    if (!d || !d->data) return nullptr;
    auto n = new ExifData;
    if (!n) return nullptr;
    try {
        for (auto i = d->data->begin(); i != d->data->end(); ++i) {
            n->data.add(*i);
        }
    } catch (std::exception& e) {
        printf("%s\n", e.what());
        delete n;
        return nullptr;
    }
    return n;
}

int exif_data_ref_is_empty(ExifDataRef* d) {
    if (!d || !d->data) return -1;
    return d->data->empty() ? 1 : 0;
}

long exif_data_ref_get_count(ExifDataRef* d) {
    if (!d || !d->data) return -1;
    return d->data->count();
}

void exif_free_value(ExifValue* value) {
    if (!value) return;
    delete value;
}

void exif_free_data(ExifData* d) {
    if (!d) return;
    delete d;
}

void exif_free_datum(ExifDatum* d) {
    if (!d) return;
    if (d->data) delete d->data;
}

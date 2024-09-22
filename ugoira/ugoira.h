#ifndef _UGOIRA_H
#define _UGOIRA_H
#define UGOIRA_OK 0
#define UGOIRA_NULL_POINTER 1
#define UGOIRA_ZIP 2
#define UGOIRA_INVALID_MAX_FPS 3
#define UGOIRA_INVALID_FRAMES 4
#define UGOIRA_INVALID_CRF 5
#define UGOIRA_REMOVE_OUTPUT_FILE_FAILED 6
#define UGOIRA_OOM 7
#define UGOIRA_NO_VIDEO_STREAM 8
#define UGOIRA_NO_AVAILABLE_DECODER 9
#define UGOIRA_NO_AVAILABLE_ENCODER 10
#define UGOIRA_OPEN_FILE 11
#define UGOIRA_UNABLE_SCALE 12
#define UGOIRA_JSON_ERROR 13
typedef struct UgoiraFrame {
    char* file;
    float delay;
    struct UgoiraFrame* next;
} UgoiraFrame;
/// <div rustbindgen opaque></div>
typedef struct AVDictionary AVDictionary;
#ifndef BUILD_UGOIRA
typedef struct zip_error {
    int zip_err;
    int sys_err;
    char* str;
} zip_error;
#else
typedef struct zip_error zip_error;
#endif
typedef struct zip_error zip_error_t;
typedef struct UgoiraError {
    int code;
    int zip_err;
    zip_error_t* zip_err2;
} UgoiraError;
UgoiraFrame* new_ugoira_frame(const char* file, float delay, UgoiraFrame* prev);
void free_ugoira_frame(UgoiraFrame* f);
void free_ugoira_frames(UgoiraFrame* frames);
UgoiraError convert_ugoira_to_mp4(const char* src, const char* dest, const UgoiraFrame* frames, float max_fps, const AVDictionary* opts, const AVDictionary* metadata);
char* ugoira_get_zip_err_msg(int code);
void ugoira_mfree(void* data);
zip_error_t* new_ugoira_error();
void free_ugoira_error(zip_error_t* zip_err);
char* ugoira_get_zip_err2_msg(zip_error_t* zip_err2);
#endif

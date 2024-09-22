#include "err.h"
#include "fileop.h"
#include "getopt.h"
#include "rapidjson/document.h"
#include "rapidjson/error/en.h"
extern "C" {
    #include "libavutil/avutil.h"
    #include "libavutil/dict.h"
    #include "ugoira.h"
}
#include "str_util.h"
#include "ugoira_config.h"
#include "wchar_util.h"

#include <fcntl.h>
#include <malloc.h>
#include <stdio.h>
#if _WIN32
#include "Windows.h"
#endif

#if HAVE_PRINTF_S
#define printf printf_s
#endif

#if HAVE_SSCANF_S
#define sscanf sscanf_s
#endif

#if _WIN32
#ifndef _O_BINARY
#define _O_BINARY 0x8000
#endif
#else
#define _O_BINARY 0
#endif

#ifndef _SH_DENYWR
#define _SH_DENYWR 0x20
#endif

void print_help() {
    printf("%s", "Usage: ugoira [options] INPUT DEST JSON\n\
Convert pixiv GIF zip to mp4 file.\n\
\n\
Options:\n\
    -h, --help              Print this help message.\n\
    -M FPS, --max-fps FPS   Set maximum FPS. Default: 60fps.\n\
    -m KEY=VALUE --meta KEY=VALUE\n\
                            Set metadata.\n\
    -f, --force-yuv420p     Force use yuv420p.\n\
    --crf CRF               Set Constant Rate Factor. Default: 18.\n\
    -p PRESET, --preset PRESET\n\
                            Set x264 encoder preset. Default: slow.\n\
    -l LEVEL, --level LEVEL Set H264 profile level.\n\
    -P PROFILE, --profile PROFILE\n\
                            Set H264 profile.\n");
}

#define CRF 128

int main(int argc, char* argv[]) {
#if _WIN32
    SetConsoleOutputCP(CP_UTF8);
    bool have_wargv = false;
    int wargc;
    char** wargv;
    if (wchar_util::getArgv(wargv, wargc)) {
        have_wargv = true;
        argc = wargc;
        argv = wargv;
    }
#endif
    if (argc == 1) {
        print_help();
#if _WIN32
        if (have_wargv) wchar_util::freeArgv(wargv, wargc);
#endif
        return 0;
    }
    struct option opts[] = {
        { "help", 0, nullptr, 'h' },
        { "max-fps", 1, nullptr, 'M' },
        { "meta", 1, nullptr, 'm' },
        { "force-yuv420p", 0, nullptr, 'f' },
        { "crf", 1, nullptr, CRF },
        { "preset", 1, nullptr, 'p' },
        { "level", 1, nullptr, 'l' },
        { "profile", 1, nullptr, 'P' },
        nullptr,
    };
    int c;
    std::string shortopts = "-hM:m:fp:l:P:";
    std::string input;
    std::string dest;
    std::string json;
    bool printh = false;
    float max_fps = 60;
    struct AVDictionary* metadata = nullptr, * options = nullptr;
    while ((c = getopt_long(argc, argv, shortopts.c_str(), opts, nullptr)) != -1) {
        switch (c) {
        case 'h':
            printh = true;
            break;
        case 'M':
            if (sscanf(optarg, "%f", &max_fps) != 1) {
                printf("Invalid max fps: %s\n", optarg);
#if _WIN32
                if (have_wargv) wchar_util::freeArgv(wargv, wargc);
#endif
                av_dict_free(&metadata);
                av_dict_free(&options);
                return UGOIRA_INVALID_MAX_FPS;
            }
            break;
        case 'm':
            if (true) {
                std::string opt(optarg);
                auto t = str_util::str_split(opt, "=", 2);
                if (av_dict_set(&metadata, t.front().c_str(), t.back().c_str(), 0) < 0) {
                    printf("Failed to set metadata: %s\n", optarg);
#if _WIN32
                    if (have_wargv) wchar_util::freeArgv(wargv, wargc);
#endif
                    av_dict_free(&metadata);
                    av_dict_free(&options);
                    return 1;
                }
            }
            break;
        case 'f':
            if (av_dict_set(&options, "force_yuv420p", "1", 0) < 0) {
                printf("Failed to set force_yuv420p: %s\n", optarg);
#if _WIN32
                if (have_wargv) wchar_util::freeArgv(wargv, wargc);
#endif
                av_dict_free(&metadata);
                av_dict_free(&options);
                return 1;
            }
            break;
        case CRF:
            if (av_dict_set(&options, "crf", optarg, 0) < 0) {
                printf("Failed to set crf: %s\n", optarg);
#if _WIN32
                if (have_wargv) wchar_util::freeArgv(wargv, wargc);
#endif
                av_dict_free(&metadata);
                av_dict_free(&options);
                return 1;
            }
            break;
        case 'p':
            if (av_dict_set(&options, "preset", optarg, 0) < 0) {
                printf("Failed to set preset: %s\n", optarg);
#if _WIN32
                if (have_wargv) wchar_util::freeArgv(wargv, wargc);
#endif
                av_dict_free(&metadata);
                av_dict_free(&options);
                return 1;
            }
            break;
        case 'l':
            if (av_dict_set(&options, "level", optarg, 0) < 0) {
                printf("Failed to set level: %s\n", optarg);
#if _WIN32
                if (have_wargv) wchar_util::freeArgv(wargv, wargc);
#endif
                av_dict_free(&metadata);
                av_dict_free(&options);
                return 1;
            }
            break;
        case 'P':
            if (av_dict_set(&options, "profile", optarg, 0) < 0) {
                printf("Failed to set profile: %s\n", optarg);
#if _WIN32
                if (have_wargv) wchar_util::freeArgv(wargv, wargc);
#endif
                av_dict_free(&metadata);
                av_dict_free(&options);
                return 1;
            }
            break;
        case 1:
            if (input.empty()) {
                input = optarg;
            } else if (dest.empty()) {
                dest = optarg;
            } else if (json.empty()) {
                json = optarg;
            } else {
#if _WIN32
                if (have_wargv) wchar_util::freeArgv(wargv, wargc);
#endif
                printf("Too much arguments.\n");
                av_dict_free(&metadata);
                av_dict_free(&options);
                return 1;
            }
            break;
        case '?':
        default:
#if _WIN32
            if (have_wargv) wchar_util::freeArgv(wargv, wargc);
#endif
            av_dict_free(&metadata);
            av_dict_free(&options);
            return 1;
        }
    }
#if _WIN32
    if (have_wargv) wchar_util::freeArgv(wargv, wargc);
#endif
    if (printh) {
        print_help();
        av_dict_free(&metadata);
        av_dict_free(&options);
        return 0;
    }
    size_t size;
    if (!fileop::get_file_size(json, size)) {
        printf("Failed to get size of JSON file.\n");
        av_dict_free(&metadata);
        av_dict_free(&options);
        return UGOIRA_OPEN_FILE;
    }
    int fd;
    int err;
    if (err = fileop::open(json, fd, O_RDONLY | _O_BINARY, _SH_DENYWR)) {
        std::string msg = "Unknown error.";
        err::get_errno_message(msg, err);
        printf("Failed to open file: %s(%d)\n", msg.c_str(), err);
        av_dict_free(&metadata);
        av_dict_free(&options);
        return UGOIRA_OPEN_FILE;
    };
    char* buf = (char*)malloc(size + 1);
    if (!buf) {
        fileop::close(fd);
        printf("Failed to malloc memory.");
        av_dict_free(&metadata);
        av_dict_free(&options);
        return UGOIRA_OOM;
    }
    FILE* f = fileop::fdopen(fd, "r");
    if (fread(buf, 1, size, f) != size) {
        printf("Failed to read JSON file.");
        fileop::fclose(f);
        free(buf);
        av_dict_free(&metadata);
        av_dict_free(&options);
        return UGOIRA_OPEN_FILE;
    }
    fileop::fclose(f);
    rapidjson::Document d;
    d.Parse(buf);
    free(buf);
    if (d.HasParseError()) {
        auto m = rapidjson::GetParseError_En(d.GetParseError());
        printf("Failed to parse JSON: %s\n", m);
        av_dict_free(&metadata);
        av_dict_free(&options);
        return UGOIRA_JSON_ERROR;
    }
    UgoiraFrame* top = nullptr, *tail = nullptr;
    auto arr = d.GetArray();
    for (auto i = arr.Begin(); i != arr.End(); i++) {
#ifdef GetObject
#undef GetObject
#endif
        auto obj = i->GetObject();
        auto file = obj["file"].GetString();
        auto delay = obj["delay"].GetFloat();
        tail = new_ugoira_frame(file, delay, tail);
        if (!tail) {
            if (top) {
                free_ugoira_frames(top);
            }
            av_dict_free(&metadata);
            av_dict_free(&options);
            printf("Failed to alloc memory for ugoira frame.\n");
            return UGOIRA_OOM;
        }
        if (!top) {
            top = tail;
        }
    }
    auto e = convert_ugoira_to_mp4(input.c_str(), dest.c_str(), top, max_fps, options, metadata);
    free_ugoira_frames(top);
    av_dict_free(&metadata);
    av_dict_free(&options);
    return e.code;
}

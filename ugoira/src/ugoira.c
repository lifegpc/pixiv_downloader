#include "ugoira_config.h"
#include "../ugoira.h"

#include <malloc.h>
#include <string.h>
#include "cfileop.h"
#include "cmath.h"
#include "cstr_util.h"
#include "libavutil/avutil.h"
#include "libavutil/opt.h"
#include "libavformat/avformat.h"
#include "libavcodec/avcodec.h"
#include "libswscale/swscale.h"
#include "zip.h"

#define RERR(e) (UgoiraError){ e, 0, 0 }
#define RZERR(e) (UgoiraError){ UGOIRA_ZIP, e, 0 }
#define CHECK_ZIP_ERR(zip, e) { zip_error_t* tmp = zip_get_error(zip); \
    if (tmp->sys_err || tmp->zip_err) { \
        copy_zip_error(&e, tmp); \
        goto end; \
    } \
}

#if HAVE_SSCANF_S
#define sscanf sscanf_s
#endif

UgoiraFrame* new_ugoira_frame(const char* file, float delay, UgoiraFrame* prev) {
    if (!file || delay <= 0) return NULL;
    UgoiraFrame* f = malloc(sizeof(UgoiraFrame));
    if (!f) return NULL;
    memset(f, 0, sizeof(UgoiraFrame));
    if (cstr_util_copy_str(&f->file, file)) {
        free(f);
        return NULL;
    }
    f->delay = delay;
    if (prev) prev->next = f;
    return f;
}

void free_ugoira_frame(UgoiraFrame* f) {
    if (!f) return;
    if (f->file) free(f->file);
    free(f);
}

void free_ugoira_frames(UgoiraFrame* frames) {
    if (!frames) return;
    UgoiraFrame* next = frames->next;
    free_ugoira_frame(frames);
    while (next) {
        frames = next;
        next = frames->next;
        free_ugoira_frame(frames);
    }
}

float ugoira_cal_fps(const UgoiraFrame* frames, float max_fps) {
    if (!frames) return max_fps;
    int re = frames->delay;
    const UgoiraFrame* cur = frames->next;
    while (cur) {
        re = GCD(re, cur->delay);
        cur = cur->next;
    }
    return FFMIN(1000 / ((float)re), max_fps);
}

int ugoira_is_supported_pixfmt(enum AVPixelFormat fmt, const enum AVPixelFormat* fmts) {
    size_t i = 0;
    if (fmt == AV_PIX_FMT_NONE || !fmts) return 0;
    while (fmts[i] != AV_PIX_FMT_NONE) {
        if (fmt == fmts[i]) return 1;
        i++;
    }
    return 0;
}

const AVCodec* ugoira_find_encoder() {
    const AVCodec* c = avcodec_find_encoder_by_name("libx264");
    if (!c) c = avcodec_find_encoder(AV_CODEC_ID_H264);
    return c;
}

int check_ugoira_frames(const UgoiraFrame* frames) {
    if (!frames) return 0;
    const UgoiraFrame* cur = frames;
    while (cur) {
        if (!cur->file || cur->delay <= 0) return 0;
        cur = cur->next;
    }
    return 1;
}

void copy_zip_error(zip_error_t* dest, zip_error_t* src) {
    dest->sys_err = src->sys_err;
    dest->zip_err = src->zip_err;
    if (dest->str) {
        free(dest->str);
        dest->str = NULL;
    }
    cstr_util_copy_str(&dest->str, src->str);
}

int zip_file_read_packet(void* f, uint8_t* buf, int buf_size) {
    zip_file_t* zf = f;
    int64_t count = zip_fread(zf, (void*)buf, buf_size);
    if (count == -1) {
        return AVERROR(EINVAL);
    }
    if (count == 0) {
        return AVERROR_EOF;
    }
    return count;
}

int ugoira_encode_video(AVFrame* ofr, AVFormatContext* oc, AVCodecContext* eoc, char* writed_data, int64_t* pts, unsigned int stream_index, AVRational time_base) {
    if (!oc || !eoc || !writed_data) return UGOIRA_NULL_POINTER;
    if (ofr && !pts) return UGOIRA_NULL_POINTER;
    int err = UGOIRA_OK;
    *writed_data = 0;
    AVPacket* pkt = av_packet_alloc();
    if (!pkt) {
        return UGOIRA_OOM;
    }
    if (ofr) {
        ofr->pts = *pts;
        *pts += av_rescale_q_rnd(1, time_base, oc->streams[stream_index]->time_base, AV_ROUND_NEAR_INF | AV_ROUND_PASS_MINMAX);
        ofr->pkt_dts = ofr->pts;
    }
    if ((err = avcodec_send_frame(eoc, ofr)) < 0) {
        if (err == AVERROR_EOF) {
            err = 0;
        } else {
            goto end;
        }
    }
    err = avcodec_receive_packet(eoc, pkt);
    if (err >= 0) {
        err = 0;
        *writed_data = 1;
    } else if (err == AVERROR_EOF || err == AVERROR(EAGAIN)) {
        err = 0;
        goto end;
    } else {
        goto end;
    }
    if (*writed_data && pkt) {
        pkt->stream_index = stream_index;
        if ((err = av_write_frame(oc, pkt)) < 0) {
            goto end;
        }
        err = 0;
    }
end:
    if (pkt) av_packet_free(&pkt);
    return err;
}

UgoiraError convert_ugoira_to_mp4(const char* src, const char* dest, const UgoiraFrame* frames, float max_fps, const AVDictionary* opts, const AVDictionary* metadata) {
    if (!src || !dest || !frames) return RERR(UGOIRA_NULL_POINTER);
    int err = UGOIRA_OK;
    int zip_err = 0;
    zip_t* zip = NULL;
    int dcrf = 18;
    AVDictionaryEntry* tmp_ent = NULL;
    if (max_fps <= 0) {
        return RERR(UGOIRA_INVALID_MAX_FPS);
    }
    if (!check_ugoira_frames(frames)) {
        return RERR(UGOIRA_INVALID_FRAMES);
    }
    tmp_ent = av_dict_get(opts, "crf", NULL, 0);
    if (tmp_ent) {
        int tmp = 0;
        if (sscanf(tmp_ent->value, "%i", &tmp) != 1) {
            return RERR(UGOIRA_INVALID_CRF);
        }
        dcrf = tmp;
    }
    zip_error_t ziperr;
    AVRational fps = { (int)(ugoira_cal_fps(frames, max_fps) * AV_TIME_BASE + 0.5), AV_TIME_BASE };
    AVRational time_base = { fps.den, fps.num };
    AVFormatContext* ic = NULL, * oc = NULL;
    AVIOContext* iioc = NULL;
    AVFrame* ifr = NULL, * ofr = NULL;
    AVCodecContext* eoc = NULL, * eic = NULL;
    AVStream* is = NULL, * os = NULL;
    const AVCodec* output_codec = NULL, * input_codec = NULL;
    const UgoiraFrame* cur_frame = frames;
    size_t i = 0;
    struct SwsContext* sws_ctx = NULL;
    enum AVPixelFormat pre_pixfmt = AV_PIX_FMT_NONE;
    int pre_width = -1, pre_height = -1;
    zip_file_t* zf = NULL;
    unsigned char* buff = NULL;
    AVPacket pkt;
    int64_t pts = 0, max_de = 0;
    const static AVRational tb = { 1, 1000 };
    char writed = 0;
    zip_error_init(&ziperr);
    if (fileop_exists(dest)) {
        if (!fileop_remove(dest)) {
            return RERR(UGOIRA_REMOVE_OUTPUT_FILE_FAILED);
        }
    }
    if (!(ifr = av_frame_alloc())) {
        err = UGOIRA_OOM;
        goto end;
    }
    if (!(ofr = av_frame_alloc())) {
        err = UGOIRA_OOM;
        goto end;
    }
    zip = zip_open(src, ZIP_RDONLY, &zip_err);
    if (!zip) {
        goto end;
    }
    while (cur_frame) {
        if (i != 0) {
            if (eic) {
                avcodec_free_context(&eic);
                eic = NULL;
            }
            avformat_close_input(&ic);
            ic = NULL;
            if (zip_fclose(zf)) {
                CHECK_ZIP_ERR(zip, ziperr);
            }
            zf = NULL;
            if (iioc) av_freep(&iioc->buffer);
            avio_context_free(&iioc);
            iioc = NULL;
        }
        if (!(ic = avformat_alloc_context())) {
            err = UGOIRA_OOM;
            goto end;
        }
        zf = zip_fopen(zip, cur_frame->file, 0);
        if (!zf) {
            CHECK_ZIP_ERR(zip, ziperr);
        }
        buff = av_malloc(4096);
        if (!buff) {
            err = UGOIRA_OOM;
            goto end;
        }
        if (!(iioc = avio_alloc_context(buff, 4096, 0, (void*)zf, zip_file_read_packet, NULL, NULL))) {
            err = UGOIRA_OOM;
            goto end;
        }
        ic->pb = iioc;
        if ((err = avformat_open_input(&ic, NULL, NULL, NULL)) < 0) {
            goto end;
        }
        if ((err = avformat_find_stream_info(ic, NULL)) < 0) {
            goto end;
        }
        err = 0;
        for (unsigned int si = 0; si < ic->nb_streams; i++) {
            AVStream* s = ic->streams[si];
            if (s->codecpar->codec_type == AVMEDIA_TYPE_VIDEO) {
                is = s;
                break;
            }
        }
        if (!is) {
            err = UGOIRA_NO_VIDEO_STREAM;
            goto end;
        }
        if (!(input_codec = avcodec_find_decoder(is->codecpar->codec_id))) {
            err = UGOIRA_NO_AVAILABLE_DECODER;
            goto end;
        }
        if (!(eic = avcodec_alloc_context3(input_codec))) {
            err = UGOIRA_OOM;
            goto end;
        }
        if ((err = avcodec_parameters_to_context(eic, is->codecpar)) < 0) {
            goto end;
        }
        err = 0;
        if ((err = avcodec_open2(eic, input_codec, NULL)) < 0) {
            goto end;
        }
        if (i == 0) {
            AVDictionaryEntry* force_yuv420p = NULL;
            output_codec = ugoira_find_encoder();
            if (!output_codec) {
                err = UGOIRA_NO_AVAILABLE_ENCODER;
                goto end;
            }
            if ((err = avformat_alloc_output_context2(&oc, NULL, "mp4", dest)) < 0) {
                goto end;
            }
            if (metadata) {
                if ((err = av_dict_copy(&oc->metadata, metadata, 0)) < 0) {
                    goto end;
                }
            }
            err = 0;
            if (!(eoc = avcodec_alloc_context3(output_codec))) {
                err = UGOIRA_OOM;
                goto end;
            }
            eoc->width = eic->width;
            eoc->height = eic->height;
            eoc->sample_aspect_ratio = eic->sample_aspect_ratio;
            eoc->framerate = fps;
            if (opts) {
                force_yuv420p = av_dict_get(opts, "force_yuv420p", NULL, 0);
            }
            if (!force_yuv420p && ugoira_is_supported_pixfmt(eic->pix_fmt, output_codec->pix_fmts)) {
                eoc->pix_fmt = eic->pix_fmt;
            } else {
                eoc->pix_fmt = AV_PIX_FMT_YUV420P;
            }
            ofr->width = eoc->width;
            ofr->height = eoc->height;
            ofr->format = eoc->pix_fmt;
            eoc->time_base = AV_TIME_BASE_Q;
            if (!strcmp(output_codec->name, "libx264")) {
                AVDictionaryEntry* tmp = NULL;
                if (opts) {
                    tmp = av_dict_get(opts, "preset", NULL, 0);
                }
                if (tmp) {
                    av_opt_set(eoc->priv_data, "preset", tmp->value, 0);
                } else {
                    av_opt_set(eoc->priv_data, "preset", "slow", 0);
                }
                av_opt_set_int(eoc->priv_data, "crf", dcrf, 0);
                if (opts) {
                    tmp = av_dict_get(opts, "level", NULL, 0);
                }
                if (tmp) {
                    av_opt_set(eoc->priv_data, "level", tmp->value, 0);
                }
                if (opts) {
                    tmp = av_dict_get(opts, "profile", NULL, 0);
                }
                if (tmp) {
                    av_opt_set(eoc->priv_data, "profile", tmp->value, 0);
                }
            }
            if ((err = av_frame_get_buffer(ofr, 0)) < 0) {
                goto end;
            }
            if (!(os = avformat_new_stream(oc, output_codec))) {
                err = UGOIRA_OOM;
                goto end;
            }
            os->avg_frame_rate = fps;
            os->r_frame_rate = fps;
            os->time_base = AV_TIME_BASE_Q;
            if ((err = avcodec_open2(eoc, output_codec, NULL)) < 0) {
                goto end;
            }
            if ((err = avcodec_parameters_from_context(os->codecpar, eoc)) < 0) {
                goto end;
            }
            err = 0;
            if (!(oc->oformat->flags & AVFMT_NOFILE)) {
                int ret = avio_open(&oc->pb, dest, AVIO_FLAG_WRITE);
                if (ret < 0) {
                    err = UGOIRA_OPEN_FILE;
                    goto end;
                }
            }
            if ((err = avformat_write_header(oc, NULL)) < 0) {
                goto end;
            }
            err = 0;
        }
        if (!sws_ctx || eic->pix_fmt != pre_pixfmt || eic->width != pre_width || eic->height != pre_height) {
            if (sws_ctx) {
                sws_freeContext(sws_ctx);
                sws_ctx = NULL;
            }
            if (!(sws_ctx = sws_getContext(eic->width, eic->height, eic->pix_fmt, eoc->width, eoc->height, eoc->pix_fmt, SWS_BILINEAR, NULL, NULL, NULL))) {
                err = UGOIRA_UNABLE_SCALE;
                goto end;
            }
            pre_pixfmt = eic->pix_fmt;
            pre_width = eic->width;
            pre_height = eic->height;
        }
        while (1) {
            if ((err = av_read_frame(ic, &pkt)) < 0) {
                goto end;
            }
            if (pkt.stream_index != is->index) {
                av_packet_unref(&pkt);
                continue;
            }
            if ((err = avcodec_send_packet(eic, &pkt)) < 0) {
                av_packet_unref(&pkt);
                goto end;
            }
            av_packet_unref(&pkt);
            err = avcodec_receive_frame(eic, ifr);
            if (err >= 0) {
                err = 0;
                if ((err = av_frame_make_writable(ofr)) < 0) {
                    goto end;
                }
                if ((err = sws_scale(sws_ctx, (const uint8_t* const*)ifr->data, ifr->linesize, 0, ifr->height, ofr->data, ofr->linesize)) < 0) {
                    goto end;
                }
                err = 0;
            } else if (err == AVERROR(EAGAIN)) {
                err = 0;
                continue;
            } else {
                goto end;
            }
            max_de += av_rescale_q_rnd(cur_frame->delay, tb, os->time_base, AV_ROUND_NEAR_INF | AV_ROUND_PASS_MINMAX);
            while (pts < max_de) {
                if ((err = ugoira_encode_video(ofr, oc, eoc, &writed, &pts, os->index, time_base)) != UGOIRA_OK) {
                    goto end;
                }
            }
            break;
        }
        cur_frame = cur_frame->next;
        i += 1;
    }
    if (os) {
        while (1) {
            if ((err = ugoira_encode_video(NULL, oc, eoc, &writed, NULL, os->index, time_base)) != UGOIRA_OK) {
                goto end;
            }
            if (!writed) {
                break;
            }
        }
    }
    if ((err = av_write_trailer(oc)) < 0) {
        goto end;
    }
end:
    if (ifr) av_frame_free(&ifr);
    if (ofr) av_frame_free(&ofr);
    if (sws_ctx) sws_freeContext(sws_ctx);
    if (eic) avcodec_free_context(&eic);
    if (eoc) avcodec_free_context(&eoc);
    if (oc) {
        if (!(oc->oformat->flags & AVFMT_NOFILE)) avio_closep(&oc->pb);
        avformat_free_context(oc);
    }
    if (ic) avformat_close_input(&ic);
    if (iioc) {
        av_freep(&iioc->buffer);
        avio_context_free(&iioc);
    }
    if (zf) {
        zip_err = zip_fclose(zf);
    }
    if (zip) {
        zip_err = zip_close(zip);
    }
    if (ziperr.sys_err || ziperr.zip_err) {
        zip_error_t* tmp = malloc(sizeof(zip_error_t));
        if (!tmp) {
            return RERR(UGOIRA_OOM);
        }
        memcpy(tmp, &ziperr, sizeof(zip_error_t));
        return (UgoiraError){ UGOIRA_ZIP, 0, tmp };
    }
    return zip_err ? RZERR(zip_err) : RERR(err);
}

char* ugoira_get_zip_err_msg(int code) {
    zip_error_t err;
    zip_error_init_with_code(&err, code);
    const char* tmp = zip_error_strerror(&err);
    char* re = NULL;
    if (cstr_util_copy_str(&re, tmp)) {
        zip_error_fini(&err);
        return NULL;
    }
    zip_error_fini(&err);
    return re;
}

void ugoira_mfree(void* data) {
    if (data) free(data);
}

void free_ugoira_error(zip_error_t* zip_err) {
    if (!zip_err) return;
    zip_error_fini(zip_err);
    free(zip_err);
}

char* ugoira_get_zip_err2_msg(zip_error_t* zip_err2) {
    if (!zip_err2) return NULL;
    const char* errmsg = zip_error_strerror(zip_err2);
    char* re = NULL;
    if (cstr_util_copy_str(&re, errmsg)) {
        return NULL;
    }
    return re;
}

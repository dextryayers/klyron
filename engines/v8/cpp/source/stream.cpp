#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstring>
#include <cstdlib>
#include <vector>
#include <mutex>

struct klyron_v8_stream {
    klyron_v8_isolate* parent;
    KlyronV8StreamReadCallback read_cb;
    KlyronV8StreamWriteCallback write_cb;
    KlyronV8StreamCloseCallback close_cb;
    void* user_data;
    void* close_user_data;
    bool ended;
    bool destroyed;
    std::vector<unsigned char> buffer;
    std::mutex mutex;
};

klyron_v8_stream_t* klyron_v8_stream_new_readable(klyron_v8_context_t* ctx,
                                                   KlyronV8StreamReadCallback read_cb,
                                                   void* user_data) {
    if (!ctx) return nullptr;
    auto* stream = new klyron_v8_stream();
    stream->parent = ctx->parent;
    stream->read_cb = read_cb;
    stream->write_cb = nullptr;
    stream->close_cb = nullptr;
    stream->user_data = user_data;
    stream->close_user_data = nullptr;
    stream->ended = false;
    stream->destroyed = false;
    return stream;
}

klyron_v8_stream_t* klyron_v8_stream_new_writable(klyron_v8_context_t* ctx,
                                                   KlyronV8StreamWriteCallback write_cb,
                                                   void* user_data) {
    if (!ctx) return nullptr;
    auto* stream = new klyron_v8_stream();
    stream->parent = ctx->parent;
    stream->read_cb = nullptr;
    stream->write_cb = write_cb;
    stream->close_cb = nullptr;
    stream->user_data = user_data;
    stream->close_user_data = nullptr;
    stream->ended = false;
    stream->destroyed = false;
    return stream;
}

klyron_v8_stream_t* klyron_v8_stream_new_transform(klyron_v8_context_t* ctx,
                                                    KlyronV8StreamReadCallback read_cb,
                                                    KlyronV8StreamWriteCallback write_cb,
                                                    void* user_data) {
    if (!ctx) return nullptr;
    auto* stream = new klyron_v8_stream();
    stream->parent = ctx->parent;
    stream->read_cb = read_cb;
    stream->write_cb = write_cb;
    stream->close_cb = nullptr;
    stream->user_data = user_data;
    stream->close_user_data = nullptr;
    stream->ended = false;
    stream->destroyed = false;
    return stream;
}

klyron_v8_result_t klyron_v8_stream_write(klyron_v8_context_t* ctx,
                                           klyron_v8_stream_t* stream,
                                           const unsigned char* data,
                                           size_t length) {
    klyron_v8_result_t result = {false, {0}};
    if (!stream || stream->destroyed) {
        std::strncpy(result.error, "stream destroyed", KLYRON_V8_ERROR_BUF_SIZE - 1);
        return result;
    }

    if (stream->write_cb) {
        size_t written = stream->write_cb(stream, data, length, stream->user_data);
        if (written == length) result.success = true;
        else std::strncpy(result.error, "write callback failed", KLYRON_V8_ERROR_BUF_SIZE - 1);
    } else {
        std::lock_guard<std::mutex> lock(stream->mutex);
        stream->buffer.insert(stream->buffer.end(), data, data + length);
        result.success = true;
    }
    return result;
}

klyron_v8_result_t klyron_v8_stream_end(klyron_v8_context_t* ctx,
                                         klyron_v8_stream_t* stream,
                                         const unsigned char* data,
                                         size_t length) {
    klyron_v8_result_t result = {false, {0}};
    if (!stream || stream->destroyed) return result;
    if (data && length > 0) {
        klyron_v8_stream_write(ctx, stream, data, length);
    }
    stream->ended = true;
    if (stream->close_cb) {
        stream->close_cb(stream, stream->close_user_data);
    }
    result.success = true;
    return result;
}

klyron_v8_result_t klyron_v8_stream_destroy(klyron_v8_context_t* ctx,
                                             klyron_v8_stream_t* stream) {
    klyron_v8_result_t result = {false, {0}};
    if (!stream) return result;
    stream->destroyed = true;
    stream->ended = true;
    if (stream->close_cb) {
        stream->close_cb(stream, stream->close_user_data);
    }
    delete stream;
    result.success = true;
    return result;
}

void klyron_v8_stream_set_close_callback(klyron_v8_stream_t* stream,
                                          KlyronV8StreamCloseCallback cb,
                                          void* user_data) {
    if (!stream) return;
    stream->close_cb = cb;
    stream->close_user_data = user_data;
}

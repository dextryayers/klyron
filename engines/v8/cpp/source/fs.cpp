#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstring>
#include <cstdio>
#include <string>
#include <vector>
#include <sys/stat.h>
#include <sys/types.h>
#include <fcntl.h>
#include <unistd.h>
#include <dirent.h>
#include <cerrno>

static void set_result_from_errno(klyron_v8_result_t* result, int err) {
    std::strncpy(result->error, strerror(err), KLYRON_V8_ERROR_BUF_SIZE - 1);
    result->error[KLYRON_V8_ERROR_BUF_SIZE - 1] = '\0';
}

static void stat_to_klyron(const struct stat& st, klyron_v8_stat_t* out) {
    out->dev = st.st_dev;
    out->ino = st.st_ino;
    out->mode = st.st_mode;
    out->uid = st.st_uid;
    out->gid = st.st_gid;
    out->size = st.st_size;
    out->blksize = st.st_blksize;
    out->blocks = st.st_blocks;
    out->atime = st.st_atime;
    out->mtime = st.st_mtime;
    out->ctime = st.st_ctime;
    if (S_ISREG(st.st_mode)) out->type = 0;
    else if (S_ISDIR(st.st_mode)) out->type = 1;
    else if (S_ISLNK(st.st_mode)) out->type = 2;
    else out->type = 3;
}

klyron_v8_result_t klyron_v8_fs_read_file(klyron_v8_context_t* ctx, const char* path, klyron_v8_value_t** result) {
    klyron_v8_result_t res = {false, {0}};
    if (!ctx || !path) return res;

    FILE* fp = std::fopen(path, "rb");
    if (!fp) { set_result_from_errno(&res, errno); return res; }

    std::fseek(fp, 0, SEEK_END);
    long size = std::ftell(fp);
    std::fseek(fp, 0, SEEK_SET);

    if (size < 0) { std::fclose(fp); set_result_from_errno(&res, errno); return res; }

    auto* buf = static_cast<unsigned char*>(std::malloc(size));
    if (!buf) { std::fclose(fp); set_result_from_errno(&res, ENOMEM); return res; }

    size_t nread = std::fread(buf, 1, size, fp);
    std::fclose(fp);

    if (nread != (size_t)size) { std::free(buf); set_result_from_errno(&res, errno); return res; }

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto ab = v8::ArrayBuffer::New(iso, size);
    std::memcpy(ab->GetBackingStore()->Data(), buf, size);
    std::free(buf);

    auto ta = v8::Uint8Array::New(ab, 0, size);
    *result = new klyron_v8_value(iso, ta, ctx->parent);
    res.success = true;
    return res;
}

klyron_v8_result_t klyron_v8_fs_write_file(klyron_v8_context_t* ctx, const char* path, const unsigned char* data, size_t length) {
    klyron_v8_result_t result = {false, {0}};
    FILE* fp = std::fopen(path, "wb");
    if (!fp) { set_result_from_errno(&result, errno); return result; }
    size_t written = std::fwrite(data, 1, length, fp);
    std::fclose(fp);
    if (written != length) { set_result_from_errno(&result, errno); return result; }
    result.success = true;
    return result;
}

klyron_v8_result_t klyron_v8_fs_append_file(klyron_v8_context_t* ctx, const char* path, const unsigned char* data, size_t length) {
    klyron_v8_result_t result = {false, {0}};
    FILE* fp = std::fopen(path, "ab");
    if (!fp) { set_result_from_errno(&result, errno); return result; }
    size_t written = std::fwrite(data, 1, length, fp);
    std::fclose(fp);
    if (written != length) { set_result_from_errno(&result, errno); return result; }
    result.success = true;
    return result;
}

klyron_v8_result_t klyron_v8_fs_stat(klyron_v8_context_t* ctx, const char* path, klyron_v8_stat_t* stat) {
    klyron_v8_result_t result = {false, {0}};
    if (!path || !stat) return result;

    struct stat st;
    if (::stat(path, &st) != 0) { set_result_from_errno(&result, errno); return result; }
    stat_to_klyron(st, stat);
    result.success = true;
    return result;
}

klyron_v8_result_t klyron_v8_fs_mkdir(klyron_v8_context_t* ctx, const char* path, int32_t mode) {
    klyron_v8_result_t result = {false, {0}};
    if (!path) return result;
    mode_t m = (mode > 0) ? (mode_t)mode : 0755;
    if (::mkdir(path, m) != 0) { set_result_from_errno(&result, errno); return result; }
    result.success = true;
    return result;
}

klyron_v8_result_t klyron_v8_fs_rmdir(klyron_v8_context_t* ctx, const char* path) {
    klyron_v8_result_t result = {false, {0}};
    if (!path) return result;
    if (::rmdir(path) != 0) { set_result_from_errno(&result, errno); return result; }
    result.success = true;
    return result;
}

klyron_v8_result_t klyron_v8_fs_unlink(klyron_v8_context_t* ctx, const char* path) {
    klyron_v8_result_t result = {false, {0}};
    if (!path) return result;
    if (::unlink(path) != 0) { set_result_from_errno(&result, errno); return result; }
    result.success = true;
    return result;
}

klyron_v8_result_t klyron_v8_fs_rename(klyron_v8_context_t* ctx, const char* old_path, const char* new_path) {
    klyron_v8_result_t result = {false, {0}};
    if (!old_path || !new_path) return result;
    if (std::rename(old_path, new_path) != 0) { set_result_from_errno(&result, errno); return result; }
    result.success = true;
    return result;
}

klyron_v8_result_t klyron_v8_fs_exists(klyron_v8_context_t* ctx, const char* path, bool* exists) {
    klyron_v8_result_t result = {false, {0}};
    if (!path || !exists) return result;
    struct stat st;
    *exists = (::stat(path, &st) == 0);
    result.success = true;
    return result;
}

klyron_v8_value_t* klyron_v8_fs_read_dir(klyron_v8_context_t* ctx, const char* path) {
    if (!ctx || !path) return nullptr;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto arr = v8::Array::New(iso);

    DIR* dir = ::opendir(path);
    if (!dir) return new klyron_v8_value(iso, arr, ctx->parent);

    struct dirent* entry;
    int idx = 0;
    while ((entry = ::readdir(dir)) != nullptr) {
        auto name = v8::String::NewFromUtf8(iso, entry->d_name, v8::NewStringType::kNormal).ToLocalChecked();
        arr->Set(context, idx++, name).Check();
    }
    ::closedir(dir);

    return new klyron_v8_value(iso, arr, ctx->parent);
}

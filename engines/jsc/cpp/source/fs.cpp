#include "klyron_jsc.h"
#include "cpp/impl/internal.h"
#include <cstring>
#include <cstdlib>
#include <string>
#include <vector>
#include <sys/stat.h>
#include <unistd.h>
#include <fcntl.h>
#include <dirent.h>
#include <cerrno>
#include <climits>

static std::string jsc_val_to_cpp_string(klyron_jsc_engine_t* engine, JSValueRef val) {
    JSValueRef exc = nullptr;
    JSStringRef str = JSValueToStringCopy(engine->ctx, val, &exc);
    if (!str) return "";
    std::string s = jsc_string_to_std(str);
    JSStringRelease(str);
    return s;
}

static klyron_jsc_value_t* make_string(klyron_jsc_engine_t* engine, const std::string& s) {
    JSStringRef jsstr = jsc_string_from_cstr(s.c_str());
    JSValueRef val = JSValueMakeString(engine->ctx, jsstr);
    JSStringRelease(jsstr);
    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

static klyron_jsc_value_t* make_number(klyron_jsc_engine_t* engine, double n) {
    auto v = new klyron_jsc_value_t(engine->ctx, JSValueMakeNumber(engine->ctx, n));
    v->protect();
    return v;
}

static klyron_jsc_value_t* make_bool(klyron_jsc_engine_t* engine, bool b) {
    auto v = new klyron_jsc_value_t(engine->ctx, JSValueMakeBoolean(engine->ctx, b));
    v->protect();
    return v;
}

static JSObjectRef make_stat_result(klyron_jsc_engine_t* engine, const struct stat& st) {
    JSObjectRef obj = JSObjectMake(engine->ctx, nullptr, nullptr);
    auto set_num = [&](const char* name, double val) {
        JSStringRef key = jsc_string_from_cstr(name);
        JSObjectSetProperty(engine->ctx, obj, key, JSValueMakeNumber(engine->ctx, val), kJSPropertyAttributeNone, nullptr);
        JSStringRelease(key);
    };
    auto set_bool = [&](const char* name, bool val) {
        JSStringRef key = jsc_string_from_cstr(name);
        JSObjectSetProperty(engine->ctx, obj, key, JSValueMakeBoolean(engine->ctx, val), kJSPropertyAttributeNone, nullptr);
        JSStringRelease(key);
    };
    set_num("dev", (double)st.st_dev);
    set_num("ino", (double)st.st_ino);
    set_num("mode", (double)st.st_mode);
    set_num("nlink", (double)st.st_nlink);
    set_num("uid", (double)st.st_uid);
    set_num("gid", (double)st.st_gid);
    set_num("rdev", (double)st.st_rdev);
    set_num("size", (double)st.st_size);
    set_num("blksize", (double)st.st_blksize);
    set_num("blocks", (double)st.st_blocks);
    set_num("atimeMs", (double)st.st_atim.tv_sec * 1000.0 + st.st_atim.tv_nsec / 1000000.0);
    set_num("mtimeMs", (double)st.st_mtim.tv_sec * 1000.0 + st.st_mtim.tv_nsec / 1000000.0);
    set_num("ctimeMs", (double)st.st_ctim.tv_sec * 1000.0 + st.st_ctim.tv_nsec / 1000000.0);
    set_bool("isFile", S_ISREG(st.st_mode));
    set_bool("isDirectory", S_ISDIR(st.st_mode));
    set_bool("isSymbolicLink", S_ISLNK(st.st_mode));
    return obj;
}

klyron_jsc_value_t* klyron_jsc_fs_read_file(klyron_jsc_engine_t* engine, const char* path) {
    if (!engine || !path) return nullptr;
    int fd = open(path, O_RDONLY);
    if (fd < 0) {
        jsc_set_error(engine, std::string("fs.readFile: ") + std::strerror(errno));
        return nullptr;
    }
    struct stat st;
    if (fstat(fd, &st) < 0) {
        close(fd);
        jsc_set_error(engine, std::string("fs.readFile: ") + std::strerror(errno));
        return nullptr;
    }
    std::vector<char> buf(st.st_size);
    ssize_t total = 0;
    while (total < (ssize_t)st.st_size) {
        ssize_t r = read(fd, buf.data() + total, st.st_size - total);
        if (r <= 0) break;
        total += r;
    }
    close(fd);
    void* bytes = std::malloc(buf.size());
    if (!bytes) return nullptr;
    std::memcpy(bytes, buf.data(), buf.size());
    JSValueRef exc = nullptr;
    JSValueRef ab = JSObjectMakeArrayBufferWithBytesNoCopy(
        engine->ctx, bytes, buf.size(),
        [](void* p, void* c) { std::free(p); }, nullptr, &exc);
    if (exc || !ab) {
        std::free(bytes);
        jsc_capture_exception(engine, exc);
        return nullptr;
    }
    auto v = new klyron_jsc_value_t(engine->ctx, ab);
    v->protect();
    return v;
}

klyron_jsc_result_t klyron_jsc_fs_write_file(klyron_jsc_engine_t* engine, const char* path, klyron_jsc_value_t* data) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !path || !data) return result;
    if (!JSValueIsObject(engine->ctx, data->value)) {
        jsc_set_error(engine, "fs.writeFile: data must be an ArrayBuffer or TypedArray");
        return result;
    }
    JSObjectRef obj = (JSObjectRef)data->value;
    JSValueRef exc = nullptr;
    unsigned char* ptr = nullptr;
    size_t length = 0;
    JSTypedArrayType ta = JSValueGetTypedArrayType(engine->ctx, obj, &exc);
    if (ta != kJSTypedArrayTypeNone) {
        JSObjectRef buf = JSObjectGetTypedArrayBuffer(engine->ctx, obj, &exc);
        if (exc || !buf) return result;
        length = JSObjectGetArrayBufferByteLength(engine->ctx, buf, &exc);
        if (exc) return result;
        ptr = (unsigned char*)JSObjectGetArrayBufferBytesPtr(engine->ctx, buf, &exc);
    } else {
        length = JSObjectGetArrayBufferByteLength(engine->ctx, obj, &exc);
        if (exc) return result;
        ptr = (unsigned char*)JSObjectGetArrayBufferBytesPtr(engine->ctx, obj, &exc);
    }
    if (exc || !ptr) return result;
    int fd = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (fd < 0) {
        jsc_set_error(engine, std::string("fs.writeFile: ") + std::strerror(errno));
        return result;
    }
    ssize_t written = write(fd, ptr, length);
    close(fd);
    if (written < 0 || (size_t)written != length) {
        jsc_set_error(engine, std::string("fs.writeFile: short write"));
        return result;
    }
    result.success = true;
    return result;
}

klyron_jsc_value_t* klyron_jsc_fs_stat(klyron_jsc_engine_t* engine, const char* path) {
    if (!engine || !path) return nullptr;
    struct stat st;
    if (stat(path, &st) < 0) {
        jsc_set_error(engine, std::string("fs.stat: ") + std::strerror(errno));
        return nullptr;
    }
    JSObjectRef obj = make_stat_result(engine, st);
    auto v = new klyron_jsc_value_t(engine->ctx, obj);
    v->protect();
    return v;
}

klyron_jsc_result_t klyron_jsc_fs_mkdir(klyron_jsc_engine_t* engine, const char* path, int mode) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !path) return result;
    if (mkdir(path, (mode_t)(mode ? mode : 0755)) < 0) {
        jsc_set_error(engine, std::string("fs.mkdir: ") + std::strerror(errno));
        return result;
    }
    result.success = true;
    return result;
}

klyron_jsc_result_t klyron_jsc_fs_mkdir_p(klyron_jsc_engine_t* engine, const char* path, int mode) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !path) return result;
    std::string p(path);
    for (size_t i = 1; i < p.size(); i++) {
        if (p[i] == '/') {
            p[i] = '\0';
            mkdir(p.c_str(), (mode_t)(mode ? mode : 0755));
            p[i] = '/';
        }
    }
    if (mkdir(p.c_str(), (mode_t)(mode ? mode : 0755)) < 0 && errno != EEXIST) {
        jsc_set_error(engine, std::string("fs.mkdir-p: ") + std::strerror(errno));
        return result;
    }
    result.success = true;
    return result;
}

klyron_jsc_value_t* klyron_jsc_fs_readdir(klyron_jsc_engine_t* engine, const char* path) {
    if (!engine || !path) return nullptr;
    DIR* dir = opendir(path);
    if (!dir) {
        jsc_set_error(engine, std::string("fs.readdir: ") + std::strerror(errno));
        return nullptr;
    }
    JSValueRef exc = nullptr;
    JSObjectRef arr = JSObjectMakeArray(engine->ctx, 0, nullptr, &exc);
    if (exc) { closedir(dir); return nullptr; }
    int idx = 0;
    struct dirent* entry;
    while ((entry = readdir(dir)) != nullptr) {
        if (std::strcmp(entry->d_name, ".") == 0 || std::strcmp(entry->d_name, "..") == 0) continue;
        JSStringRef val_str = jsc_string_from_cstr(entry->d_name);
        JSValueRef val = JSValueMakeString(engine->ctx, val_str);
        JSStringRelease(val_str);
        JSStringRef idx_str = JSStringCreateWithUTF8CString(std::to_string(idx++).c_str());
        JSObjectSetProperty(engine->ctx, arr, idx_str, val, kJSPropertyAttributeNone, &exc);
        JSStringRelease(idx_str);
    }
    closedir(dir);
    auto v = new klyron_jsc_value_t(engine->ctx, arr);
    v->protect();
    return v;
}

klyron_jsc_result_t klyron_jsc_fs_unlink(klyron_jsc_engine_t* engine, const char* path) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !path) return result;
    if (unlink(path) < 0) {
        jsc_set_error(engine, std::string("fs.unlink: ") + std::strerror(errno));
        return result;
    }
    result.success = true;
    return result;
}

klyron_jsc_result_t klyron_jsc_fs_rmdir(klyron_jsc_engine_t* engine, const char* path) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !path) return result;
    if (rmdir(path) < 0) {
        jsc_set_error(engine, std::string("fs.rmdir: ") + std::strerror(errno));
        return result;
    }
    result.success = true;
    return result;
}

klyron_jsc_result_t klyron_jsc_fs_rename(klyron_jsc_engine_t* engine, const char* old_path, const char* new_path) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !old_path || !new_path) return result;
    if (rename(old_path, new_path) < 0) {
        jsc_set_error(engine, std::string("fs.rename: ") + std::strerror(errno));
        return result;
    }
    result.success = true;
    return result;
}

klyron_jsc_result_t klyron_jsc_fs_chmod(klyron_jsc_engine_t* engine, const char* path, int mode) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !path) return result;
    if (chmod(path, (mode_t)mode) < 0) {
        jsc_set_error(engine, std::string("fs.chmod: ") + std::strerror(errno));
        return result;
    }
    result.success = true;
    return result;
}

klyron_jsc_value_t* klyron_jsc_fs_realpath(klyron_jsc_engine_t* engine, const char* path) {
    if (!engine || !path) return nullptr;
    char resolved[PATH_MAX];
    if (!realpath(path, resolved)) {
        jsc_set_error(engine, std::string("fs.realpath: ") + std::strerror(errno));
        return nullptr;
    }
    return make_string(engine, resolved);
}

klyron_jsc_value_t* klyron_jsc_fs_exists(klyron_jsc_engine_t* engine, const char* path) {
    if (!engine || !path) return make_bool(engine, false);
    struct stat st;
    return make_bool(engine, stat(path, &st) == 0);
}

klyron_jsc_value_t* klyron_jsc_fs_lstat(klyron_jsc_engine_t* engine, const char* path) {
    if (!engine || !path) return nullptr;
    struct stat st;
    if (lstat(path, &st) < 0) {
        jsc_set_error(engine, std::string("fs.lstat: ") + std::strerror(errno));
        return nullptr;
    }
    JSObjectRef obj = make_stat_result(engine, st);
    auto v = new klyron_jsc_value_t(engine->ctx, obj);
    v->protect();
    return v;
}

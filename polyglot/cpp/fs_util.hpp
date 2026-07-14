#ifndef KLYRON_FS_UTIL_HPP
#define KLYRON_FS_UTIL_HPP

#include "klyron.hpp"
#include <filesystem>
#include <fstream>

namespace klyron {
namespace fs {

namespace fsys = std::filesystem;

inline Opt<String> read_file(const String &path) {
    std::ifstream f(path, std::ios::binary | std::ios::ate);
    if (!f) return std::nullopt;
    auto size = f.tellg();
    f.seekg(0);
    String content(size, '\0');
    f.read(content.data(), size);
    return content;
}

inline String read_file_string(const String &path) {
    auto content = read_file(path);
    return content.value_or("");
}

inline bool write_file(const String &path, const String &data) {
    fsys::create_directories(fsys::path(path).parent_path());
    std::ofstream f(path, std::ios::binary);
    if (!f) return false;
    f.write(data.data(), data.size());
    return true;
}

inline bool append_file(const String &path, const String &data) {
    std::ofstream f(path, std::ios::binary | std::ios::app);
    if (!f) return false;
    f.write(data.data(), data.size());
    return true;
}

inline bool exists(const String &path) {
    return fsys::exists(path);
}

inline bool mkdir(const String &path) {
    return fsys::create_directories(path);
}

inline bool remove(const String &path) {
    return fsys::remove_all(path) > 0;
}

inline bool copy(const String &src, const String &dst) {
    try {
        fsys::copy(src, dst, fsys::copy_options::recursive | fsys::copy_options::overwrite_existing);
        return true;
    } catch (...) { return false; }
}

inline bool rename(const String &old_path, const String &new_path) {
    try {
        fsys::rename(old_path, new_path);
        return true;
    } catch (...) { return false; }
}

inline Opt<FileInfo> stat(const String &path) {
    try {
        auto s = fsys::status(path);
        FileInfo info;
        info.path = path;
        info.size = fsys::is_regular_file(s) ? fsys::file_size(path) : 0;
        info.is_dir = fsys::is_directory(s);
        info.is_file = fsys::is_regular_file(s);
        info.modified = fsys::last_write_time(path).time_since_epoch().count();
        return info;
    } catch (...) { return std::nullopt; }
}

inline Vec<FileInfo> read_dir(const String &path) {
    Vec<FileInfo> entries;
    if (!fsys::exists(path) || !fsys::is_directory(path)) return entries;
    for (const auto &e : fsys::directory_iterator(path)) {
        auto info = stat(e.path().string());
        if (info) entries.push_back(*info);
    }
    return entries;
}

inline Vec<FileInfo> read_dir_recursive(const String &path) {
    Vec<FileInfo> entries;
    if (!fsys::exists(path)) return entries;
    for (const auto &e : fsys::recursive_directory_iterator(path)) {
        auto info = stat(e.path().string());
        if (info) entries.push_back(*info);
    }
    return entries;
}

inline Vec<String> list_files(const String &path) {
    Vec<String> files;
    if (!fsys::exists(path) || !fsys::is_directory(path)) return files;
    for (const auto &e : fsys::directory_iterator(path)) {
        files.push_back(e.path().filename().string());
    }
    return files;
}

inline int64_t file_size(const String &path) {
    try {
        if (fsys::is_regular_file(path))
            return static_cast<int64_t>(fsys::file_size(path));
    } catch (...) {}
    return -1;
}

inline bool is_dir(const String &path) {
    return fsys::is_directory(path);
}

inline bool is_file(const String &path) {
    return fsys::is_regular_file(path);
}

inline String cwd() {
    return fsys::current_path().string();
}

inline bool chdir(const String &path) {
    try {
        fsys::current_path(path);
        return true;
    } catch (...) { return false; }
}

}} // namespace klyron::fs

#endif

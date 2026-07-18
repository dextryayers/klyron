#include "klyron_jsc.h"
#include "cpp/impl/internal.h"
#include <cstring>
#include <cstdlib>
#include <string>
#include <vector>
#include <algorithm>
#include <climits>

static std::string normalize_path(const std::string& path) {
    if (path.empty()) return ".";
    std::vector<std::string> parts;
    bool absolute = path[0] == '/';
    size_t start = 0;
    while (start < path.size()) {
        while (start < path.size() && path[start] == '/') start++;
        if (start >= path.size()) break;
        size_t end = path.find('/', start);
        if (end == std::string::npos) end = path.size();
        std::string part = path.substr(start, end - start);
        if (part == "..") {
            if (!parts.empty() && parts.back() != "..") {
                parts.pop_back();
            } else if (!absolute) {
                parts.push_back("..");
            }
        } else if (part != "." && !part.empty()) {
            parts.push_back(part);
        }
        start = end + 1;
    }
    std::string result;
    if (absolute) result = "/";
    for (size_t i = 0; i < parts.size(); i++) {
        if (i > 0 || absolute) result += "/";
        result += parts[i];
    }
    return result.empty() ? "." : result;
}

klyron_jsc_string_result_t klyron_jsc_path_basename(klyron_jsc_engine_t* engine, const char* path_str) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !path_str) return result;
    std::string p = path_str;
    while (!p.empty() && p.back() == '/') p.pop_back();
    size_t slash = p.rfind('/');
    std::string base = (slash == std::string::npos) ? p : p.substr(slash + 1);
    jsc_set_string_result(&result, base);
    return result;
}

klyron_jsc_string_result_t klyron_jsc_path_dirname(klyron_jsc_engine_t* engine, const char* path_str) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !path_str) return result;
    std::string p = path_str;
    while (!p.empty() && p.back() == '/') p.pop_back();
    if (p.empty()) { jsc_set_string_result(&result, std::string(".")); return result; }
    size_t slash = p.rfind('/');
    if (slash == std::string::npos) { jsc_set_string_result(&result, std::string(".")); return result; }
    if (slash == 0) { jsc_set_string_result(&result, std::string("/")); return result; }
    jsc_set_string_result(&result, p.substr(0, slash));
    return result;
}

klyron_jsc_string_result_t klyron_jsc_path_extname(klyron_jsc_engine_t* engine, const char* path_str) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !path_str) return result;
    std::string p = path_str;
    while (!p.empty() && p.back() == '/') p.pop_back();
    size_t dot = p.rfind('.');
    size_t slash = p.rfind('/');
    if (dot == std::string::npos || (slash != std::string::npos && dot < slash) || dot == 0)
    { jsc_set_string_result(&result, std::string("")); return result; }
    jsc_set_string_result(&result, p.substr(dot));
    return result;
}

klyron_jsc_string_result_t klyron_jsc_path_join(klyron_jsc_engine_t* engine, klyron_jsc_value_t** parts, size_t count) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine) return result;
    std::string joined;
    for (size_t i = 0; i < count; i++) {
        if (!parts[i]) continue;
        if (!JSValueIsString(engine->ctx, parts[i]->value)) {
            JSValueRef exc = nullptr;
            JSStringRef str = JSValueToStringCopy(engine->ctx, parts[i]->value, &exc);
            if (str) {
                std::string s = jsc_string_to_std(str);
                JSStringRelease(str);
                if (!joined.empty() && !s.empty()) joined += "/";
                joined += s;
            }
            continue;
        }
        JSValueRef exc = nullptr;
        JSStringRef str = JSValueToStringCopy(engine->ctx, parts[i]->value, &exc);
        if (str) {
            std::string s = jsc_string_to_std(str);
            JSStringRelease(str);
            if (!joined.empty() && !s.empty()) joined += "/";
            joined += s;
        }
    }
    jsc_set_string_result(&result, normalize_path(joined));
    return result;
}

klyron_jsc_string_result_t klyron_jsc_path_resolve(klyron_jsc_engine_t* engine, klyron_jsc_value_t** parts, size_t count) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine) return result;
    std::string resolved;
    for (size_t i = 0; i < count; i++) {
        if (!parts[i]) continue;
        JSValueRef exc = nullptr;
        JSStringRef str = JSValueToStringCopy(engine->ctx, parts[i]->value, &exc);
        if (!str) continue;
        std::string s = jsc_string_to_std(str);
        JSStringRelease(str);
        if (s.empty()) continue;
        if (s[0] == '/') {
            resolved = s;
        } else {
            if (!resolved.empty() && resolved.back() != '/') resolved += "/";
            resolved += s;
        }
    }
    if (resolved.empty()) {
        char cwd[PATH_MAX];
        if (getcwd(cwd, sizeof(cwd))) resolved = cwd;
        else resolved = ".";
    }
    jsc_set_string_result(&result, normalize_path(resolved));
    return result;
}

klyron_jsc_string_result_t klyron_jsc_path_normalize(klyron_jsc_engine_t* engine, const char* path_str) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !path_str) return result;
    jsc_set_string_result(&result, normalize_path(std::string(path_str)));
    return result;
}

klyron_jsc_string_result_t klyron_jsc_path_relative(klyron_jsc_engine_t* engine, const char* from, const char* to_str) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !from || !to_str) return result;
    std::string f = normalize_path(from);
    std::string t = normalize_path(to_str);
    auto split = [](const std::string& s) -> std::vector<std::string> {
        std::vector<std::string> parts;
        size_t start = 0;
        while (start < s.size()) {
            if (s[start] == '/') { start++; continue; }
            size_t end = s.find('/', start);
            if (end == std::string::npos) end = s.size();
            parts.push_back(s.substr(start, end - start));
            start = end + 1;
        }
        return parts;
    };
    auto fa = split(f);
    auto ta = split(t);
    size_t common = 0;
    while (common < fa.size() && common < ta.size() && fa[common] == ta[common]) common++;
    std::string result_str;
    for (size_t i = common; i < fa.size(); i++) {
        if (!result_str.empty()) result_str += "/";
        result_str += "..";
    }
    for (size_t i = common; i < ta.size(); i++) {
        if (!result_str.empty() && result_str != "..") result_str += "/";
        else if (!result_str.empty()) result_str += "/";
        result_str += ta[i];
    }
    if (result_str.empty()) result_str = ".";
    jsc_set_string_result(&result, result_str);
    return result;
}

klyron_jsc_value_t* klyron_jsc_path_is_absolute(klyron_jsc_engine_t* engine, const char* path_str) {
    if (!engine || !path_str) {
        auto v = new klyron_jsc_value_t(engine->ctx, JSValueMakeBoolean(engine->ctx, false));
        v->protect();
        return v;
    }
    auto v = new klyron_jsc_value_t(engine->ctx, JSValueMakeBoolean(engine->ctx, path_str[0] == '/'));
    v->protect();
    return v;
}

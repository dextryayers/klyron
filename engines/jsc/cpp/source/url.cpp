#include "klyron_jsc.h"
#include "cpp/impl/internal.h"
#include <cstring>
#include <cstdlib>
#include <string>
#include <vector>
#include <algorithm>
#include <cctype>

struct ParsedURL {
    std::string scheme;
    std::string authority;
    std::string host;
    int port;
    std::string path;
    std::string query;
    std::string fragment;
    std::string userinfo;
};

static ParsedURL parse_url(const std::string& url_str) {
    ParsedURL result;
    result.port = -1;
    std::string s = url_str;
    size_t pos = 0;
    size_t scheme_end = s.find("://");
    if (scheme_end != std::string::npos) {
        result.scheme = s.substr(0, scheme_end);
        pos = scheme_end + 3;
    }
    size_t fragment_pos = s.find('#', pos);
    if (fragment_pos != std::string::npos) {
        result.fragment = s.substr(fragment_pos + 1);
        s = s.substr(0, fragment_pos);
    }
    size_t query_pos = s.find('?', pos);
    if (query_pos != std::string::npos) {
        result.query = s.substr(query_pos + 1);
        s = s.substr(0, query_pos);
    }
    size_t auth_end = s.find('/', pos);
    if (auth_end == std::string::npos) auth_end = s.size();
    result.authority = s.substr(pos, auth_end - pos);
    size_t at_pos = result.authority.find('@');
    if (at_pos != std::string::npos) {
        result.userinfo = result.authority.substr(0, at_pos);
        result.authority = result.authority.substr(at_pos + 1);
    }
    size_t colon_pos = result.authority.find(':');
    if (colon_pos != std::string::npos) {
        result.host = result.authority.substr(0, colon_pos);
        std::string port_str = result.authority.substr(colon_pos + 1);
        if (!port_str.empty()) result.port = std::stoi(port_str);
    } else {
        result.host = result.authority;
    }
    if (auth_end < s.size()) {
        result.path = s.substr(auth_end);
    } else {
        result.path = "/";
    }
    return result;
}

static bool is_absolute(const std::string& url_str) {
    return url_str.find("://") != std::string::npos || url_str.find("//") == 0;
}

static std::string resolve_path(const std::string& base, const std::string& rel) {
    if (rel.empty()) return base;
    if (rel[0] == '/') return rel;
    std::string combined = base;
    size_t last_slash = combined.rfind('/');
    if (last_slash != std::string::npos) {
        combined = combined.substr(0, last_slash + 1);
    } else {
        combined = "/";
    }
    combined += rel;
    std::vector<std::string> parts;
    size_t start = 0;
    while (start < combined.size()) {
        size_t end = combined.find('/', start);
        if (end == std::string::npos) end = combined.size();
        std::string part = combined.substr(start, end - start);
        if (part == "..") {
            if (!parts.empty()) parts.pop_back();
        } else if (!part.empty() && part != ".") {
            parts.push_back(part);
        }
        start = end + 1;
    }
    std::string result;
    for (const auto& p : parts) result += "/" + p;
    return result.empty() ? "/" : result;
}

klyron_jsc_value_t* klyron_jsc_url_parse(klyron_jsc_engine_t* engine, const char* url_str) {
    if (!engine || !url_str) return nullptr;
    ParsedURL parsed = parse_url(url_str);
    JSObjectRef obj = JSObjectMake(engine->ctx, nullptr, nullptr);
    auto set_prop = [&](const char* name, const std::string& val) {
        JSStringRef key = jsc_string_from_cstr(name);
        JSStringRef val_str = jsc_string_from_cstr(val.c_str());
        JSValueRef js_val = JSValueMakeString(engine->ctx, val_str);
        JSObjectSetProperty(engine->ctx, obj, key, js_val, kJSPropertyAttributeNone, nullptr);
        JSStringRelease(val_str);
        JSStringRelease(key);
    };
    set_prop("href", url_str);
    set_prop("protocol", parsed.scheme + ":");
    set_prop("hostname", parsed.host);
    if (parsed.port >= 0) set_prop("port", std::to_string(parsed.port));
    set_prop("pathname", parsed.path);
    set_prop("search", parsed.query.empty() ? "" : "?" + parsed.query);
    set_prop("hash", parsed.fragment.empty() ? "" : "#" + parsed.fragment);
    set_prop("host", parsed.authority);
    set_prop("origin", parsed.scheme + "://" + parsed.authority);
    set_prop("username", parsed.userinfo);
    auto v = new klyron_jsc_value_t(engine->ctx, obj);
    v->protect();
    return v;
}

klyron_jsc_string_result_t klyron_jsc_url_resolve(klyron_jsc_engine_t* engine, const char* base, const char* relative) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !base || !relative) return result;
    if (is_absolute(relative)) {
        jsc_set_string_result(&result, std::string(relative));
        return result;
    }
    ParsedURL base_parsed = parse_url(base);
    std::string resolved_scheme = base_parsed.scheme.empty() ? "https" : base_parsed.scheme;
    std::string resolved_authority = base_parsed.authority;
    std::string resolved_path;
    if (relative[0] == '/') {
        resolved_path = relative;
    } else {
        resolved_path = resolve_path(base_parsed.path, relative);
    }
    std::string full = resolved_scheme + "://" + resolved_authority + resolved_path;
    jsc_set_string_result(&result, full);
    return result;
}

klyron_jsc_value_t* klyron_jsc_url_format(klyron_jsc_engine_t* engine, klyron_jsc_value_t* url_obj) {
    if (!engine || !url_obj || !JSValueIsObject(engine->ctx, url_obj->value)) return nullptr;
    JSObjectRef obj = (JSObjectRef)url_obj->value;
    auto get_prop = [&](const char* name) -> std::string {
        JSStringRef key = jsc_string_from_cstr(name);
        JSValueRef exc = nullptr;
        JSValueRef val = JSObjectGetProperty(engine->ctx, obj, key, &exc);
        JSStringRelease(key);
        if (exc || !val) return "";
        JSStringRef str = JSValueToStringCopy(engine->ctx, val, &exc);
        if (!str) return "";
        std::string s = jsc_string_to_std(str);
        JSStringRelease(str);
        return s;
    };
    std::string protocol = get_prop("protocol");
    std::string host = get_prop("host");
    std::string pathname = get_prop("pathname");
    std::string search = get_prop("search");
    std::string hash = get_prop("hash");
    std::string url = protocol + "//" + host + pathname;
    if (!search.empty()) url += search;
    if (!hash.empty()) url += hash;
    JSStringRef jsstr = jsc_string_from_cstr(url.c_str());
    JSValueRef val = JSValueMakeString(engine->ctx, jsstr);
    JSStringRelease(jsstr);
    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

klyron_jsc_string_result_t klyron_jsc_url_domain_to_ascii(klyron_jsc_engine_t* engine, const char* domain) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !domain) return result;
    std::string ascii;
    for (size_t i = 0; domain[i]; i++) {
        if ((unsigned char)domain[i] > 127) {
            ascii += "xn--";
            for (size_t j = 0; domain[j]; j++) {
                if ((unsigned char)domain[j] > 127) {
                    char buf[8];
                    std::snprintf(buf, sizeof(buf), "%02x", (unsigned char)domain[j]);
                    ascii += buf;
                } else {
                    ascii += domain[j];
                }
            }
            jsc_set_string_result(&result, ascii);
            return result;
        }
    }
    jsc_set_string_result(&result, std::string(domain));
    return result;
}

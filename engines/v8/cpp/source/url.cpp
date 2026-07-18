#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstring>
#include <cstdlib>
#include <string>
#include <algorithm>

static char* strdup_new(const std::string& s) {
    char* p = static_cast<char*>(std::malloc(s.size() + 1));
    if (p) { std::memcpy(p, s.c_str(), s.size() + 1); }
    return p;
}

static void parse_url(const std::string& url, const std::string& base,
                      klyron_v8_url_t* out) {
    std::memset(out, 0, sizeof(*out));

    std::string target = url;
    if (target.empty()) target = base;
    if (target.empty()) return;

    size_t pos = 0;

    auto extract = [&](const std::string& prefix) -> std::string {
        if (target.substr(pos, prefix.size()) == prefix) {
            pos += prefix.size();
            return prefix;
        }
        return "";
    };

    std::string protocol = extract("https://");
    if (protocol.empty()) protocol = extract("http://");
    if (protocol.empty()) protocol = extract("ftp://");
    if (protocol.empty()) protocol = extract("file://");
    if (protocol.empty()) protocol = extract("wss://");
    if (protocol.empty()) protocol = extract("ws://");
    if (protocol.empty()) {
        size_t colon = target.find("://");
        if (colon != std::string::npos) {
            protocol = target.substr(0, colon + 3);
            pos = colon + 3;
        }
    }

    if (!protocol.empty()) {
        out->protocol = strdup_new(protocol);
    }

    std::string host;
    size_t slash = target.find('/', pos);
    size_t qmark = target.find('?', pos);
    size_t hash = target.find('#', pos);

    size_t host_end = target.size();
    if (slash != std::string::npos) host_end = std::min(host_end, slash);
    if (qmark != std::string::npos) host_end = std::min(host_end, qmark);
    if (hash != std::string::npos) host_end = std::min(host_end, hash);

    if (host_end > pos) {
        host = target.substr(pos, host_end - pos);
        pos = host_end;
    }

    if (!host.empty()) {
        out->host = strdup_new(host);
        size_t colon_pos = host.find(':');
        if (colon_pos != std::string::npos) {
            out->hostname = strdup_new(host.substr(0, colon_pos));
            out->port = strdup_new(host.substr(colon_pos + 1));
        } else {
            out->hostname = strdup_new(host);
        }
    }

    std::string path;
    if (slash != std::string::npos && pos == slash) {
        size_t path_end = target.size();
        if (qmark != std::string::npos) path_end = std::min(path_end, qmark);
        if (hash != std::string::npos) path_end = std::min(path_end, hash);
        path = target.substr(pos, path_end - pos);
        pos = path_end;
    }
    if (path.empty()) path = "/";
    out->pathname = strdup_new(path);

    if (qmark != std::string::npos && pos == qmark) {
        size_t search_end = target.size();
        if (hash != std::string::npos) search_end = std::min(search_end, hash);
        out->search = strdup_new(target.substr(pos, search_end - pos));
        pos = search_end;
    }

    if (hash != std::string::npos && pos == hash) {
        out->hash = strdup_new(target.substr(pos));
    }

    if (protocol == "file://" || protocol.empty()) {
        out->origin = strdup_new("null");
    } else if (!protocol.empty() && !host.empty()) {
        out->origin = strdup_new(protocol + host);
    }

    out->href = strdup_new(target);
}

klyron_v8_url_t* klyron_v8_url_parse(const char* url, const char* base) {
    if (!url) return nullptr;
    auto* result = static_cast<klyron_v8_url_t*>(std::malloc(sizeof(klyron_v8_url_t)));
    if (!result) return nullptr;
    parse_url(url, base ? base : "", result);
    return result;
}

void klyron_v8_url_dispose(klyron_v8_url_t* url) {
    if (!url) return;
    std::free(url->href);
    std::free(url->protocol);
    std::free(url->hostname);
    std::free(url->port);
    std::free(url->pathname);
    std::free(url->search);
    std::free(url->hash);
    std::free(url->host);
    std::free(url->origin);
    std::free(url);
}

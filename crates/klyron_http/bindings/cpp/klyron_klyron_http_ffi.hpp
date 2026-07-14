#ifndef KLYRON_KLYRON_HTTP_FFI_HPP
#define KLYRON_KLYRON_HTTP_FFI_HPP

extern "C" {
    const char* klyron_http_version();
}

inline const char* klyron_http_version_str() { return klyron_http_version(); }

#endif

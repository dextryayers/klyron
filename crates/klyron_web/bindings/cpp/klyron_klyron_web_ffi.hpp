#ifndef KLYRON_KLYRON_WEB_FFI_HPP
#define KLYRON_KLYRON_WEB_FFI_HPP

extern "C" {
    const char* klyron_web_version();
}

inline const char* klyron_web_version_str() { return klyron_web_version(); }

#endif

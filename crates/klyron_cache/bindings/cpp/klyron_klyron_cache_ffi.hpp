#ifndef KLYRON_KLYRON_CACHE_FFI_HPP
#define KLYRON_KLYRON_CACHE_FFI_HPP

extern "C" {
    const char* klyron_cache_version();
}

inline const char* klyron_cache_version_str() { return klyron_cache_version(); }

#endif

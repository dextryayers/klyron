#ifndef KLYRON_KLYRON_LOADER_FFI_HPP
#define KLYRON_KLYRON_LOADER_FFI_HPP

extern "C" {
    const char* klyron_loader_version();
}

inline const char* klyron_loader_version_str() { return klyron_loader_version(); }

#endif

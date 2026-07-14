#ifndef KLYRON_KLYRON_PROCESS_FFI_HPP
#define KLYRON_KLYRON_PROCESS_FFI_HPP

extern "C" {
    const char* klyron_process_version();
}

inline const char* klyron_process_version_str() { return klyron_process_version(); }

#endif

#ifndef KLYRON_KLYRON_LOGGER_FFI_HPP
#define KLYRON_KLYRON_LOGGER_FFI_HPP

extern "C" {
    const char* klyron_logger_version();
}

inline const char* klyron_logger_version_str() { return klyron_logger_version(); }

#endif

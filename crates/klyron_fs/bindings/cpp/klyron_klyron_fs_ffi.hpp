#ifndef KLYRON_KLYRON_FS_FFI_HPP
#define KLYRON_KLYRON_FS_FFI_HPP

extern "C" {
    const char* klyron_fs_version();
}

inline const char* klyron_fs_version_str() { return klyron_fs_version(); }

#endif

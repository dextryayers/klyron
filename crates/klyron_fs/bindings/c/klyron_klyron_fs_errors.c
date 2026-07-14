#include "klyron_klyron_fs_errors.h"

const char* klyron_klyron_fs_error_string(klyron_klyron_fs_error_t err) {
    switch(err) {
        case KLYRON_KLYRON_FS_OK: return "ok";
        case KLYRON_KLYRON_FS_ERR_GENERIC: return "generic error";
        default: return "unknown";
    }
}

#include "shell_errors.h"

const char* klyron_shell_error_string(int err) {
    switch (err) {
        case KLYRON_SHELL_OK: return "ok";
        case KLYRON_SHELL_ERR_INIT: return "init failed";
        case KLYRON_SHELL_ERR_OPERATION: return "operation failed";
        default: return "unknown error";
    }
}

#include "template_errors.h"

const char* klyron_template_error_string(int err) {
    switch (err) {
        case KLYRON_TEMPLATE_OK: return "ok";
        case KLYRON_TEMPLATE_ERR_INIT: return "init failed";
        case KLYRON_TEMPLATE_ERR_OPERATION: return "operation failed";
        default: return "unknown error";
    }
}

#include "deploy_errors.h"

const char* klyron_deploy_error_string(int err) {
    switch (err) {
        case KLYRON_DEPLOY_OK: return "ok";
        case KLYRON_DEPLOY_ERR_INIT: return "init failed";
        case KLYRON_DEPLOY_ERR_OPERATION: return "operation failed";
        default: return "unknown error";
    }
}

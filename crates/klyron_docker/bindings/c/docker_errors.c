#include "docker_errors.h"

const char* klyron_docker_error_string(int err) {
    switch (err) {
        case KLYRON_DOCKER_OK: return "ok";
        case KLYRON_DOCKER_ERR_INIT: return "init failed";
        case KLYRON_DOCKER_ERR_OPERATION: return "operation failed";
        default: return "unknown error";
    }
}

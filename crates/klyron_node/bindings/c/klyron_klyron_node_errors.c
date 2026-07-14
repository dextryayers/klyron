#include "klyron_klyron_node_errors.h"

const char* klyron_klyron_node_error_string(klyron_klyron_node_error_t err) {
    switch(err) {
        case KLYRON_KLYRON_NODE_OK: return "ok";
        case KLYRON_KLYRON_NODE_ERR_GENERIC: return "generic error";
        default: return "unknown";
    }
}

#ifndef KLYRON_KLYRON_NODE_ERRORS_H
#define KLYRON_KLYRON_NODE_ERRORS_H

typedef enum { KLYRON_KLYRON_NODE_OK = 0, KLYRON_KLYRON_NODE_ERR_GENERIC = -1 } klyron_klyron_node_error_t;
const char* klyron_klyron_node_error_string(klyron_klyron_node_error_t err);

#endif

#ifndef KLYRON_REGISTRY_ERRORS_H
#define KLYRON_REGISTRY_ERRORS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    KLYRON_REGISTRY_OK = 0,
    KLYRON_REGISTRY_ERR_FAILED = -1,
} klyron_registry_error_t;

const char* klyron_registry_error_string(klyron_registry_error_t err);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_REGISTRY_ERRORS_H */

#ifndef KLYRON_NAPI_ERRORS_H
#define KLYRON_NAPI_ERRORS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    KLYRON_NAPI_OK = 0,
    KLYRON_NAPI_ERR_MODULE_NOT_FOUND = -1,
    KLYRON_NAPI_ERR_LOAD_FAILED = -2,
    KLYRON_NAPI_ERR_VERSION_MISMATCH = -3,
    KLYRON_NAPI_ERR_INVALID_ARGUMENT = -4,
} klyron_napi_error_t;

const char* klyron_napi_error_string(klyron_napi_error_t err);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_NAPI_ERRORS_H */

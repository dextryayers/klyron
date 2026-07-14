#ifndef KLYRON_TRANSPILER_ERRORS_H
#define KLYRON_TRANSPILER_ERRORS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    KLYRON_TRANSPILER_OK = 0,
    KLYRON_TRANSPILER_ERR_FAILED = -1,
} klyron_transpiler_error_t;

const char* klyron_transpiler_error_string(klyron_transpiler_error_t err);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_TRANSPILER_ERRORS_H */

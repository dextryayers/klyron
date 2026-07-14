#ifndef KLYRON_LINTER_ERRORS_H
#define KLYRON_LINTER_ERRORS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    KLYRON_LINTER_OK = 0,
    KLYRON_LINTER_ERR_FAILED = -1,
} klyron_linter_error_t;

const char* klyron_linter_error_string(klyron_linter_error_t err);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_LINTER_ERRORS_H */

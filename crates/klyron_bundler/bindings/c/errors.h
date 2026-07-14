#ifndef KLYRON_BUNDLER_ERRORS_H
#define KLYRON_BUNDLER_ERRORS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    KLYRON_BUNDLER_OK = 0,
    KLYRON_BUNDLER_ERR_FAILED = -1,
} klyron_bundler_error_t;

const char* klyron_bundler_error_string(klyron_bundler_error_t err);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_BUNDLER_ERRORS_H */

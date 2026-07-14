#ifndef KLYRON_FORMATTER_ERRORS_H
#define KLYRON_FORMATTER_ERRORS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    KLYRON_FORMATTER_OK = 0,
    KLYRON_FORMATTER_ERR_FAILED = -1,
} klyron_formatter_error_t;

const char* klyron_formatter_error_string(klyron_formatter_error_t err);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_FORMATTER_ERRORS_H */

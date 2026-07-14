#ifndef KLYRON_WATCHER_ERRORS_H
#define KLYRON_WATCHER_ERRORS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    KLYRON_WATCHER_OK = 0,
    KLYRON_WATCHER_ERR_FAILED = -1,
} klyron_watcher_error_t;

const char* klyron_watcher_error_string(klyron_watcher_error_t err);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_WATCHER_ERRORS_H */

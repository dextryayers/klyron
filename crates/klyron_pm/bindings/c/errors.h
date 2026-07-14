#ifndef KLYRON_PM_ERRORS_H
#define KLYRON_PM_ERRORS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    KLYRON_PM_OK = 0,
    KLYRON_PM_ERR_FAILED = -1,
} klyron_pm_error_t;

const char* klyron_pm_error_string(klyron_pm_error_t err);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_PM_ERRORS_H */

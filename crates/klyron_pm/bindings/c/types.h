#ifndef KLYRON_PM_TYPES_H
#define KLYRON_PM_TYPES_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    bool enabled;
    bool verbose;
} klyron_pm_config_t;

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_PM_TYPES_H */

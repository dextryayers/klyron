#ifndef KLYRON_PM_H
#define KLYRON_PM_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    bool enabled;
    bool verbose;
} klyron_pm_config_t;

uint32_t klyron_pm_version(void);
klyron_pm_config_t klyron_pm_config_default(void);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_PM_H */

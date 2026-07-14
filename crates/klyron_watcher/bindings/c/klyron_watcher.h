#ifndef KLYRON_WATCHER_H
#define KLYRON_WATCHER_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    bool enabled;
    bool verbose;
} klyron_watcher_config_t;

uint32_t klyron_watcher_version(void);
klyron_watcher_config_t klyron_watcher_config_default(void);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_WATCHER_H */

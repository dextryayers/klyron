#ifndef KLYRON_REGISTRY_H
#define KLYRON_REGISTRY_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    bool enabled;
    bool verbose;
} klyron_registry_config_t;

uint32_t klyron_registry_version(void);
klyron_registry_config_t klyron_registry_config_default(void);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_REGISTRY_H */

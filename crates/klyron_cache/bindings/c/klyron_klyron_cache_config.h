#ifndef KLYRON_KLYRON_CACHE_CONFIG_H
#define KLYRON_KLYRON_CACHE_CONFIG_H

#include <stdint.h>

typedef struct { uint64_t timeout_ms; } klyron_klyron_cache_config_t;
klyron_klyron_cache_config_t klyron_klyron_cache_config_default(void);

#endif

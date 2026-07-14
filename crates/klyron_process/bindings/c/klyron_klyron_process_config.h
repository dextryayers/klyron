#ifndef KLYRON_KLYRON_PROCESS_CONFIG_H
#define KLYRON_KLYRON_PROCESS_CONFIG_H

#include <stdint.h>

typedef struct { uint64_t timeout_ms; } klyron_klyron_process_config_t;
klyron_klyron_process_config_t klyron_klyron_process_config_default(void);

#endif

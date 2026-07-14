#ifndef KLYRON_TRANSPILER_H
#define KLYRON_TRANSPILER_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    bool enabled;
    bool verbose;
} klyron_transpiler_config_t;

uint32_t klyron_transpiler_version(void);
klyron_transpiler_config_t klyron_transpiler_config_default(void);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_TRANSPILER_H */

#ifndef KLYRON_LINTER_H
#define KLYRON_LINTER_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    bool enabled;
    bool verbose;
} klyron_linter_config_t;

uint32_t klyron_linter_version(void);
klyron_linter_config_t klyron_linter_config_default(void);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_LINTER_H */

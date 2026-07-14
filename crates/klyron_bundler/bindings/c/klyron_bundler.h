#ifndef KLYRON_BUNDLER_H
#define KLYRON_BUNDLER_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    bool enabled;
    bool verbose;
} klyron_bundler_config_t;

uint32_t klyron_bundler_version(void);
klyron_bundler_config_t klyron_bundler_config_default(void);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_BUNDLER_H */

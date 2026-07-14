#ifndef KLYRON_FORMATTER_H
#define KLYRON_FORMATTER_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    bool enabled;
    bool verbose;
} klyron_formatter_config_t;

uint32_t klyron_formatter_version(void);
klyron_formatter_config_t klyron_formatter_config_default(void);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_FORMATTER_H */

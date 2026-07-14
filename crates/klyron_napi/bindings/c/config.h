#ifndef KLYRON_NAPI_CONFIG_H
#define KLYRON_NAPI_CONFIG_H

#include "types.h"
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

klyron_napi_config_t* klyron_napi_config_default(void);
void klyron_napi_config_free(klyron_napi_config_t* config);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_NAPI_CONFIG_H */

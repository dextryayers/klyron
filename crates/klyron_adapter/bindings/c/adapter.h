#ifndef KLYRON_ADAPTER_H
#define KLYRON_ADAPTER_H

typedef struct {
    const char* version;
} klyron_adapter_config_t;

klyron_adapter_config_t* klyron_adapter_config_new(void);
void klyron_adapter_config_free(klyron_adapter_config_t* config);
const char* klyron_adapter_version(void);

#endif /* KLYRON_ADAPTER_H */

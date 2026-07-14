#ifndef KLYRON_CONFIG_H
#define KLYRON_CONFIG_H

typedef struct {
    const char* version;
} klyron_config_config_t;

klyron_config_config_t* klyron_config_config_new(void);
void klyron_config_config_free(klyron_config_config_t* config);
const char* klyron_config_version(void);

#endif /* KLYRON_CONFIG_H */

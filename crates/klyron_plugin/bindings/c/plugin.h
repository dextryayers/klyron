#ifndef KLYRON_PLUGIN_H
#define KLYRON_PLUGIN_H

typedef struct {
    const char* version;
} klyron_plugin_config_t;

klyron_plugin_config_t* klyron_plugin_config_new(void);
void klyron_plugin_config_free(klyron_plugin_config_t* config);
const char* klyron_plugin_version(void);

#endif /* KLYRON_PLUGIN_H */

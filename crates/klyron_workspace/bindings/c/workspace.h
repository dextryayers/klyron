#ifndef KLYRON_WORKSPACE_H
#define KLYRON_WORKSPACE_H

typedef struct {
    const char* version;
} klyron_workspace_config_t;

klyron_workspace_config_t* klyron_workspace_config_new(void);
void klyron_workspace_config_free(klyron_workspace_config_t* config);
const char* klyron_workspace_version(void);

#endif /* KLYRON_WORKSPACE_H */

#ifndef KLYRON_DEPLOY_H
#define KLYRON_DEPLOY_H

typedef struct {
    const char* version;
} klyron_deploy_config_t;

klyron_deploy_config_t* klyron_deploy_config_new(void);
void klyron_deploy_config_free(klyron_deploy_config_t* config);
const char* klyron_deploy_version(void);

#endif /* KLYRON_DEPLOY_H */

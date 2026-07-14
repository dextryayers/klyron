#ifndef KLYRON_DOCKER_H
#define KLYRON_DOCKER_H

typedef struct {
    const char* version;
} klyron_docker_config_t;

klyron_docker_config_t* klyron_docker_config_new(void);
void klyron_docker_config_free(klyron_docker_config_t* config);
const char* klyron_docker_version(void);

#endif /* KLYRON_DOCKER_H */

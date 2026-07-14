#ifndef KLYRON_COMPAT_H
#define KLYRON_COMPAT_H

typedef struct {
    const char* version;
} klyron_compat_config_t;

klyron_compat_config_t* klyron_compat_config_new(void);
void klyron_compat_config_free(klyron_compat_config_t* config);
const char* klyron_compat_version(void);

#endif /* KLYRON_COMPAT_H */

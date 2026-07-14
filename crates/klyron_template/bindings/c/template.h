#ifndef KLYRON_TEMPLATE_H
#define KLYRON_TEMPLATE_H

typedef struct {
    const char* version;
} klyron_template_config_t;

klyron_template_config_t* klyron_template_config_new(void);
void klyron_template_config_free(klyron_template_config_t* config);
const char* klyron_template_version(void);

#endif /* KLYRON_TEMPLATE_H */

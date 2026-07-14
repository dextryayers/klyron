#ifndef KLYRON_SHELL_H
#define KLYRON_SHELL_H

typedef struct {
    const char* version;
} klyron_shell_config_t;

klyron_shell_config_t* klyron_shell_config_new(void);
void klyron_shell_config_free(klyron_shell_config_t* config);
const char* klyron_shell_version(void);

#endif /* KLYRON_SHELL_H */

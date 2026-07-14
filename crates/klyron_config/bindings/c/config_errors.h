#ifndef KLYRON_CONFIG_ERRORS_H
#define KLYRON_CONFIG_ERRORS_H

#define KLYRON_CONFIG_OK 0
#define KLYRON_CONFIG_ERR_INIT -1
#define KLYRON_CONFIG_ERR_OPERATION -2

const char* klyron_config_error_string(int err);

#endif /* KLYRON_CONFIG_ERRORS_H */

#ifndef KLYRON_PLUGIN_ERRORS_H
#define KLYRON_PLUGIN_ERRORS_H

#define KLYRON_PLUGIN_OK 0
#define KLYRON_PLUGIN_ERR_INIT -1
#define KLYRON_PLUGIN_ERR_OPERATION -2

const char* klyron_plugin_error_string(int err);

#endif /* KLYRON_PLUGIN_ERRORS_H */

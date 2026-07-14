#ifndef KLYRON_ADAPTER_ERRORS_H
#define KLYRON_ADAPTER_ERRORS_H

#define KLYRON_ADAPTER_OK 0
#define KLYRON_ADAPTER_ERR_INIT -1
#define KLYRON_ADAPTER_ERR_OPERATION -2

const char* klyron_adapter_error_string(int err);

#endif /* KLYRON_ADAPTER_ERRORS_H */

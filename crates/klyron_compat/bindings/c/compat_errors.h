#ifndef KLYRON_COMPAT_ERRORS_H
#define KLYRON_COMPAT_ERRORS_H

#define KLYRON_COMPAT_OK 0
#define KLYRON_COMPAT_ERR_INIT -1
#define KLYRON_COMPAT_ERR_OPERATION -2

const char* klyron_compat_error_string(int err);

#endif /* KLYRON_COMPAT_ERRORS_H */

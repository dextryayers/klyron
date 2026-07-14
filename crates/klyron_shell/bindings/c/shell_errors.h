#ifndef KLYRON_SHELL_ERRORS_H
#define KLYRON_SHELL_ERRORS_H

#define KLYRON_SHELL_OK 0
#define KLYRON_SHELL_ERR_INIT -1
#define KLYRON_SHELL_ERR_OPERATION -2

const char* klyron_shell_error_string(int err);

#endif /* KLYRON_SHELL_ERRORS_H */

#ifndef KLYRON_WORKSPACE_ERRORS_H
#define KLYRON_WORKSPACE_ERRORS_H

#define KLYRON_WORKSPACE_OK 0
#define KLYRON_WORKSPACE_ERR_INIT -1
#define KLYRON_WORKSPACE_ERR_OPERATION -2

const char* klyron_workspace_error_string(int err);

#endif /* KLYRON_WORKSPACE_ERRORS_H */

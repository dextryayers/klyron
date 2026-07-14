#ifndef KLYRON_DEPLOY_ERRORS_H
#define KLYRON_DEPLOY_ERRORS_H

#define KLYRON_DEPLOY_OK 0
#define KLYRON_DEPLOY_ERR_INIT -1
#define KLYRON_DEPLOY_ERR_OPERATION -2

const char* klyron_deploy_error_string(int err);

#endif /* KLYRON_DEPLOY_ERRORS_H */

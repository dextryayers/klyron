#ifndef KLYRON_DOCKER_ERRORS_H
#define KLYRON_DOCKER_ERRORS_H

#define KLYRON_DOCKER_OK 0
#define KLYRON_DOCKER_ERR_INIT -1
#define KLYRON_DOCKER_ERR_OPERATION -2

const char* klyron_docker_error_string(int err);

#endif /* KLYRON_DOCKER_ERRORS_H */

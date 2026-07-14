#ifndef KLYRON_TEMPLATE_ERRORS_H
#define KLYRON_TEMPLATE_ERRORS_H

#define KLYRON_TEMPLATE_OK 0
#define KLYRON_TEMPLATE_ERR_INIT -1
#define KLYRON_TEMPLATE_ERR_OPERATION -2

const char* klyron_template_error_string(int err);

#endif /* KLYRON_TEMPLATE_ERRORS_H */

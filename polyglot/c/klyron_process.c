#include "klyron_process.h"
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>
#include <errno.h>

int klyron_process_exec(const char *cmd, char *const argv[])
{
    pid_t pid = fork();
    if (pid == 0) {
        execvp(cmd, argv);
        _exit(127);
    }
    if (pid < 0) return -1;

    int wstatus;
    if (waitpid(pid, &wstatus, 0) < 0) return -1;
    if (WIFEXITED(wstatus)) return WEXITSTATUS(wstatus);
    return -1;
}

klyron_process_result_t klyron_process_capture(const char *cmd)
{
    klyron_process_result_t result = {0};
    FILE *fp = popen(cmd, "r");
    if (!fp) return result;

    size_t capacity = 4096;
    result.stdout_data = malloc(capacity);
    if (!result.stdout_data) {
        pclose(fp);
        return result;
    }

    size_t total = 0;
    int ch;
    while ((ch = fgetc(fp)) != EOF) {
        if (total + 1 >= capacity) {
            capacity *= 2;
            char *tmp = realloc(result.stdout_data, capacity);
            if (!tmp) {
                free(result.stdout_data);
                result.stdout_data = NULL;
                pclose(fp);
                return result;
            }
            result.stdout_data = tmp;
        }
        result.stdout_data[total++] = (char)ch;
    }
    result.stdout_data[total] = '\0';

    int status = pclose(fp);
    if (WIFEXITED(status)) {
        result.exit_code = WEXITSTATUS(status);
        result.success = result.exit_code == 0;
    }

    result.stderr_data = NULL;
    return result;
}

void klyron_process_free_result(klyron_process_result_t *result)
{
    if (!result) return;
    free(result->stdout_data);
    free(result->stderr_data);
    result->stdout_data = NULL;
    result->stderr_data = NULL;
    result->exit_code = 0;
    result->success = false;
}

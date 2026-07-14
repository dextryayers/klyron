#include "klyron_process.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <signal.h>
#include <sys/wait.h>
#include <errno.h>

klyron_process_result_t klyron_process_exec(const char *cmd) {
    klyron_process_result_t result = {NULL, NULL, -1, false};
    FILE *fp = popen(cmd, "r");
    if (!fp) return result;

    char buf[4096];
    size_t cap = 4096, len = 0;
    result.stdout_data = (char *)malloc(cap);
    if (!result.stdout_data) { pclose(fp); return result; }
    result.stdout_data[0] = '\0';

    while (fgets(buf, sizeof(buf), fp)) {
        size_t blen = strlen(buf);
        if (len + blen + 1 > cap) {
            cap *= 2;
            result.stdout_data = (char *)realloc(result.stdout_data, cap);
        }
        memcpy(result.stdout_data + len, buf, blen + 1);
        len += blen;
    }
    result.exit_code = pclose(fp);
    result.success = (result.exit_code == 0);
    return result;
}

klyron_process_result_t klyron_process_exec_args(const char *cmd, char *const argv[]) {
    klyron_process_result_t result = {NULL, NULL, -1, false};
    int pipefd[2];
    if (pipe(pipefd) < 0) return result;

    pid_t pid = fork();
    if (pid < 0) { close(pipefd[0]); close(pipefd[1]); return result; }

    if (pid == 0) {
        close(pipefd[0]);
        dup2(pipefd[1], STDOUT_FILENO);
        close(pipefd[1]);
        execvp(cmd, argv);
        _exit(127);
    }

    close(pipefd[1]);
    char buf[4096];
    size_t cap = 4096, len = 0;
    result.stdout_data = (char *)malloc(cap);
    if (!result.stdout_data) { close(pipefd[0]); return result; }
    result.stdout_data[0] = '\0';

    ssize_t n;
    while ((n = read(pipefd[0], buf, sizeof(buf) - 1)) > 0) {
        buf[n] = '\0';
        if (len + (size_t)n + 1 > cap) {
            cap *= 2;
            result.stdout_data = (char *)realloc(result.stdout_data, cap);
        }
        memcpy(result.stdout_data + len, buf, (size_t)n + 1);
        len += (size_t)n;
    }
    close(pipefd[0]);

    int status;
    waitpid(pid, &status, 0);
    if (WIFEXITED(status)) {
        result.exit_code = WEXITSTATUS(status);
        result.success = (result.exit_code == 0);
    }
    return result;
}

void klyron_process_free_result(klyron_process_result_t *r) {
    free(r->stdout_data);
    free(r->stderr_data);
    r->stdout_data = NULL;
    r->stderr_data = NULL;
}

bool klyron_process_which(const char *program, char *out, size_t out_size) {
    char cmd[512];
    snprintf(cmd, sizeof(cmd), "which %s", program);
    klyron_process_result_t r = klyron_process_exec(cmd);
    if (r.success && r.stdout_data) {
        char *nl = strchr(r.stdout_data, '\n');
        if (nl) *nl = '\0';
        strncpy(out, r.stdout_data, out_size - 1);
        out[out_size - 1] = '\0';
        klyron_process_free_result(&r);
        return true;
    }
    klyron_process_free_result(&r);
    return false;
}

int klyron_process_spawn(const char *cmd, char *const argv[]) {
    pid_t pid = fork();
    if (pid < 0) return -1;
    if (pid == 0) {
        execvp(cmd, argv);
        _exit(127);
    }
    return (int)pid;
}

bool klyron_process_kill(int pid, int sig) {
    return kill((pid_t)pid, sig) == 0;
}

bool klyron_process_is_running(int pid) {
    return kill((pid_t)pid, 0) == 0;
}

klyron_process_result_t klyron_process_exec_with_stdin(const char *cmd, const char *stdin_data) {
    klyron_process_result_t result = {NULL, NULL, -1, false};
    int out_pipe[2], in_pipe[2];
    if (pipe(out_pipe) < 0 || pipe(in_pipe) < 0) {
        if (out_pipe[0]) { close(out_pipe[0]); close(out_pipe[1]); }
        return result;
    }

    pid_t pid = fork();
    if (pid < 0) { close(out_pipe[0]); close(out_pipe[1]); close(in_pipe[0]); close(in_pipe[1]); return result; }

    if (pid == 0) {
        close(out_pipe[0]);
        dup2(out_pipe[1], STDOUT_FILENO);
        close(out_pipe[1]);
        close(in_pipe[1]);
        dup2(in_pipe[0], STDIN_FILENO);
        close(in_pipe[0]);
        execl("/bin/sh", "sh", "-c", cmd, NULL);
        _exit(127);
    }

    close(out_pipe[1]);
    close(in_pipe[0]);

    if (stdin_data) {
        write(in_pipe[1], stdin_data, strlen(stdin_data));
    }
    close(in_pipe[1]);

    char buf[4096];
    size_t cap = 4096, len = 0;
    result.stdout_data = (char *)malloc(cap);
    if (!result.stdout_data) { close(out_pipe[0]); return result; }
    result.stdout_data[0] = '\0';

    ssize_t n;
    while ((n = read(out_pipe[0], buf, sizeof(buf) - 1)) > 0) {
        buf[n] = '\0';
        if (len + (size_t)n + 1 > cap) {
            cap *= 2;
            result.stdout_data = (char *)realloc(result.stdout_data, cap);
        }
        memcpy(result.stdout_data + len, buf, (size_t)n + 1);
        len += (size_t)n;
    }
    close(out_pipe[0]);

    int status;
    waitpid(pid, &status, 0);
    if (WIFEXITED(status)) {
        result.exit_code = WEXITSTATUS(status);
        result.success = (result.exit_code == 0);
    }
    return result;
}

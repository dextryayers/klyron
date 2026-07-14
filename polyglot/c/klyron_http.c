#include "klyron_http.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <netdb.h>
#include <sys/socket.h>
#include <errno.h>

static int parse_url(const char *url, char *host, size_t hostlen,
                     char *port, size_t portlen, const char **path)
{
    const char *p = url;
    if (strncmp(p, "http://", 7) == 0) p += 7;
    else if (strncmp(p, "https://", 8) == 0) p += 8;

    size_t i = 0;
    while (p[i] && p[i] != ':' && p[i] != '/' && p[i] != '?') i++;

    if (i == 0 || i >= hostlen) return -1;
    memcpy(host, p, i);
    host[i] = '\0';

    if (p[i] == ':') {
        p += i + 1;
        i = 0;
        while (p[i] && p[i] != '/' && p[i] != '?') i++;
        if (i == 0 || i >= portlen) return -1;
        memcpy(port, p, i);
        port[i] = '\0';
    } else {
        snprintf(port, portlen, "80");
    }

    p += i;
    if (*p == '\0') *path = "/";
    else *path = p;
    return 0;
}

static klyron_response_t *http_request(const char *url, const char *method,
                                       const char *body, const char *content_type)
{
    if (!url) return NULL;

    char host[256] = {0};
    char port[16] = {0};
    const char *path = NULL;

    if (parse_url(url, host, sizeof(host), port, sizeof(port), &path) != 0)
        return NULL;

    struct addrinfo hints = {0};
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;

    struct addrinfo *ai = NULL;
    if (getaddrinfo(host, port, &hints, &ai) != 0 || !ai) return NULL;

    int fd = socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol);
    if (fd < 0) {
        freeaddrinfo(ai);
        return NULL;
    }

    if (connect(fd, ai->ai_addr, ai->ai_addrlen) < 0) {
        close(fd);
        freeaddrinfo(ai);
        return NULL;
    }
    freeaddrinfo(ai);

    char req[8192];
    int n;
    if (body && content_type) {
        n = snprintf(req, sizeof(req),
            "%s %s HTTP/1.1\r\n"
            "Host: %s\r\n"
            "Content-Type: %s\r\n"
            "Content-Length: %zu\r\n"
            "Connection: close\r\n"
            "\r\n"
            "%s",
            method, path, host, content_type, strlen(body), body);
    } else {
        n = snprintf(req, sizeof(req),
            "%s %s HTTP/1.1\r\n"
            "Host: %s\r\n"
            "Connection: close\r\n"
            "\r\n",
            method, path, host);
    }

    if (n < 0 || (size_t)n >= sizeof(req)) {
        close(fd);
        return NULL;
    }

    size_t total = 0;
    while (total < (size_t)n) {
        ssize_t w = write(fd, req + total, (size_t)(n - total));
        if (w <= 0) break;
        total += (size_t)w;
    }

    char buf[16384];
    size_t resp_len = 0;
    size_t resp_cap = sizeof(buf);
    char *resp = malloc(resp_cap);
    if (!resp) { close(fd); return NULL; }

    ssize_t r;
    while ((r = read(fd, resp + resp_len, resp_cap - resp_len - 1)) > 0) {
        resp_len += (size_t)r;
        if (resp_len + 256 >= resp_cap) {
            resp_cap *= 2;
            char *tmp = realloc(resp, resp_cap);
            if (!tmp) { free(resp); close(fd); return NULL; }
            resp = tmp;
        }
    }
    close(fd);
    resp[resp_len] = '\0';

    char *header_end = strstr(resp, "\r\n\r\n");
    if (!header_end) { free(resp); return NULL; }

    size_t header_len = (size_t)(header_end - resp);
    char *body_start = header_end + 4;
    size_t body_len = resp_len - header_len - 4;

    klyron_response_t *result = calloc(1, sizeof(klyron_response_t));
    if (!result) { free(resp); return NULL; }

    char status_line[512];
    const char *nl = strchr(resp, '\r');
    if (nl) {
        size_t sl = (size_t)(nl - resp);
        if (sl >= sizeof(status_line)) sl = sizeof(status_line) - 1;
        memcpy(status_line, resp, sl);
        status_line[sl] = '\0';

        if (sscanf(status_line, "HTTP/%*d.%*d %d", &result->status) == 1) {
            const char *sp = strchr(status_line, ' ');
            if (sp) {
                sp = strchr(sp + 1, ' ');
                if (sp) result->status_text = strdup(sp + 1);
            }
        }
    }

    if (body_len > 0) {
        result->body = malloc(body_len + 1);
        if (result->body) {
            memcpy(result->body, body_start, body_len);
            result->body[body_len] = '\0';
        }
    }

    free(resp);
    return result;
}

klyron_response_t *klyron_http_get(const char *url)
{
    return http_request(url, "GET", NULL, NULL);
}

klyron_response_t *klyron_http_post(const char *url, const char *body,
                                    const char *content_type)
{
    return http_request(url, "POST", body, content_type);
}

void klyron_http_free_response(klyron_response_t *resp)
{
    if (!resp) return;
    free(resp->status_text);
    free(resp->body);
    if (resp->headers) {
        for (size_t i = 0; i < resp->headers_len; i++)
            free(resp->headers[i]);
        free(resp->headers);
    }
    free(resp);
}

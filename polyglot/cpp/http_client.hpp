#ifndef KLYRON_HTTP_CLIENT_HPP
#define KLYRON_HTTP_CLIENT_HPP

#include "klyron.hpp"
#include <curl/curl.h>

namespace klyron {

class HttpClient {
public:
    static HttpResponse get(const String &url) {
        return request("GET", url, "", "");
    }

    static HttpResponse post(const String &url, const String &body, const String &content_type = "application/json") {
        return request("POST", url, body, content_type);
    }

    static HttpResponse put(const String &url, const String &body, const String &content_type = "application/json") {
        return request("PUT", url, body, content_type);
    }

    static HttpResponse del(const String &url) {
        return request("DELETE", url, "", "");
    }

    static HttpResponse patch(const String &url, const String &body, const String &content_type = "application/json") {
        return request("PATCH", url, body, content_type);
    }

    static HttpResponse head(const String &url) {
        return request("HEAD", url, "", "");
    }

    static HttpResponse request(const String &method, const String &url,
                                const String &body = "", const String &content_type = "") {
        HttpResponse resp;
        auto *curl = curl_easy_init();
        if (!curl) return resp;

        curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
        curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, method.c_str());
        curl_easy_setopt(curl, CURLOPT_TIMEOUT, 30L);
        curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);
        curl_easy_setopt(curl, CURLOPT_MAXREDIRS, 10L);

        String response_body;
        String response_headers;
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_callback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response_body);
        curl_easy_setopt(curl, CURLOPT_HEADERDATA, &response_headers);
        curl_easy_setopt(curl, CURLOPT_HEADERFUNCTION, write_callback);

        struct curl_slist *headers = nullptr;
        if (!body.empty()) {
            curl_easy_setopt(curl, CURLOPT_POSTFIELDS, body.c_str());
            curl_easy_setopt(curl, CURLOPT_POSTFIELDSIZE, (long)body.size());
            if (!content_type.empty()) {
                headers = curl_slist_append(headers, ("Content-Type: " + content_type).c_str());
            }
        }
        if (headers) curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);

        CURLcode res = curl_easy_perform(curl);
        if (res == CURLE_OK) {
            curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &resp.status);
            resp.body = response_body;
            resp.status_text = (resp.status == 200) ? "OK" : "Error";

            std::istringstream hstream(response_headers);
            String line;
            while (std::getline(hstream, line)) {
                auto colon = line.find(':');
                if (colon != String::npos) {
                    auto key = line.substr(0, colon);
                    auto val = line.substr(colon + 1);
                    val.erase(0, val.find_first_not_of(" \t\r\n"));
                    val.erase(val.find_last_not_of(" \t\r\n") + 1);
                    resp.headers[key] = val;
                }
            }
        } else {
            resp.status = 0;
            resp.status_text = curl_easy_strerror(res);
        }
        curl_slist_free_all(headers);
        curl_easy_cleanup(curl);
        return resp;
    }

    static Opt<String> get_body(const String &url) {
        auto resp = get(url);
        if (resp.ok()) return resp.body;
        return std::nullopt;
    }

    template<typename T>
    static Opt<T> get_json(const String &url) {
        auto resp = get(url);
        if (resp.ok()) return json::parse(resp.body);
        return std::nullopt;
    }

private:
    static size_t write_callback(void *contents, size_t size, size_t nmemb, void *userp) {
        size_t total = size * nmemb;
        static_cast<String *>(userp)->append(static_cast<char *>(contents), total);
        return total;
    }
};

} // namespace klyron

#endif

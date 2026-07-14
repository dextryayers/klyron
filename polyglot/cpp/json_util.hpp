#ifndef KLYRON_JSON_UTIL_HPP
#define KLYRON_JSON_UTIL_HPP

#include "klyron.hpp"
#include <nlohmann/json.hpp>

namespace klyron {

using json = nlohmann::json;

class JsonUtil {
public:
    static json parse(const String &text) {
        return json::parse(text, nullptr, false);
    }

    static String stringify(const json &j, bool pretty = false) {
        return pretty ? j.dump(2) : j.dump();
    }

    static bool is_valid(const String &text) {
        return json::accept(text);
    }

    static json merge(const json &a, const json &b) {
        json result = a;
        result.update(b);
        return result;
    }

    static Opt<String> get_string(const json &j, const String &key) {
        if (j.contains(key) && j[key].is_string())
            return j[key].get<String>();
        return std::nullopt;
    }

    static Opt<int> get_int(const json &j, const String &key) {
        if (j.contains(key) && j[key].is_number_integer())
            return j[key].get<int>();
        return std::nullopt;
    }
};

} // namespace klyron

#endif

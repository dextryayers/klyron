#include "errors.hpp"
#include <string>

namespace klyron {

std::string klyron_watcher_error_string(int code) {
    switch (code) {
        case 0: return "ok";
        default: return "unknown error";
    }
}

}

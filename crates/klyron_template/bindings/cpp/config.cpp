#include "config.hpp"

namespace klyron::_template {

TemplateConfigBuilder& TemplateConfigBuilder::with_version(const std::string& v) {
    config_.version = v;
    return *this;
}

TemplateConfig TemplateConfigBuilder::build() {
    return config_;
}

}

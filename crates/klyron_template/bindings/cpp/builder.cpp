#include "builder.hpp"

namespace klyron::_template {

TemplateBuilder::TemplateBuilder() : config_{} {}

TemplateBuilder& TemplateBuilder::set_config(const TemplateConfig& cfg) {
    config_ = cfg;
    return *this;
}

TemplateConfig TemplateBuilder::config() const {
    return config_;
}

}

#pragma once
#include "types.hpp"
#include "config.hpp"

namespace klyron::_template {

class TemplateBuilder {
public:
    TemplateBuilder();
    TemplateBuilder& set_config(const TemplateConfig& cfg);
    TemplateConfig config() const;
private:
    TemplateConfig config_;
};

}

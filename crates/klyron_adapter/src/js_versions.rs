use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsFrameworkCategory {
    FrontendFramework,
    BackendFramework,
    FullstackFramework,
    MetaFramework,
    StaticSiteGenerator,
    ApiFramework,
    PolyglotFramework,
}

pub struct JsVersions;

impl JsVersions {
    pub fn latest(framework: &str) -> &'static str {
        match framework {
            "react" => "19.1.0",
            "vue" => "3.6.0",
            "next" => "15.3.0",
            "astro" => "5.4.0",
            "express" => "5.1.0",
            "fastify" => "5.2.0",
            "hono" => "4.7.0",
            "nuxt" => "3.16.0",
            "sveltekit" => "2.18.0",
            "remix" => "2.16.0",
            "angular" => "19.2.0",
            "nestjs" => "11.0.0",
            "adonis" => "6.4.0",
            "koa" => "2.16.0",
            "hapi" => "21.4.0",
            "solid" => "1.9.0",
            "qwik" => "1.12.0",
            "preact" => "10.26.0",
            "lit" => "3.2.0",
            "svelte" => "5.20.0",
            "trpc" => "11.0.0",
            "graphql" => "5.12.0",
            "gatsby" => "5.14.0",
            "laravel" => "13.0",
            "django" => "5.2.0",
            "rails" => "8.0.0",
            "tanstack-router" => "1.60.0",
            "remotion" => "4.0.0",
            "fresh" => "1.7.0",
            "eleventy" => "3.0.0",
            "jekyll" => "4.3.0",
            "elysia" => "1.1.0",
            "stoker" => "0.1.0",
            "sanic" => "24.12.0",
            "gin" => "1.10.0",
            "fastapi" => "0.115.0",
            _ => "1.0.0",
        }
    }

    pub fn all_versions(framework: &str) -> Vec<&'static str> {
        match framework {
            "react" => vec!["18.0", "19.0", "19.1"],
            "vue" => vec!["3.4", "3.5", "3.6"],
            "next" => vec!["14.0", "15.0", "15.1", "15.2", "15.3"],
            "astro" => vec!["4.0", "5.0", "5.4"],
            "express" => vec!["4.18", "5.0", "5.1"],
            "fastify" => vec!["4.28", "5.0", "5.2"],
            "hono" => vec!["4.6", "4.7"],
            "nuxt" => vec!["3.13", "3.14", "3.15", "3.16"],
            "sveltekit" => vec!["2.0", "2.18"],
            "remix" => vec!["2.14", "2.15", "2.16"],
            "angular" => vec!["17.0", "18.0", "19.0", "19.2"],
            "nestjs" => vec!["10.0", "11.0"],
            "adonis" => vec!["6.0", "6.4"],
            "koa" => vec!["2.15", "2.16"],
            "hapi" => vec!["21.0", "21.4"],
            "solid" => vec!["1.9"],
            "qwik" => vec!["1.9", "1.12"],
            "preact" => vec!["10.24", "10.26"],
            "lit" => vec!["3.2"],
            "svelte" => vec!["4.2", "5.0", "5.20"],
            "trpc" => vec!["10.45", "11.0"],
            "graphql" => vec!["3.0", "5.12"],
            "gatsby" => vec!["5.13", "5.14"],
            "laravel" => vec!["11.0", "12.0", "13.0"],
            "django" => vec!["4.2", "5.0", "5.2"],
            "rails" => vec!["7.1", "8.0"],
            "tanstack-router" => vec!["1.60"],
            "remotion" => vec!["4.0"],
            "fresh" => vec!["1.7"],
            "eleventy" => vec!["3.0"],
            "jekyll" => vec!["4.3"],
            "elysia" => vec!["1.1"],
            "stoker" => vec!["0.1"],
            "sanic" => vec!["24.12"],
            "gin" => vec!["1.10"],
            "fastapi" => vec!["0.115"],
            _ => vec!["1.0"],
        }
    }

    pub fn all_frameworks() -> Vec<&'static str> {
        vec![
            "react", "vue", "next", "astro", "express", "fastify", "hono",
            "nuxt", "sveltekit", "remix", "angular", "nestjs", "adonis",
            "koa", "hapi", "solid", "qwik", "preact", "lit", "svelte",
            "trpc", "graphql", "gatsby", "laravel", "django", "rails",
            "tanstack-router", "remotion", "fresh", "eleventy", "jekyll",
            "elysia", "stoker", "sanic", "gin", "fastapi",
        ]
    }

    pub fn categorize(framework: &str) -> JsFrameworkCategory {
        match framework {
            "react" | "vue" | "solid" | "qwik" | "preact" | "lit" | "svelte" | "tanstack-router" | "remotion" => {
                JsFrameworkCategory::FrontendFramework
            }
            "express" | "fastify" | "hono" | "nestjs" | "adonis" | "koa" | "hapi" | "trpc" | "graphql" | "elysia" | "stoker" | "sanic" | "gin" | "fastapi" => {
                JsFrameworkCategory::BackendFramework
            }
            "next" | "nuxt" | "sveltekit" | "remix" | "fresh" => {
                JsFrameworkCategory::FullstackFramework
            }
            "angular" => JsFrameworkCategory::MetaFramework,
            "astro" | "gatsby" | "eleventy" | "jekyll" => JsFrameworkCategory::StaticSiteGenerator,
            "laravel" | "django" | "rails" => JsFrameworkCategory::PolyglotFramework,
            _ => JsFrameworkCategory::PolyglotFramework,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latest_versions() {
        assert_eq!(JsVersions::latest("react"), "19.1.0");
        assert_eq!(JsVersions::latest("express"), "5.1.0");
    }

    #[test]
    fn test_all_versions() {
        let react_versions = JsVersions::all_versions("react");
        assert!(react_versions.contains(&"19.1"));
    }

    #[test]
    fn test_all_frameworks() {
        let frameworks = JsVersions::all_frameworks();
        assert!(frameworks.contains(&"react"));
        assert!(frameworks.contains(&"express"));
        assert!(frameworks.len() >= 36);
    }

    #[test]
    fn test_categorize() {
        assert_eq!(JsVersions::categorize("react"), JsFrameworkCategory::FrontendFramework);
        assert_eq!(JsVersions::categorize("express"), JsFrameworkCategory::BackendFramework);
        assert_eq!(JsVersions::categorize("next"), JsFrameworkCategory::FullstackFramework);
        assert_eq!(JsVersions::categorize("gatsby"), JsFrameworkCategory::StaticSiteGenerator);
        assert_eq!(JsVersions::categorize("laravel"), JsFrameworkCategory::PolyglotFramework);
    }
}

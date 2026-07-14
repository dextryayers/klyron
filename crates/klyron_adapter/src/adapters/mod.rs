mod react;
mod vue;
mod next;
mod astro;
mod express;
mod fastify;
mod hono;
mod nuxt;
mod sveltekit;
mod remix;
mod angular;
mod nestjs;
mod adonis;
mod koa;
mod hapi;
mod solid;
mod qwik;
mod preact;
mod lit;
mod svelte;
mod trpc;
mod graphql;
mod laravel;
mod django;
mod rails;
mod gatsby;

use std::path::Path;
use std::sync::Arc;
use crate::AdapterRegistry;

pub use react::ReactAdapter;
pub use vue::VueAdapter;
pub use next::NextAdapter;
pub use astro::AstroAdapter;
pub use express::ExpressAdapter;
pub use fastify::FastifyAdapter;
pub use hono::HonoAdapter;
pub use nuxt::NuxtAdapter;
pub use sveltekit::SvelteKitAdapter;
pub use remix::RemixAdapter;
pub use angular::AngularAdapter;
pub use nestjs::NestJsAdapter;
pub use adonis::AdonisAdapter;
pub use koa::KoaAdapter;
pub use hapi::HapiAdapter;
pub use solid::SolidAdapter;
pub use qwik::QwikAdapter;
pub use preact::PreactAdapter;
pub use lit::LitAdapter;
pub use svelte::SvelteAdapter;
pub use trpc::TrpcAdapter;
pub use graphql::GraphqlAdapter;
pub use laravel::LaravelAdapter;
pub use django::DjangoAdapter;
pub use rails::RailsAdapter;
pub use gatsby::GatsbyAdapter;

pub fn register_all(registry: &mut AdapterRegistry) {
    registry.register(Arc::new(ReactAdapter));
    registry.register(Arc::new(VueAdapter));
    registry.register(Arc::new(NextAdapter));
    registry.register(Arc::new(AstroAdapter));
    registry.register(Arc::new(ExpressAdapter));
    registry.register(Arc::new(FastifyAdapter));
    registry.register(Arc::new(HonoAdapter));
    registry.register(Arc::new(NuxtAdapter));
    registry.register(Arc::new(SvelteKitAdapter));
    registry.register(Arc::new(RemixAdapter));
    registry.register(Arc::new(AngularAdapter));
    registry.register(Arc::new(NestJsAdapter));
    registry.register(Arc::new(AdonisAdapter));
    registry.register(Arc::new(KoaAdapter));
    registry.register(Arc::new(HapiAdapter));
    registry.register(Arc::new(SolidAdapter));
    registry.register(Arc::new(QwikAdapter));
    registry.register(Arc::new(PreactAdapter));
    registry.register(Arc::new(LitAdapter));
    registry.register(Arc::new(SvelteAdapter));
    registry.register(Arc::new(TrpcAdapter));
    registry.register(Arc::new(GraphqlAdapter));
    registry.register(Arc::new(LaravelAdapter));
    registry.register(Arc::new(DjangoAdapter));
    registry.register(Arc::new(RailsAdapter));
    registry.register(Arc::new(GatsbyAdapter));
}

pub fn detect_adapter(dir: &Path) -> Option<&'static str> {
    let adapters: [&dyn crate::FrameworkAdapter; 26] = [
        &ReactAdapter,
        &VueAdapter,
        &NextAdapter,
        &AstroAdapter,
        &ExpressAdapter,
        &FastifyAdapter,
        &HonoAdapter,
        &NuxtAdapter,
        &SvelteKitAdapter,
        &RemixAdapter,
        &AngularAdapter,
        &NestJsAdapter,
        &AdonisAdapter,
        &KoaAdapter,
        &HapiAdapter,
        &SolidAdapter,
        &QwikAdapter,
        &PreactAdapter,
        &LitAdapter,
        &SvelteAdapter,
        &TrpcAdapter,
        &GraphqlAdapter,
        &LaravelAdapter,
        &DjangoAdapter,
        &RailsAdapter,
        &GatsbyAdapter,
    ];
    for adapter in &adapters {
        if adapter.detect(dir) {
            return Some(adapter.name());
        }
    }
    None
}

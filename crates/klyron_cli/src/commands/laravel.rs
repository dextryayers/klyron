use clap::Subcommand;
use klyron_adapter::laravel_ecosystem::LaravelEcosystem;

#[derive(Subcommand)]
pub enum LaravelCommand {
    /// Install Laravel Horizon (queue monitoring)
    HorizonInstall,
    /// Start Horizon
    HorizonStart,
    /// Pause Horizon
    HorizonPause,
    /// Resume Horizon
    HorizonResume,
    /// Terminate Horizon
    HorizonTerminate,
    /// Check Horizon status
    HorizonStatus,
    /// Clear Horizon metrics
    HorizonClear,
    /// Create a Horizon snapshot
    HorizonSnapshot,
    /// Install Laravel Telescope (debug dashboard)
    TelescopeInstall,
    /// Prune old Telescope entries
    TelescopePrune,
    /// Clear all Telescope entries
    TelescopeClear,
    /// Publish Telescope config
    TelescopePublish,
    /// Install Laravel Reverb (WebSocket server)
    ReverbInstall,
    /// Start Reverb WebSocket server
    ReverbStart {
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        #[arg(long, default_value_t = 8080)]
        port: u16,
        #[arg(long)]
        debug: bool,
    },
    /// Install Laravel Pulse (health monitoring)
    PulseInstall,
    /// Run Pulse health check
    PulseCheck,
    /// Clear Pulse data
    PulseClear,
    /// Install Laravel Pennant (feature flags)
    PennantInstall,
    /// Create a Pennant feature
    PennantFeature {
        name: String,
    },
    /// Install Laravel Breeze (auth scaffolding)
    BreezeInstall {
        #[arg(default_value = "blade")]
        stack: String,
        #[arg(long)]
        testing: bool,
        #[arg(long)]
        dark: bool,
        #[arg(long)]
        pest: bool,
    },
    /// Install Laravel Jetstream (advanced auth)
    JetstreamInstall {
        #[arg(default_value = "livewire")]
        stack: String,
        #[arg(long)]
        teams: bool,
        #[arg(long)]
        pest: bool,
    },
    /// Install Laravel Sail (Docker dev env)
    SailInstall {
        #[arg(long)]
        with: Vec<String>,
    },
    /// Start Sail
    SailUp {
        #[arg(long)]
        daemon: bool,
    },
    /// Stop Sail
    SailDown,
    /// Open Sail shell
    SailShell,
    /// Build Sail images
    SailBuild {
        #[arg(long)]
        no_cache: bool,
    },
    /// View Sail logs
    SailLogs {
        #[arg(long)]
        follow: bool,
    },
    /// Create an Artisan make:command
    MakeCommand {
        name: String,
    },
    /// Create a controller
    MakeController {
        name: String,
        #[arg(long)]
        resource: bool,
        #[arg(long)]
        api: bool,
        #[arg(long)]
        invokable: bool,
        #[arg(long)]
        model: Option<String>,
    },
    /// Create a model
    MakeModel {
        name: String,
        #[arg(long)]
        migration: bool,
        #[arg(long)]
        factory: bool,
        #[arg(long)]
        seed: bool,
        #[arg(long)]
        controller: bool,
        #[arg(long)]
        resource: bool,
        #[arg(long)]
        policy: bool,
    },
    /// Create a migration
    MakeMigration {
        name: String,
        #[arg(long)]
        create: Option<String>,
        #[arg(long)]
        table: Option<String>,
    },
    /// Create a seeder
    MakeSeeder {
        name: String,
    },
    /// Create a factory
    MakeFactory {
        name: String,
        #[arg(long)]
        model: Option<String>,
    },
    /// Create a mail class
    MakeMail {
        name: String,
    },
    /// Create a notification
    MakeNotification {
        name: String,
    },
    /// Create a job
    MakeJob {
        name: String,
        #[arg(long)]
        sync: bool,
    },
    /// Create an event
    MakeEvent {
        name: String,
    },
    /// Create a listener
    MakeListener {
        name: String,
        #[arg(long)]
        event: Option<String>,
        #[arg(long)]
        queued: bool,
    },
    /// Create a policy
    MakePolicy {
        name: String,
        #[arg(long)]
        model: Option<String>,
    },
    /// Create a provider
    MakeProvider {
        name: String,
    },
    /// Create middleware
    MakeMiddleware {
        name: String,
    },
    /// Create a form request
    MakeRequest {
        name: String,
    },
    /// Create an API resource
    MakeResource {
        name: String,
    },
    /// Create a validation rule
    MakeRule {
        name: String,
    },
    /// Create a custom cast
    MakeCast {
        name: String,
    },
    /// Create a broadcasting channel
    MakeChannel {
        name: String,
    },
    /// Create a query scope
    MakeScope {
        name: String,
    },
    /// Create an observer
    MakeObserver {
        name: String,
        #[arg(long)]
        model: Option<String>,
    },
    /// Create a Livewire/Blade component
    MakeComponent {
        name: String,
        #[arg(long)]
        inline: bool,
    },
    /// Create a view
    MakeView {
        name: String,
    },
    /// Create a test
    MakeTest {
        name: String,
        #[arg(long)]
        unit: bool,
        #[arg(long)]
        pest: bool,
    },
}

pub fn run_laravel(cmd: LaravelCommand) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match cmd {
        LaravelCommand::HorizonInstall => LaravelEcosystem::horizon_install(&dir),
        LaravelCommand::HorizonStart => LaravelEcosystem::horizon_start(&dir),
        LaravelCommand::HorizonPause => LaravelEcosystem::horizon_pause(&dir),
        LaravelCommand::HorizonResume => LaravelEcosystem::horizon_resume(&dir),
        LaravelCommand::HorizonTerminate => LaravelEcosystem::horizon_terminate(&dir),
        LaravelCommand::HorizonStatus => LaravelEcosystem::horizon_status(&dir),
        LaravelCommand::HorizonClear => LaravelEcosystem::horizon_clear(&dir),
        LaravelCommand::HorizonSnapshot => LaravelEcosystem::horizon_snapshot(&dir),
        LaravelCommand::TelescopeInstall => LaravelEcosystem::telescope_install(&dir),
        LaravelCommand::TelescopePrune => LaravelEcosystem::telescope_prune(&dir),
        LaravelCommand::TelescopeClear => LaravelEcosystem::telescope_clear(&dir),
        LaravelCommand::TelescopePublish => LaravelEcosystem::telescope_publish(&dir),
        LaravelCommand::ReverbInstall => LaravelEcosystem::reverb_install(&dir),
        LaravelCommand::ReverbStart { host, port, debug } => LaravelEcosystem::reverb_start(&dir, Some(&host), Some(port), debug),
        LaravelCommand::PulseInstall => LaravelEcosystem::pulse_install(&dir),
        LaravelCommand::PulseCheck => LaravelEcosystem::pulse_check(&dir),
        LaravelCommand::PulseClear => LaravelEcosystem::pulse_clear(&dir),
        LaravelCommand::PennantInstall => LaravelEcosystem::pennant_install(&dir),
        LaravelCommand::PennantFeature { name } => LaravelEcosystem::pennant_feature(&dir, &name),
        LaravelCommand::BreezeInstall { stack, testing, dark, pest } => LaravelEcosystem::breeze_install(&dir, &stack, testing, dark, pest),
        LaravelCommand::JetstreamInstall { stack, teams, pest } => LaravelEcosystem::jetstream_install(&dir, &stack, teams, pest),
        LaravelCommand::SailInstall { with } => {
            let with_refs: Vec<&str> = with.iter().map(|s| s.as_str()).collect();
            LaravelEcosystem::sail_install(&dir, &with_refs)
        }
        LaravelCommand::SailUp { daemon } => LaravelEcosystem::sail_up(&dir, daemon),
        LaravelCommand::SailDown => LaravelEcosystem::sail_down(&dir),
        LaravelCommand::SailShell => LaravelEcosystem::sail_shell(&dir),
        LaravelCommand::SailBuild { no_cache } => LaravelEcosystem::sail_build(&dir, no_cache),
        LaravelCommand::SailLogs { follow } => LaravelEcosystem::sail_logs(&dir, follow),
        LaravelCommand::MakeCommand { name } => LaravelEcosystem::make_command(&dir, &name),
        LaravelCommand::MakeController { name, resource, api, invokable, model } => LaravelEcosystem::make_controller(&dir, &name, resource, api, invokable, model.as_deref()),
        LaravelCommand::MakeModel { name, migration, factory, seed, controller, resource, policy } => LaravelEcosystem::make_model(&dir, &name, migration, factory, seed, controller, resource, policy),
        LaravelCommand::MakeMigration { name, create, table } => LaravelEcosystem::make_migration(&dir, &name, create.as_deref(), table.as_deref()),
        LaravelCommand::MakeSeeder { name } => LaravelEcosystem::make_seeder(&dir, &name),
        LaravelCommand::MakeFactory { name, model } => LaravelEcosystem::make_factory(&dir, &name, model.as_deref()),
        LaravelCommand::MakeMail { name } => LaravelEcosystem::make_mail(&dir, &name),
        LaravelCommand::MakeNotification { name } => LaravelEcosystem::make_notification(&dir, &name),
        LaravelCommand::MakeJob { name, sync } => LaravelEcosystem::make_job(&dir, &name, sync),
        LaravelCommand::MakeEvent { name } => LaravelEcosystem::make_event(&dir, &name),
        LaravelCommand::MakeListener { name, event, queued } => LaravelEcosystem::make_listener(&dir, &name, event.as_deref(), queued),
        LaravelCommand::MakePolicy { name, model } => LaravelEcosystem::make_policy(&dir, &name, model.as_deref()),
        LaravelCommand::MakeProvider { name } => LaravelEcosystem::make_provider(&dir, &name),
        LaravelCommand::MakeMiddleware { name } => LaravelEcosystem::make_middleware(&dir, &name),
        LaravelCommand::MakeRequest { name } => LaravelEcosystem::make_request(&dir, &name),
        LaravelCommand::MakeResource { name } => LaravelEcosystem::make_resource(&dir, &name),
        LaravelCommand::MakeRule { name } => LaravelEcosystem::make_rule(&dir, &name),
        LaravelCommand::MakeCast { name } => LaravelEcosystem::make_cast(&dir, &name),
        LaravelCommand::MakeChannel { name } => LaravelEcosystem::make_channel(&dir, &name),
        LaravelCommand::MakeScope { name } => LaravelEcosystem::make_scope(&dir, &name),
        LaravelCommand::MakeObserver { name, model } => LaravelEcosystem::make_observer(&dir, &name, model.as_deref()),
        LaravelCommand::MakeComponent { name, inline } => LaravelEcosystem::make_component(&dir, &name, inline),
        LaravelCommand::MakeView { name } => LaravelEcosystem::make_view(&dir, &name),
        LaravelCommand::MakeTest { name, unit, pest } => LaravelEcosystem::make_test(&dir, &name, unit, pest),
    }
}

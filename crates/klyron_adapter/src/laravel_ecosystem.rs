use std::path::Path;
use std::process::Command;

pub struct LaravelEcosystem;

impl LaravelEcosystem {
    fn artisan(dir: &Path, args: &[&str]) -> anyhow::Result<()> {
        let status = Command::new("php")
            .arg("artisan")
            .args(args)
            .current_dir(dir)
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to run artisan: {e}"))?;
        if !status.success() {
            anyhow::bail!("artisan exited with code {}", status);
        }
        Ok(())
    }

    fn composer(dir: &Path, args: &[&str]) -> anyhow::Result<()> {
        let status = Command::new("composer")
            .args(args)
            .current_dir(dir)
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to run composer: {e}"))?;
        if !status.success() {
            anyhow::bail!("composer exited with code {}", status);
        }
        Ok(())
    }

    fn npm(dir: &Path, args: &[&str]) -> anyhow::Result<()> {
        let status = Command::new("npm")
            .args(args)
            .current_dir(dir)
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to run npm: {e}"))?;
        if !status.success() {
            anyhow::bail!("npm exited with code {}", status);
        }
        Ok(())
    }

    // ── Horizon ────────────────────────────────────────────────────────────

    pub fn horizon_install(dir: &Path) -> anyhow::Result<()> {
        Self::composer(dir, &["require", "laravel/horizon"])?;
        Self::artisan(dir, &["horizon:install"])?;
        println!("Horizon installed. Run `klyron artisan horizon` to start.");
        Ok(())
    }

    pub fn horizon_start(dir: &Path) -> anyhow::Result<()> {
        Self::artisan(dir, &["horizon"])
    }

    pub fn horizon_pause(dir: &Path) -> anyhow::Result<()> {
        Self::artisan(dir, &["horizon:pause"])
    }

    pub fn horizon_resume(dir: &Path) -> anyhow::Result<()> {
        Self::artisan(dir, &["horizon:resume"])
    }

    pub fn horizon_terminate(dir: &Path) -> anyhow::Result<()> {
        Self::artisan(dir, &["horizon:terminate"])
    }

    pub fn horizon_status(dir: &Path) -> anyhow::Result<()> {
        Self::artisan(dir, &["horizon:status"])
    }

    pub fn horizon_clear(dir: &Path) -> anyhow::Result<()> {
        Self::artisan(dir, &["horizon:clear"])
    }

    pub fn horizon_snapshot(dir: &Path) -> anyhow::Result<()> {
        Self::artisan(dir, &["horizon:snapshot"])
    }

    // ── Telescope ──────────────────────────────────────────────────────────

    pub fn telescope_install(dir: &Path) -> anyhow::Result<()> {
        Self::composer(dir, &["require", "laravel/telescope"])?;
        Self::artisan(dir, &["telescope:install"])?;
        Self::artisan(dir, &["migrate"])?;
        println!("Telescope installed at /telescope");
        Ok(())
    }

    pub fn telescope_prune(dir: &Path) -> anyhow::Result<()> {
        Self::artisan(dir, &["telescope:prune"])
    }

    pub fn telescope_clear(dir: &Path) -> anyhow::Result<()> {
        Self::artisan(dir, &["telescope:clear"])
    }

    pub fn telescope_publish(dir: &Path) -> anyhow::Result<()> {
        Self::artisan(dir, &["vendor:publish", "--tag=telescope-config"])
    }

    // ── Reverb ─────────────────────────────────────────────────────────────

    pub fn reverb_install(dir: &Path) -> anyhow::Result<()> {
        Self::composer(dir, &["require", "laravel/reverb"])?;
        Self::artisan(dir, &["reverb:install"])?;
        println!("Reverb installed. Run `klyron reverb start` to start WebSocket server.");
        Ok(())
    }

    pub fn reverb_start(dir: &Path, host: Option<&str>, port: Option<u16>, debug: bool) -> anyhow::Result<()> {
        let mut args: Vec<String> = vec!["reverb:start".into()];
        if let Some(h) = host {
            args.push("--host".into());
            args.push(h.into());
        }
        if let Some(p) = port {
            args.push("--port".into());
            args.push(p.to_string());
        }
        if debug {
            args.push("--debug".into());
        }
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        Self::artisan(dir, &args_refs)
    }

    // ── Pulse ──────────────────────────────────────────────────────────────

    pub fn pulse_install(dir: &Path) -> anyhow::Result<()> {
        Self::composer(dir, &["require", "laravel/pulse"])?;
        Self::artisan(dir, &["pulse:install"])?;
        Self::artisan(dir, &["migrate"])?;
        println!("Pulse installed at /pulse");
        Ok(())
    }

    pub fn pulse_check(dir: &Path) -> anyhow::Result<()> {
        Self::artisan(dir, &["pulse:check"])
    }

    pub fn pulse_clear(dir: &Path) -> anyhow::Result<()> {
        Self::artisan(dir, &["pulse:clear"])
    }

    // ── Pennant ────────────────────────────────────────────────────────────

    pub fn pennant_install(dir: &Path) -> anyhow::Result<()> {
        Self::composer(dir, &["require", "laravel/pennant"])?;
        Self::artisan(dir, &["vendor:publish", "--tag=pennant-config"])?;
        Self::artisan(dir, &["migrate"])?;
        println!("Pennant installed. Use `klyron artisan pennant:feature` to manage features.");
        Ok(())
    }

    pub fn pennant_feature(dir: &Path, name: &str) -> anyhow::Result<()> {
        Self::artisan(dir, &["pennant:feature", name])
    }

    // ── Breeze ─────────────────────────────────────────────────────────────

    pub fn breeze_install(dir: &Path, stack: &str, testing: bool, dark: bool, pest: bool) -> anyhow::Result<()> {
        Self::composer(dir, &["require", "laravel/breeze", "--dev"])?;
        let mut args = vec!["breeze:install", stack];
        if testing { args.push("--with-tests"); }
        if dark { args.push("--dark"); }
        if pest { args.push("--pest"); }
        Self::artisan(dir, &args)?;
        Self::npm(dir, &["install"])?;
        Self::npm(dir, &["run", "build"])?;
        Self::artisan(dir, &["migrate"])?;
        println!("Breeze ({stack}) installed successfully.");
        Ok(())
    }

    // ── Jetstream ──────────────────────────────────────────────────────────

    pub fn jetstream_install(dir: &Path, stack: &str, teams: bool, pest: bool) -> anyhow::Result<()> {
        if stack != "livewire" && stack != "inertia" {
            anyhow::bail!("Jetstream stack must be 'livewire' or 'inertia'");
        }
        Self::composer(dir, &["require", "laravel/jetstream"])?;
        let mut args = vec!["jetstream:install", stack];
        if teams { args.push("--teams"); }
        if pest { args.push("--pest"); }
        Self::artisan(dir, &args)?;
        Self::npm(dir, &["install"])?;
        Self::npm(dir, &["run", "build"])?;
        Self::artisan(dir, &["migrate"])?;
        println!("Jetstream ({stack}) installed successfully.");
        Ok(())
    }

    // ── Sail ───────────────────────────────────────────────────────────────

    pub fn sail_install(dir: &Path, with: &[&str]) -> anyhow::Result<()> {
        Self::composer(dir, &["require", "laravel/sail", "--dev"])?;
        let mut args = vec!["sail:install"];
        for w in with {
            args.push("--with");
            args.push(w);
        }
        Self::artisan(dir, &args)?;
        println!("Sail installed. Use `klyron sail up` to start.");
        Ok(())
    }

    pub fn sail_up(dir: &Path, daemon: bool) -> anyhow::Result<()> {
        let mut cmd = Command::new("./vendor/bin/sail");
        cmd.arg("up");
        if daemon { cmd.arg("-d"); }
        cmd.current_dir(dir);
        let status = cmd.status().map_err(|e| anyhow::anyhow!("sail up: {e}"))?;
        if !status.success() { anyhow::bail!("sail up failed"); }
        Ok(())
    }

    pub fn sail_down(dir: &Path) -> anyhow::Result<()> {
        sail_cmd(dir, &["down"])
    }

    pub fn sail_shell(dir: &Path) -> anyhow::Result<()> {
        sail_cmd(dir, &["shell"])
    }

    pub fn sail_build(dir: &Path, no_cache: bool) -> anyhow::Result<()> {
        let mut args = vec!["build"];
        if no_cache { args.push("--no-cache"); }
        sail_cmd(dir, &args)
    }

    pub fn sail_logs(dir: &Path, follow: bool) -> anyhow::Result<()> {
        let mut args = vec!["logs"];
        if follow { args.push("-f"); }
        sail_cmd(dir, &args)
    }

    // ── Artisan Make Commands ──────────────────────────────────────────────

    pub fn make_command(dir: &Path, name: &str) -> anyhow::Result<()> {
        Self::artisan(dir, &["make:command", name])
    }

    pub fn make_controller(dir: &Path, name: &str, resource: bool, api: bool, invokable: bool, model: Option<&str>) -> anyhow::Result<()> {
        let mut args = vec!["make:controller", name];
        if resource { args.push("--resource"); }
        if api { args.push("--api"); }
        if invokable { args.push("--invokable"); }
        if let Some(m) = model { args.push("--model"); args.push(m); }
        Self::artisan(dir, &args)
    }

    pub fn make_model(dir: &Path, name: &str, migration: bool, factory: bool, seed: bool, controller: bool, resource: bool, policy: bool) -> anyhow::Result<()> {
        let mut args = vec!["make:model", name];
        if migration { args.push("-m"); }
        if factory { args.push("-f"); }
        if seed { args.push("-s"); }
        if controller { args.push("-c"); }
        if resource { args.push("-r"); }
        if policy { args.push("-p"); }
        Self::artisan(dir, &args)
    }

    pub fn make_migration(dir: &Path, name: &str, create: Option<&str>, table: Option<&str>) -> anyhow::Result<()> {
        let mut args = vec!["make:migration", name];
        if let Some(c) = create { args.push("--create"); args.push(c); }
        if let Some(t) = table { args.push("--table"); args.push(t); }
        Self::artisan(dir, &args)
    }

    pub fn make_seeder(dir: &Path, name: &str) -> anyhow::Result<()> {
        Self::artisan(dir, &["make:seeder", name])
    }

    pub fn make_factory(dir: &Path, name: &str, model: Option<&str>) -> anyhow::Result<()> {
        let mut args = vec!["make:factory", name];
        if let Some(m) = model { args.push("--model"); args.push(m); }
        Self::artisan(dir, &args)
    }

    pub fn make_mail(dir: &Path, name: &str) -> anyhow::Result<()> {
        Self::artisan(dir, &["make:mail", name])
    }

    pub fn make_notification(dir: &Path, name: &str) -> anyhow::Result<()> {
        Self::artisan(dir, &["make:notification", name])
    }

    pub fn make_job(dir: &Path, name: &str, sync: bool) -> anyhow::Result<()> {
        let mut args = vec!["make:job", name];
        if sync { args.push("--sync"); }
        Self::artisan(dir, &args)
    }

    pub fn make_event(dir: &Path, name: &str) -> anyhow::Result<()> {
        Self::artisan(dir, &["make:event", name])
    }

    pub fn make_listener(dir: &Path, name: &str, event: Option<&str>, queued: bool) -> anyhow::Result<()> {
        let mut args = vec!["make:listener", name];
        if let Some(e) = event { args.push("--event"); args.push(e); }
        if queued { args.push("--queued"); }
        Self::artisan(dir, &args)
    }

    pub fn make_policy(dir: &Path, name: &str, model: Option<&str>) -> anyhow::Result<()> {
        let mut args = vec!["make:policy", name];
        if let Some(m) = model { args.push("--model"); args.push(m); }
        Self::artisan(dir, &args)
    }

    pub fn make_provider(dir: &Path, name: &str) -> anyhow::Result<()> {
        Self::artisan(dir, &["make:provider", name])
    }

    pub fn make_middleware(dir: &Path, name: &str) -> anyhow::Result<()> {
        Self::artisan(dir, &["make:middleware", name])
    }

    pub fn make_request(dir: &Path, name: &str) -> anyhow::Result<()> {
        Self::artisan(dir, &["make:request", name])
    }

    pub fn make_resource(dir: &Path, name: &str) -> anyhow::Result<()> {
        Self::artisan(dir, &["make:resource", name])
    }

    pub fn make_rule(dir: &Path, name: &str) -> anyhow::Result<()> {
        Self::artisan(dir, &["make:rule", name])
    }

    pub fn make_cast(dir: &Path, name: &str) -> anyhow::Result<()> {
        Self::artisan(dir, &["make:cast", name])
    }

    pub fn make_channel(dir: &Path, name: &str) -> anyhow::Result<()> {
        Self::artisan(dir, &["make:channel", name])
    }

    pub fn make_scope(dir: &Path, name: &str) -> anyhow::Result<()> {
        Self::artisan(dir, &["make:scope", name])
    }

    pub fn make_observer(dir: &Path, name: &str, model: Option<&str>) -> anyhow::Result<()> {
        let mut args = vec!["make:observer", name];
        if let Some(m) = model { args.push("--model"); args.push(m); }
        Self::artisan(dir, &args)
    }

    pub fn make_component(dir: &Path, name: &str, inline: bool) -> anyhow::Result<()> {
        let mut args = vec!["make:component", name];
        if inline { args.push("--inline"); }
        Self::artisan(dir, &args)
    }

    pub fn make_view(dir: &Path, name: &str) -> anyhow::Result<()> {
        Self::artisan(dir, &["make:view", name])
    }

    pub fn make_test(dir: &Path, name: &str, unit: bool, pest: bool) -> anyhow::Result<()> {
        let mut args = vec!["make:test", name];
        if unit { args.push("--unit"); }
        if pest { args.push("--pest"); }
        Self::artisan(dir, &args)
    }
}

fn sail_cmd(dir: &Path, args: &[&str]) -> anyhow::Result<()> {
    let mut cmd = Command::new("./vendor/bin/sail");
    cmd.args(args);
    cmd.current_dir(dir);
    let status = cmd.status().map_err(|e| anyhow::anyhow!("sail command: {e}"))?;
    if !status.success() { anyhow::bail!("sail command failed"); }
    Ok(())
}

pub fn detect_laravel_version(dir: &Path) -> Option<String> {
    let composer_path = dir.join("composer.json");
    let content = std::fs::read_to_string(composer_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let require = json.get("require")?;
    let framework = require.get("laravel/framework")?.as_str()?;
    // Extract major version from constraint like "^9.52" or "^10.48" or "^11.0"
    let version = framework.trim_start_matches('^').split('.').next()?;
    Some(version.to_string())
}

pub fn composer_require(dir: &Path, packages: &[&str], dev: bool) -> anyhow::Result<()> {
    let mut args = vec!["require"];
    if dev { args.push("--dev"); }
    args.extend(packages);
    LaravelEcosystem::composer(dir, &args)
}

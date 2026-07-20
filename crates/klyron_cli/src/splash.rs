use crate::color::Color;
use std::io::Write;

const BANNER: &str = r#"
                                                                                =-                         
                                                                            +*                             
                                                                   **                +=                    
                                                        -:::--           ::::+ *::-          +*            
                                                    :::--=**      -:::::::::::::-         +                
                                                :::==-**##  =-::::::::::::::::-      ::                    
                                            *::+-+***#%%=---:::---::::::::::-   -:::-                      
                                          ::==****#####----++++*####::::::-:::::::=                        
                                       :::+*****#####****#######%#:::::::::::::-     ***                   
                                     ::-*****################%%%+--:::::::----                             
                                    ::-****################%%%*---::::-----=                               
                                  :::=*####################%%+--:::------+  +#    =                        
                                 ::::#######################==**#==---=+       *=                          
                                ::::*#########################%*==---+*   ==+                              
                                :::*#############%%%%########%*==--=+=====#                                
                               -:-############%%%%=#%%%#####%%%==+=====+      =:::                         
                               :%%%#########%%%::::::%%####%%#==+====+      :::                            
                               :%::%%#####%%%%:::::::%%######+++=+=                                        
                                :-::%%%%%%%%%:::::::*%%#####+++=    #*                                     
                                :%:::%%%%%%%%::::::#%%####*++   ***#*   %+*                                
                                 :%--%%%%%%%%%+-#%%#*####*   *#%  %                                        
                                  :+###%%%%%###**######*  %%%###*+#                                        
                                    :+##############*   %%%#########*+===%                                 
                                +       #******#      %%###############%+                                  
                               =*####%%%%%%        %#######%%%%%%#***                                      
                                 +######%%%%%=*###########%%%                                              
                                    #***#*##  +*#######%##                                                 
                                                                                                           
                                                                                                           
                                                                                                           
                                                                                                           
                                                                                                           
                ::   =::      :=          ::    ::      ::::::::=     ::::::::     -::    ::               
                :: *::        :=           +::::=       ::     :-     ::     :     -: ::  ::               
                ::  ::        :+             ::         ::::::::      ::     :     -:  :::::               
                ::   +::      ::::::::       ::         ::    ::      ::::::::     -:    :::"#;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn ansi_rgb(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{};{};{}m", r, g, b)
}

fn rgb_paint(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("{}{}\x1b[0m", ansi_rgb(r, g, b), text)
}

fn hide_cursor() {
    print!("\x1b[?25l");
    std::io::stdout().flush().ok();
}

fn show_cursor() {
    print!("\x1b[?25h");
    std::io::stdout().flush().ok();
}

pub fn show_splash() {
    hide_cursor();
    print!("\x1b[2J\x1b[H");
    animate_loading(1);
    print_banner();

    println!();
    println!("{}", Color::BRIGHT_CYAN.bold("  ╭──────────────────────────────────────────────────────────────╮"));
    println!("{}", Color::BRIGHT_CYAN.bold("  │              Klyron - Universal Polyglot Runtime             │"));
    println!("{}", Color::BRIGHT_CYAN.bold("  │               Code Anything, Run Anywhere                    │"));
    println!("{}", Color::BRIGHT_CYAN.bold("  ╰──────────────────────────────────────────────────────────────╯"));
    println!();

    println!("  {} {}",
        Color::GREEN.paint("◆"),
        Color::WHITE.bold("Klyron is a universal polyglot runtime, project scaffolder, and package manager.")
    );
    println!("  {} {}",
        Color::GREEN.paint("◆"),
        Color::WHITE.bold("Run JS/TS/JSX/TSX, scaffold 40+ frameworks, manage deps, dev servers, DB, deploy.")
    );
    println!();

    println!("  {} {}",
        Color::YELLOW.paint("©"),
        Color::WHITE.paint(format!("klyron {}  —  by AniipID  —  Universal Polyglot Runtime & Project Scaffolder", VERSION))
    );
    println!("  {} {}",
        Color::DIM.paint("  "),
        Color::DIM.paint("Code anything. Run anywhere. Use any package manager.")
    );
    println!();

    println!("  {} {}",
        Color::CYAN.bold("Usage:"),
        Color::WHITE.paint("klyron [OPTIONS] <COMMAND> [ARGS]")
    );
    println!();

    // ── Flags ──────────────────────────────────────────────────────────
    section("Flags");
    flag("-v, --verbose", "Increase verbosity level (use multiple times for more detail)");
    flag("-q, --quiet", "Suppress all non-essential output (errors only)");
    flag("-V, --version", "Print version information and exit");
    flag("--json", "Output results in JSON format for programmatic use");
    flag("--pm <mode>", "Package manager mode: npm, pnpm, yarn, bun, klyron");
    flag("--engine <engine>", "JavaScript engine: v8, boa, quickjs, jsc, auto");
    flag("--engine-pool-size <n>", "Size of the concurrent engine pool (default: 4)");
    flag("--pre-warm", "Pre-warm JS engines at startup for faster evaluation");
    flag("-h, --help", "Print help information for any command");
    println!();

    // ── Runtime ────────────────────────────────────────────────────────────
    section("Runtime");
    cmd("run", "Execute a file — auto-detects .js, .ts, .tsx, .jsx, .py, .rb, .go, .rs, .zig, .c, .cpp");
    cmd("repl", "Start an interactive REPL session with live evaluation");
    cmd(r#"eval "code""#, "Evaluate JavaScript/TypeScript code inline from the command line");
    cmd("shell", "Start an interactive polyglot shell for multiple languages");
    println!();

    // ── Development ────────────────────────────────────────────────────────
    section("Development");
    cmd("dev", "Start dev server with HMR, file watching and live reload");
    cmd("dev <file>", "Start dev server with a custom entry point (e.g. src/index.ts)");
    cmd("dev --watch", "Start dev server in watch mode — auto-restart on file changes");
    cmd("dev --hot", "Start dev server with hot module replacement enabled");
    cmd("dev --host", "Start dev server bound to 0.0.0.0 (accessible on network)");
    cmd("dev --port", "Start dev server on a specific port (e.g. --port 3000)");
    println!();

    // ── Build ──────────────────────────────────────────────────────────────
    section("Build");
    cmd("build", "Build the current project for production");
    cmd("build <file>", "Build a specific entry file");
    cmd("build --minify", "Build with minification enabled");
    cmd("build --sourcemap", "Build with sourcemaps generated");
    cmd("build --target browser", "Build targeting browser runtime");
    cmd("build --target node", "Build targeting Node.js runtime");
    cmd("build --target edge", "Build targeting edge compute (Cloudflare Workers, Deno)");
    cmd("build --target lambda", "Build targeting AWS Lambda deployment");
    println!();

    // ── Package Manager ────────────────────────────────────────────────────
    section("Package Manager");
    cmd("install", "Install all dependencies from package.json / composer.json / Cargo.toml");
    cmd("install <pkg>", "Install one or more packages (e.g. install react react-dom)");
    cmd("add <pkg>", "Alias for install — add a package dependency");
    cmd("remove <pkg>", "Remove a package dependency");
    cmd("uninstall <pkg>", "Alias for remove");
    cmd("update", "Update all out-of-date dependencies");
    cmd("upgrade", "Upgrade the klyron CLI itself to the latest version");
    cmd("outdated", "List packages that have newer versions available");
    cmd("audit", "Audit dependencies for security vulnerabilities");
    cmd("doctor", "Run system diagnostics to check your environment health");
    cmd("dedupe", "Deduplicate repeated dependencies in the lockfile");
    println!();

    // ── Package.json Scripts ───────────────────────────────────────────────
    section("Package.json Scripts");
    cmd("start", "Run the start script from package.json");
    cmd("test", "Run the test suite (auto-detects vitest, jest, mocha, etc.)");
    cmd("lint", "Lint the codebase (auto-detects eslint, oxlint, etc.)");
    cmd("format", "Format the codebase (auto-detects prettier, dprint, etc.)");
    cmd("run dev", "Run the dev script from package.json");
    cmd("run build", "Run the build script from package.json");
    cmd("run <script>", "Run any custom script defined in package.json");
    println!();

    // ── Workspace / Monorepo ──────────────────────────────────────────────
    section("Workspace / Monorepo");
    cmd("workspace init", "Initialize a new monorepo workspace");
    cmd("workspace list", "List all workspace members");
    cmd("workspace add", "Add a new package to the workspace");
    cmd("workspace run", "Run a script across all workspace members");
    println!();

    // ── Framework Scaffold (Frontend) ─────────────────────────────────────
    section("Framework Generator — Frontend");
    cmd("create react", "Scaffold a React + Vite + TypeScript application");
    cmd("create vue", "Scaffold a Vue 3 + Vite + TypeScript application");
    cmd("create next", "Scaffold a Next.js App Router application");
    cmd("create nuxt", "Scaffold a Nuxt 3 SSR application");
    cmd("create sveltekit", "Scaffold a SvelteKit application");
    cmd("create astro", "Scaffold an Astro content-driven site");
    cmd("create solid", "Scaffold a Solid.js application");
    cmd("create qwik", "Scaffold a Qwik application with resumability");
    cmd("create angular", "Scaffold an Angular CLI application");
    cmd("create remix", "Scaffold a Remix + React Router application");
    println!();

    // ── Framework Scaffold (Backend) ──────────────────────────────────────
    section("Framework Generator — Backend");
    cmd("create express", "Scaffold an Express.js API with ESM and middleware");
    cmd("create fastify", "Scaffold a Fastify backend application");
    cmd("create nest", "Scaffold a NestJS modular backend");
    cmd("create hono", "Scaffold a Hono lightweight API server");
    cmd("create koa", "Scaffold a Koa.js application");
    cmd("create hapi", "Scaffold a Hapi.js application");
    cmd("create adonis", "Scaffold an AdonisJS full-stack application");
    println!();

    // ── Laravel Integration ──────────────────────────────────────────────
    section("Laravel Integration");
    cmd("create laravel-react", "Scaffold Laravel + React SPA with Breeze");
    cmd("create laravel-vue", "Scaffold Laravel + Vue 3 SPA with Breeze");
    cmd("create laravel-inertia-react", "Scaffold Laravel + Inertia.js + React");
    cmd("create laravel-inertia-vue", "Scaffold Laravel + Inertia.js + Vue");
    cmd("create laravel-livewire", "Scaffold Laravel + Livewire + Volt");
    cmd("create laravel-next", "Scaffold Laravel Breeze + Next.js API mode");
    cmd("create laravel-astro", "Scaffold Laravel Breeze + Astro frontend");
    cmd("create laravel-api", "Scaffold a headless Laravel JSON API");
    println!();

    // ── Template ──────────────────────────────────────────────────────────
    section("Template");
    cmd("template list", "List all available templates from adapters directory");
    cmd("template show <template>", "Show detailed info about a template");
    cmd("template create <template> <project>", "Create project with interactive version picker (↑↓)");
    println!();

    // ── Database ─────────────────────────────────────────────────────────
    section("Database (ORM-agnostic)");
    cmd("db init", "Initialize the database schema and configuration");
    cmd("db generate", "Generate database client / type definitions");
    cmd("db migrate", "Run pending database migrations");
    cmd("db push", "Push schema changes directly to the database");
    cmd("db pull", "Pull the database schema into your project");
    cmd("db seed", "Seed the database with sample data");
    cmd("db reset", "Drop, recreate, and re-seed the database");
    cmd("db studio", "Open a web-based database studio GUI");
    println!();

    section("  ── Prisma Compat ──");
    cmd("prisma generate", "Generate Prisma client from schema");
    cmd("prisma migrate", "Run Prisma database migrations");
    cmd("prisma studio", "Open Prisma Studio GUI");
    cmd("prisma db push", "Push Prisma schema to database");
    println!();

    section("  ── Drizzle Compat ──");
    cmd("drizzle generate", "Generate Drizzle ORM migrations");
    cmd("drizzle migrate", "Run Drizzle migrations");
    cmd("drizzle studio", "Open Drizzle Studio GUI");
    println!();

    // ── Testing ───────────────────────────────────────────────────────────
    section("Testing");
    cmd("test", "Run the full test suite");
    cmd("test watch", "Run tests in watch mode (re-run on changes)");
    cmd("test coverage", "Run tests with coverage reporting");
    cmd("test ui", "Open the test UI dashboard (Vitest UI)");
    cmd("test e2e", "Run end-to-end tests");
    cmd("test unit", "Run unit tests only");
    cmd("test integration", "Run integration tests only");
    println!();

    // ── Benchmark ─────────────────────────────────────────────────────────
    section("Benchmark");
    cmd("bench", "Run the project benchmark suite");
    cmd("bench runtime", "Benchmark JavaScript runtime execution speed");
    cmd("bench http", "Benchmark HTTP server request throughput");
    cmd("bench memory", "Benchmark memory usage patterns");
    cmd("bench startup", "Benchmark cold-start initialization time");
    println!();

    // ── Linter / Formatter / Type Check ──────────────────────────────────
    section("Linter & Formatter");
    cmd("lint", "Lint the entire codebase");
    cmd("lint src/", "Lint a specific directory or file");
    cmd("lint --fix", "Lint and auto-fix issues");
    println!();

    section("  ── Format ──");
    cmd("format", "Check code formatting (without writing)");
    cmd("format src/", "Check formatting of a specific directory");
    cmd("format --write", "Format and write changes to disk");
    println!();

    section("  ── Type Check ──");
    cmd("check", "Run type checking on the project");
    cmd("check types", "Run TypeScript type checking only");
    cmd("check project", "Run a full project health check");
    println!();

    // ── Plugin System ────────────────────────────────────────────────────
    section("Plugin System");
    cmd("plugin install", "Install a klyron plugin");
    cmd("plugin remove", "Remove an installed plugin");
    cmd("plugin list", "List all installed plugins");
    cmd("plugin update", "Update all plugins to latest versions");
    cmd("plugin create", "Scaffold a new klyron plugin project");
    println!();

    // ── Registry ─────────────────────────────────────────────────────────
    section("Registry (npm-compatible)");
    cmd("publish", "Publish the current package to the registry");
    cmd("unpublish <pkg>", "Remove a published package version");
    cmd("login", "Authenticate with the registry");
    cmd("logout", "Clear the registry authentication token");
    cmd("whoami", "Display the currently authenticated user");
    cmd("search <query>", "Search the registry for packages");
    cmd("info <pkg>", "Show detailed info about a package");
    println!();

    // ── Cache ────────────────────────────────────────────────────────────
    section("Cache");
    cmd("cache clean", "Clear all cached data");
    cmd("cache prune", "Remove stale cache entries");
    cmd("cache info", "Show cache statistics and disk usage");
    println!();

    // ── Node Compatibility ───────────────────────────────────────────────
    section("Node.js Compatibility");
    cmd("compat check", "Check the current project for Node.js compatibility issues");
    cmd("compat react", "Check React-specific compatibility");
    cmd("compat next", "Check Next.js compatibility with klyron");
    cmd("compat astro", "Check Astro compatibility");
    cmd("compat nest", "Check NestJS compatibility");
    println!();

    // ── Native Modules ───────────────────────────────────────────────────
    section("Native Modules (NAPI)");
    cmd("napi build", "Build native N-API modules");
    cmd("napi generate", "Generate N-API bindings from a definition");
    cmd("napi test", "Test native module integration");
    println!();

    // ── Docker ───────────────────────────────────────────────────────────
    section("Docker");
    cmd("docker init", "Generate Dockerfile and .dockerignore");
    cmd("docker build", "Build a Docker image for the project");
    cmd("docker run", "Run the project inside a Docker container");
    println!();

    // ── Deployment ──────────────────────────────────────────────────────
    section("Deployment");
    cmd("deploy vercel", "Deploy to Vercel with zero configuration");
    cmd("deploy cloudflare", "Deploy to Cloudflare Workers / Pages");
    cmd("deploy railway", "Deploy to Railway");
    cmd("deploy fly", "Deploy to Fly.io");
    cmd("deploy docker", "Deploy via Docker image to any host");
    println!();

    // ── Project Utilities ───────────────────────────────────────────────
    section("Project Utilities");
    cmd("init", "Initialize klyron config in the current directory");
    cmd("upgrade", "Upgrade klyron to the latest version");
    cmd("doctor", "Run system diagnostics and environment checks");
    cmd("info", "Show project and environment information");
    cmd("version", "Show klyron version (also: -V / --version)");
    cmd("telemetry", "Manage telemetry preferences (enable / disable / status)");
    cmd("config", "View and edit klyron configuration");
    cmd("clean", "Clean build artifacts and temporary files");
    println!();

    // ── AI-powered ──────────────────────────────────────────────────────
    section("AI-powered (Enterprise)");
    cmd("ai generate", "Generate code using AI (components, routes, APIs)");
    cmd("ai optimize", "Optimize code for performance with AI suggestions");
    cmd("ai review", "Review code with AI-powered static analysis");
    cmd("ai docs", "Generate documentation using AI");
    cmd("ai test", "Generate test cases using AI");
    cmd("ai migrate", "Migrate code between frameworks using AI (React↔Vue, Express↔Fastify)");
    println!();

    // ── Most Frequently Used ────────────────────────────────────────────
    section("Most Frequently Used");
    cmd("create react", "Scaffold a React project");
    cmd("create next", "Scaffold a Next.js project");
    cmd("create astro", "Scaffold an Astro project");
    cmd("create nest", "Scaffold a NestJS backend");
    cmd("create laravel-react", "Scaffold Laravel + React");
    cmd("create laravel-vue", "Scaffold Laravel + Vue");
    cmd("install", "Install project dependencies");
    cmd("add <pkg>", "Add a package dependency");
    cmd("dev", "Start the development server");
    cmd("build", "Build for production");
    cmd("test", "Run tests");
    cmd("lint", "Lint the codebase");
    cmd("format", "Format the codebase");
    cmd("template list", "List available project templates");
    cmd("template create <template> <project>", "Create project with version picker");
    cmd("deploy vercel", "Deploy to Vercel");
    println!();

    // ── Summary ─────────────────────────────────────────────────────────
    println!("  {} {}",
        Color::DIM.paint("──"),
        Color::DIM.paint("Klyron combines the best of Bun, Deno, pnpm, Vite, Prisma CLI, Laravel Installer & Cargo.")
    );
    println!("  {} {}",
        Color::DIM.paint("──"),
        Color::DIM.paint("Run 'klyron help <command>' or 'klyron <command> --help' for detailed info on any command.")
    );
    println!("  {} {}",
        Color::DIM.paint("──"),
        Color::DIM.paint("Run 'klyron template list' to see all available templates from adapters/.")
    );
    println!("  {} {}",
        Color::DIM.paint("──"),
        Color::DIM.paint(format!("Documentation: https://klyron.dev  |  Version: {}  |  by AniipID", VERSION))
    );
    println!();

    show_cursor();
}

fn section(name: &str) {
    println!("  {}", Color::BRIGHT_YELLOW.bold(name));
}

fn cmd(name: &str, desc: &str) {
    println!("    {}  {}{}",
        Color::GREEN.paint("▶"),
        Color::BRIGHT_CYAN.paint(format!("{:<28}", name)),
        Color::WHITE.paint(desc)
    );
}

fn flag(name: &str, desc: &str) {
    println!("    {}  {}{}",
        Color::YELLOW.paint("◆"),
        Color::BRIGHT_GREEN.paint(format!("{:<28}", name)),
        Color::WHITE.paint(desc)
    );
}

fn print_banner() {
    let colors = [
        (255, 56, 168),
        (200, 40, 255),
        (140, 80, 255),
        (80, 120, 255),
        (0, 200, 255),
        (0, 230, 180),
    ];

    let lines: Vec<&str> = BANNER.trim_end_matches('\n').lines().collect();
    let total_lines = lines.len();

    for (i, line) in lines.iter().enumerate() {
        let (r, g, b) = colors[i % colors.len()];
        // Per-character gradient within each line
        let gradient: String = line.chars().enumerate().map(|(j, c)| {
            let t = j as f64 / line.len().max(1) as f64;
            let next_i = (i + 1) % colors.len();
            let (r1, g1, b1) = colors[i % colors.len()];
            let (r2, g2, b2) = colors[next_i];
            let r = (r1 as f64 + (r2 as f64 - r1 as f64) * t) as u8;
            let g = (g1 as f64 + (g2 as f64 - g1 as f64) * t) as u8;
            let b = (b1 as f64 + (b2 as f64 - b1 as f64) * t) as u8;
            rgb_paint(r, g, b, &c.to_string())
        }).collect();
        println!("{}", gradient);
    }
}

fn animate_loading(seconds: u64) {
    use std::thread::sleep;
    use std::time::Duration;

    let steps = seconds * 25;
    let bar_width = 28;

    // Phase 1: Particle burst
    let particles = ['·', '∙', '•', '●', '○', '◌', '◍', '◎', '●'];
    for i in 0..8 {
        let phase = i as f64 / 8.0;
        let (r, g, b) = (200, 80 + (phase * 100.0) as u8, 255);
        let dot = rgb_paint(r, g, b, &particles[i as usize].to_string());
        let label = Color::DIM.paint("klyron");
        let dots = ".".repeat(i as usize + 1);
        print!("\r  {}  {}{}", dot, label, Color::DIM.paint(&dots));
        std::io::stdout().flush().ok();
        sleep(Duration::from_millis(30));
    }

    // Phase 2: Gradient scan bar with glow
    for i in 0..steps {
        let progress = i as f64 / steps as f64;
        let filled = (progress * bar_width as f64) as usize;

        let bar: String = (0..bar_width)
            .map(|j| {
                let t = j as f64 / bar_width as f64;
                let (r_base, g_base, b_base) = (
                    (180.0 + (1.0 - t) * 75.0) as u8,
                    (80.0 + t * 120.0) as u8,
                    (255.0 - t * 80.0) as u8,
                );
                if j < filled {
                    let pulse = (i as f64 * 0.4 + j as f64 * 0.6).sin() * 0.2 + 0.8;
                    let glow = if j == filled.saturating_sub(1) { 1.3 } else { 1.0 };
                    let (r2, g2, b2) = (
                        ((r_base as f64 * pulse * glow).min(255.0)) as u8,
                        ((g_base as f64 * pulse * glow).min(255.0)) as u8,
                        ((b_base as f64 * pulse * glow).min(255.0)) as u8,
                    );
                    rgb_paint(r2, g2, b2, "█")
                } else {
                    rgb_paint(40, 40, 60, "░")
                }
            })
            .collect();

        let pct = format!("{:>3}%", (progress * 100.0) as u8);
        let pct_color = rgb_paint(
            (200.0 - progress * 100.0) as u8,
            (80.0 + progress * 150.0) as u8,
            255,
            &pct,
        );
        let glow_r = (120.0 + progress * 120.0) as u8;
        let glow_g = (40.0 + progress * 80.0) as u8;
        let label = Color::DIM.paint("starting");
        print!("\r  {}  {}  {}", bar, pct_color, label);
        std::io::stdout().flush().ok();
        sleep(Duration::from_millis(40));
    }

    // Phase 3: Completion flash
    let full_bar: String = (0..bar_width)
        .map(|j| {
            let t = j as f64 / bar_width as f64;
            let (r, g, b) = (
                (180.0 + (1.0 - t) * 75.0) as u8,
                (80.0 + t * 150.0) as u8,
                (255.0 - t * 120.0) as u8,
            );
            rgb_paint(r, g, b, "█")
        })
        .collect();
    println!("\r  {}  {}  {}",
        full_bar,
        rgb_paint(0, 230, 180, "100%"),
        rgb_paint(0, 230, 180, "ready  ")
    );
}

pub fn show_version() {
    println!("{} {}",
        Color::YELLOW.paint("©"),
        Color::WHITE.bold(format!("klyron {}  —  by AniipID", VERSION))
    );
    println!("{} {}",
        Color::DIM.paint("  Universal Polyglot Runtime & Project Scaffolder"),
        Color::DIM.paint("")
    );
    println!("{} {}",
        Color::DIM.paint("  Code anything. Run anywhere. Use any package manager."),
        Color::DIM.paint("")
    );
}

pub fn show_spinner_init(msg: &str) -> crate::color::Spinner {
    crate::color::Spinner::new(msg)
}

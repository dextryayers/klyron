use std::path::PathBuf;

pub struct ShellCompletions;

impl ShellCompletions {
    pub fn generate_bash() -> String {
        format!(
            r#"#!/usr/bin/env bash
_klyron_completions() {{
    local cur prev words cword
    _init_completion || return

    local commands="{commands}"
    local global_opts="{global_opts}"
    local subcommands

    case $prev in
        {case_prev}
        *)
            if [[ $cword -eq 1 ]]; then
                COMPREPLY=($(compgen -W "$commands $global_opts" -- "$cur"))
            else
                local cmd=${{words[1]}}
                case $cmd in
                    {case_cmd}
                    *)
                        COMPREPLY=($(compgen -W "$global_opts" -- "$cur"))
                        ;;
                esac
            fi
            ;;
    esac
}} &&
complete -F _klyron_completions klyron
"#,
            commands = "eval run repl shell bundle cc cxx ts php py rb go zig rs js artisan composer blade tinker create create-next-app create-react-app create-angular create-vue create-svelte create-express create-fastify create-nest create-nuxt create-remix create-gatsby create-astro create-adonis create-laravel create-laravel-react create-laravel-vue create-laravel-inertia-react create-laravel-inertia-vue create-laravel-livewire create-laravel-next create-laravel-astro create-laravel-api create-django create-rails create-actix create-axum create-rocket create-solid create-qwik create-preact create-lit create-fast-api create-flask create-go-gin create-go-fiber create-go-echo create-tauri create-leptos create-symfony create-code-igniter create-word-press create-sveltekit create-hono create-koa create-hapi dev build test lint format check bench start run-script db prisma drizzle typeorm mikro-orm sequelize mongoose kysely knex orm install add remove uninstall outdated update audit dedupe lock publish unpublish login logout whoami search info pack link unlink dist-tag why workspace plugin cache docker napi deploy compat ai watch init upgrade doctor info version clean coverage telemetry config laravel serve completions",
            global_opts = "-h --help -V --version -v --verbose -q --quiet --engine --engine-pool-size --pre-warm --json",
            case_prev = r#"
        --engine) COMPREPLY=($(compgen -W "v8 boa quickjs jsc auto" -- "$cur")); return ;;
        --engine-pool-size) return ;;
        create|create-next-app|create-react-app) COMPREPLY=($(compgen -W "--list --versions" -- "$cur")); return ;;
        --shell) COMPREPLY=($(compgen -W "bash zsh fish powershell" -- "$cur")); return ;;
        --format) COMPREPLY=($(compgen -W "json pretty text" -- "$cur")); return ;;
        --output) _filedir; return ;;
"#,
            case_cmd = r#"
        eval) COMPREPLY=($(compgen -W "--ts --typescript --jsx --input-file $global_opts" -- "$cur"));;
        run) COMPREPLY=($(compgen -W "--watch $global_opts" -- "$cur")); _filedir;;
        repl|shell) COMPREPLY=($(compgen -W "$global_opts" -- "$cur"));;
        bundle) COMPREPLY=($(compgen -W "--output --minify $global_opts" -- "$cur")); _filedir;;
        cc|cxx|ts|php|py|rb|go|zig|rs|js) COMPREPLY=($(compgen -W "--watch --args $global_opts" -- "$cur"));;
        artisan) COMPREPLY=($(compgen -W "--project $global_opts" -- "$cur"));;
        composer) COMPREPLY=($(compgen -W "--project $global_opts" -- "$cur"));;
        blade) COMPREPLY=($(compgen -W "--data --project $global_opts" -- "$cur"));;
        tinker) COMPREPLY=($(compgen -W "--project $global_opts" -- "$cur"));;
        install) COMPREPLY=($(compgen -W "--frozen-lockfile $global_opts" -- "$cur"));;
        add) COMPREPLY=($(compgen -W "--dev $global_opts" -- "$cur"));;
        remove|uninstall) COMPREPLY=($(compgen -W "$global_opts" -- "$cur"));;
        outdated|update|audit|dedupe) COMPREPLY=($(compgen -W "$global_opts" -- "$cur"));;
        lock) COMPREPLY=($(compgen -W "verify update migrate $global_opts" -- "$cur"));;
        publish) COMPREPLY=($(compgen -W "--tag --access --registry --otp $global_opts" -- "$cur"));;
        unpublish) COMPREPLY=($(compgen -W "--registry --force $global_opts" -- "$cur"));;
        login|logout) COMPREPLY=($(compgen -W "--registry $global_opts" -- "$cur"));;
        search) COMPREPLY=($(compgen -W "--registry --json $global_opts" -- "$cur"));;
        info) COMPREPLY=($(compgen -W "--json $global_opts" -- "$cur"));;
        pack) COMPREPLY=($(compgen -W "--output $global_opts" -- "$cur"));;
        link) COMPREPLY=($(compgen -W "--global --global-dir $global_opts" -- "$cur"));;
        unlink) COMPREPLY=($(compgen -W "$global_opts" -- "$cur"));;
        dist-tag) COMPREPLY=($(compgen -W "add remove list $global_opts" -- "$cur"));;
        why) COMPREPLY=($(compgen -W "$global_opts" -- "$cur"));;
        workspace) COMPREPLY=($(compgen -W "list info add remove run $global_opts" -- "$cur"));;
        plugin) COMPREPLY=($(compgen -W "list install remove update $global_opts" -- "$cur"));;
        cache) COMPREPLY=($(compgen -W "clean list size $global_opts" -- "$cur"));;
        docker) COMPREPLY=($(compgen -W "build run push pull exec $global_opts" -- "$cur"));;
        napi) COMPREPLY=($(compgen -W "build build-release $global_opts" -- "$cur"));;
        deploy) COMPREPLY=($(compgen -W "--prod --preview $global_opts" -- "$cur"));;
        compat) COMPREPLY=($(compgen -W "--check --list $global_opts" -- "$cur"));;
        ai) COMPREPLY=($(compgen -W "--model --prompt --file --output $global_opts" -- "$cur"));;
        watch) COMPREPLY=($(compgen -W "--ext --ignore $global_opts" -- "$cur")); _filedir;;
        dev) COMPREPLY=($(compgen -W "--port --host --dir --watch --hot --no-hmr-inject $global_opts" -- "$cur"));;
        build) COMPREPLY=($(compgen -W "--out-dir --minify --sourcemap --watch $global_opts" -- "$cur"));;
        test) COMPREPLY=($(compgen -W "--watch --coverage --file --pattern $global_opts" -- "$cur"));;
        lint) COMPREPLY=($(compgen -W "--fix --file --pattern $global_opts" -- "$cur"));;
        format) COMPREPLY=($(compgen -W "--check --file --pattern $global_opts" -- "$cur"));;
        check) COMPREPLY=($(compgen -W "--files --all $global_opts" -- "$cur"));;
        bench) COMPREPLY=($(compgen -W "--output --compare --filter $global_opts" -- "$cur"));;
        db) COMPREPLY=($(compgen -W "migrate seed push pull status studio generate $global_opts" -- "$cur"));;
        prisma) COMPREPLY=($(compgen -W "generate db push db pull migrate studio validate format $global_opts" -- "$cur"));;
        drizzle) COMPREPLY=($(compgen -W "generate push pull migrate studio check introspect $global_opts" -- "$cur"));;
        typeorm) COMPREPLY=($(compgen -W "migration:run migration:generate migration:revert schema:sync $global_opts" -- "$cur"));;
        mikro-orm) COMPREPLY=($(compgen -W "migration:up migration:down schema:update $global_opts" -- "$cur"));;
        sequelize) COMPREPLY=($(compgen -W "db:migrate db:seed db:seed:all migration:generate seed:generate $global_opts" -- "$cur"));;
        mongoose) COMPREPLY=($(compgen -W "sync index $global_opts" -- "$cur"));;
        kysely) COMPREPLY=($(compgen -W "migrate:up migrate:down migrate:list $global_opts" -- "$cur"));;
        knex) COMPREPLY=($(compgen -W "migrate:up migrate:down migrate:latest seed:run $global_opts" -- "$cur"));;
        orm) COMPREPLY=($(compgen -W "prisma drizzle typeorm mikro-orm sequelize mongoose kysely knex $global_opts" -- "$cur"));;
        serve) COMPREPLY=($(compgen -W "--host --port --dir --watch $global_opts" -- "$cur"));;
        completions) COMPREPLY=($(compgen -W "bash zsh fish powershell" -- "$cur"));;
        doctor) COMPREPLY=($(compgen -W "--fix $global_opts" -- "$cur"));;
        coverage) COMPREPLY=($(compgen -W "--file --dir --output --format --html --lcov $global_opts" -- "$cur"));;
        telemetry) COMPREPLY=($(compgen -W "enable disable status view $global_opts" -- "$cur"));;
        config) COMPREPLY=($(compgen -W "get set list delete $global_opts" -- "$cur"));;
        clean) COMPREPLY=($(compgen -W "--yes $global_opts" -- "$cur"));;
        init|upgrade|version|whoami|start) COMPREPLY=($(compgen -W "$global_opts" -- "$cur"));;
        laravel) COMPREPLY=($(compgen -W "artisan serve tinker horizon telescope pulse nova $global_opts" -- "$cur"));;
        create-*) COMPREPLY=($(compgen -W "--version --dir $global_opts" -- "$cur"));;
"#,
        )
    }

    pub fn generate_zsh() -> String {
        format!(
            r#"#compdef klyron

_klyron() {{
    local context state state_descr line
    typeset -A opt_args

    _arguments -C \
        '(-h --help)'{{-h,--help}}'[Show help information]' \
        '(-V --version)'{{-V,--version}}'[Show version]' \
        '(-v --verbose)'{{-v,--verbose}}'[Increase verbosity]' \
        '(-q --quiet)'{{-q,--quiet}}'[Quiet mode]' \
        '--engine[Engine to use]:engine:(v8 boa quickjs jsc auto)' \
        '--engine-pool-size[Engine pool size]:number:' \
        '--pre-warm[Pre-warm engines]' \
        '--json[JSON output]' \
        '1: :->command' \
        '*:: :->args'

    case $state in
        command)
            local commands; commands=(
                'eval:Evaluate JavaScript/TypeScript code'
                'run:Run a file'
                'repl:Start interactive REPL'
                'shell:Start interactive shell'
                'bundle:Bundle JavaScript/TypeScript'
                'cc:Run C code'
                'cxx:Run C++ code'
                'ts:Run TypeScript code'
                'php:Run PHP code'
                'py:Run Python code'
                'rb:Run Ruby code'
                'go:Run Go code'
                'zig:Run Zig code'
                'rs:Run Rust code'
                'js:Run JavaScript code'
                'artisan:Run Laravel Artisan'
                'composer:Run PHP Composer'
                'blade:Render Blade template'
                'tinker:Run Laravel Tinker'
                'create:Scaffold new project'
                'create-next-app:Create Next.js app'
                'create-react-app:Create React app'
                'create-angular:Create Angular app'
                'create-vue:Create Vue app'
                'create-svelte:Create Svelte app'
                'create-express:Create Express app'
                'create-fastify:Create Fastify app'
                'create-nest:Create NestJS app'
                'create-nuxt:Create Nuxt app'
                'create-remix:Create Remix app'
                'create-gatsby:Create Gatsby app'
                'create-astro:Create Astro app'
                'create-adonis:Create AdonisJS app'
                'create-laravel:Create Laravel app'
                'create-django:Create Django app'
                'create-rails:Create Rails app'
                'create-solid:Create SolidJS app'
                'create-qwik:Create Qwik app'
                'create-preact:Create Preact app'
                'create-lit:Create Lit app'
                'create-tauri:Create Tauri app'
                'create-leptos:Create Leptos app'
                'create-sveltekit:Create SvelteKit app'
                'create-hono:Create Hono app'
                'create-koa:Create Koa app'
                'create-hapi:Create Hapi app'
                'dev:Start development server'
                'build:Build project'
                'test:Run tests'
                'lint:Lint project'
                'format:Format code'
                'check:Type check'
                'bench:Run benchmarks'
                'start:Start production server'
                'run-script:Run npm script'
                'db:Database operations'
                'prisma:Prisma ORM commands'
                'drizzle:Drizzle ORM commands'
                'typeorm:TypeORM commands'
                'mikro-orm:MikroORM commands'
                'sequelize:Sequelize commands'
                'mongoose:Mongoose commands'
                'kysely:Kysely commands'
                'knex:Knex commands'
                'orm:ORM commands'
                'install:Install dependencies'
                'add:Add package'
                'remove:Remove package'
                'uninstall:Uninstall package'
                'outdated:List outdated packages'
                'update:Update packages'
                'audit:Security audit'
                'dedupe:Deduplicate dependencies'
                'lock:Lockfile operations'
                'publish:Publish package'
                'unpublish:Unpublish package'
                'login:Login to registry'
                'logout:Logout from registry'
                'whoami:Show current user'
                'search:Search packages'
                'info:Show package info'
                'pack:Pack package'
                'link:Link package'
                'unlink:Unlink package'
                'dist-tag:Manage dist-tags'
                'why:Why package is needed'
                'workspace:Workspace operations'
                'plugin:Plugin management'
                'cache:Cache operations'
                'docker:Docker operations'
                'napi:NAPI build'
                'deploy:Deploy project'
                'compat:Compatibility check'
                'ai:AI operations'
                'watch:Watch files'
                'init:Initialize project'
                'upgrade:Upgrade klyron'
                'doctor:System diagnostics'
                'info:Show system info'
                'version:Show version'
                'clean:Clean artifacts'
                'coverage:Generate coverage'
                'telemetry:Telemetry settings'
                'config:Configuration'
                'laravel:Laravel commands'
                'serve:Serve static files'
                'completions:Generate completions'
            )
            _describe 'command' commands
            ;;
        args)
            case $line[1] in
                eval) _arguments '--tsx[JSX support]' '--typescript[TypeScript mode]' '--input-file[Input file]:file:_files' ;;
                run) _arguments '*:file:_files' '--watch[Watch mode]' ;;
                bundle) _arguments '1:entry file:_files' '--output[Output file]:file:_files' '--minify[Minify output]' ;;
                cc|cxx|ts|php|py|rb|go|zig|rs|js) _arguments '--watch[Watch mode]' '--args[Arguments]:string' ;;
                install) _arguments '--frozen-lockfile[Do not update lockfile]' ;;
                add) _arguments '--dev[Dev dependency]' '*:package:' ;;
                dev) _arguments '--port[Port]:port:' '--host[Host]:host:' '--dir[Directory]:dir:_files' '--watch[Watch]' '--hot[HMR]' ;;
                build) _arguments '--out-dir[Output dir]:dir:_files' '--minify[Minify]' '--sourcemap[Sourcemap]' ;;
                test) _arguments '--watch[Watch]' '--coverage[Coverage]' '--file[File]:file:_files' ;;
                completions) _arguments '1:shell:(bash zsh fish powershell)' ;;
                doctor) _arguments '--fix[Auto-fix issues]' ;;
                *) _arguments '*: :' ;;
            esac
            ;;
    esac
}}

_klyron "$@"
"#,
        )
    }

    pub fn generate_fish() -> String {
        format!(
            r#"#!/usr/bin/env fish

function __fish_klyron_using_command
    set -l cmd (commandline -opc)
    if [ (count $cmd) -ge 2 ]
        contains -- $cmd[2] $argv
    end
end

function __fish_klyron_needs_command
    set -l cmd (commandline -opc)
    [ (count $cmd) -eq 1 -a (commandline -t) != " " ]
end

# Global options
complete -c klyron -s h -l help -d "Show help"
complete -c klyron -s V -l version -d "Show version"
complete -c klyron -s v -l verbose -d "Increase verbosity"
complete -c klyron -s q -l quiet -d "Quiet mode"
complete -c klyron -l engine -x -a "v8 boa quickjs jsc auto" -d "JS engine"
complete -c klyron -l engine-pool-size -x -d "Engine pool size"
complete -c klyron -l pre-warm -d "Pre-warm engines"
complete -c klyron -l json -d "JSON output"

# Commands
complete -c klyron -n __fish_klyron_needs_command -a eval -d "Evaluate code"
complete -c klyron -n __fish_klyron_needs_command -a run -d "Run a file"
complete -c klyron -n __fish_klyron_needs_command -a repl -d "Start REPL"
complete -c klyron -n __fish_klyron_needs_command -a shell -d "Start shell"
complete -c klyron -n __fish_klyron_needs_command -a bundle -d "Bundle code"
complete -c klyron -n __fish_klyron_needs_command -a cc -d "Run C code"
complete -c klyron -n __fish_klyron_needs_command -a cxx -d "Run C++ code"
complete -c klyron -n __fish_klyron_needs_command -a ts -d "Run TypeScript"
complete -c klyron -n __fish_klyron_needs_command -a php -d "Run PHP"
complete -c klyron -n __fish_klyron_needs_command -a py -d "Run Python"
complete -c klyron -n __fish_klyron_needs_command -a rb -d "Run Ruby"
complete -c klyron -n __fish_klyron_needs_command -a go -d "Run Go"
complete -c klyron -n __fish_klyron_needs_command -a zig -d "Run Zig"
complete -c klyron -n __fish_klyron_needs_command -a rs -d "Run Rust"
complete -c klyron -n __fish_klyron_needs_command -a js -d "Run JavaScript"
complete -c klyron -n __fish_klyron_needs_command -a artisan -d "Laravel Artisan"
complete -c klyron -n __fish_klyron_needs_command -a composer -d "PHP Composer"
complete -c klyron -n __fish_klyron_needs_command -a blade -d "Blade template"
complete -c klyron -n __fish_klyron_needs_command -a tinker -d "Laravel Tinker"
complete -c klyron -n __fish_klyron_needs_command -a create -d "Scaffold project"
complete -c klyron -n __fish_klyron_needs_command -a dev -d "Dev server"
complete -c klyron -n __fish_klyron_needs_command -a build -d "Build project"
complete -c klyron -n __fish_klyron_needs_command -a test -d "Run tests"
complete -c klyron -n __fish_klyron_needs_command -a lint -d "Lint project"
complete -c klyron -n __fish_klyron_needs_command -a format -d "Format code"
complete -c klyron -n __fish_klyron_needs_command -a check -d "Type check"
complete -c klyron -n __fish_klyron_needs_command -a bench -d "Run benchmarks"
complete -c klyron -n __fish_klyron_needs_command -a start -d "Start server"
complete -c klyron -n __fish_klyron_needs_command -a install -d "Install deps"
complete -c klyron -n __fish_klyron_needs_command -a add -d "Add package"
complete -c klyron -n __fish_klyron_needs_command -a remove -d "Remove package"
complete -c klyron -n __fish_klyron_needs_command -a uninstall -d "Uninstall package"
complete -c klyron -n __fish_klyron_needs_command -a outdated -d "Outdated packages"
complete -c klyron -n __fish_klyron_needs_command -a update -d "Update packages"
complete -c klyron -n __fish_klyron_needs_command -a audit -d "Security audit"
complete -c klyron -n __fish_klyron_needs_command -a dedupe -d "Deduplicate"
complete -c klyron -n __fish_klyron_needs_command -a lock -d "Lockfile ops"
complete -c klyron -n __fish_klyron_needs_command -a publish -d "Publish package"
complete -c klyron -n __fish_klyron_needs_command -a unpublish -d "Unpublish"
complete -c klyron -n __fish_klyron_needs_command -a login -d "Login"
complete -c klyron -n __fish_klyron_needs_command -a logout -d "Logout"
complete -c klyron -n __fish_klyron_needs_command -a whoami -d "Show user"
complete -c klyron -n __fish_klyron_needs_command -a search -d "Search packages"
complete -c klyron -n __fish_klyron_needs_command -a info -d "Package info"
complete -c klyron -n __fish_klyron_needs_command -a pack -d "Pack package"
complete -c klyron -n __fish_klyron_needs_command -a link -d "Link package"
complete -c klyron -n __fish_klyron_needs_command -a unlink -d "Unlink"
complete -c klyron -n __fish_klyron_needs_command -a workspace -d "Workspace ops"
complete -c klyron -n __fish_klyron_needs_command -a plugin -d "Plugin mgmt"
complete -c klyron -n __fish_klyron_needs_command -a cache -d "Cache ops"
complete -c klyron -n __fish_klyron_needs_command -a docker -d "Docker ops"
complete -c klyron -n __fish_klyron_needs_command -a napi -d "NAPI build"
complete -c klyron -n __fish_klyron_needs_command -a deploy -d "Deploy"
complete -c klyron -n __fish_klyron_needs_command -a compat -d "Compat check"
complete -c klyron -n __fish_klyron_needs_command -a ai -d "AI operations"
complete -c klyron -n __fish_klyron_needs_command -a watch -d "Watch files"
complete -c klyron -n __fish_klyron_needs_command -a init -d "Initialize"
complete -c klyron -n __fish_klyron_needs_command -a upgrade -d "Upgrade klyron"
complete -c klyron -n __fish_klyron_needs_command -a doctor -d "Diagnostics"
complete -c klyron -n __fish_klyron_needs_command -a version -d "Version"
complete -c klyron -n __fish_klyron_needs_command -a clean -d "Clean"
complete -c klyron -n __fish_klyron_needs_command -a coverage -d "Coverage"
complete -c klyron -n __fish_klyron_needs_command -a telemetry -d "Telemetry"
complete -c klyron -n __fish_klyron_needs_command -a config -d "Config"
complete -c klyron -n __fish_klyron_needs_command -a laravel -d "Laravel"
complete -c klyron -n __fish_klyron_needs_command -a serve -d "Static serve"
complete -c klyron -n __fish_klyron_needs_command -a completions -d "Completions"
complete -c klyron -n __fish_klyron_needs_command -a orm -d "ORM commands"
complete -c klyron -n __fish_klyron_needs_command -a prisma -d "Prisma"
complete -c klyron -n __fish_klyron_needs_command -a drizzle -d "Drizzle"
complete -c klyron -n __fish_klyron_needs_command -a typeorm -d "TypeORM"
complete -c klyron -n __fish_klyron_needs_command -a db -d "Database"

# Command-specific completions
complete -c klyron -n "__fish_klyron_using_command eval" -l tsx -d "JSX support"
complete -c klyron -n "__fish_klyron_using_command eval" -l typescript -d "TS mode"
complete -c klyron -n "__fish_klyron_using_command eval" -l input-file -r -d "Input file"
complete -c klyron -n "__fish_klyron_using_command run" -l watch -d "Watch mode"
complete -c klyron -n "__fish_klyron_using_command run" -a "(__fish_complete_suffix .js .ts .mjs .cjs)"
complete -c klyron -n "__fish_klyron_using_command bundle" -l output -r -d "Output file"
complete -c klyron -n "__fish_klyron_using_command bundle" -l minify -d "Minify"
complete -c klyron -n "__fish_klyron_using_command add" -l dev -d "Dev dependency"
complete -c klyron -n "__fish_klyron_using_command install" -l frozen-lockfile -d "Frozen lockfile"
complete -c klyron -n "__fish_klyron_using_command dev" -l port -r -d "Port"
complete -c klyron -n "__fish_klyron_using_command dev" -l host -r -d "Host"
complete -c klyron -n "__fish_klyron_using_command dev" -l dir -r -a "(__fish_complete_directories)" -d "Directory"
complete -c klyron -n "__fish_klyron_using_command dev" -l watch -d "Watch"
complete -c klyron -n "__fish_klyron_using_command dev" -l hot -d "HMR"
complete -c klyron -n "__fish_klyron_using_command test" -l watch -d "Watch"
complete -c klyron -n "__fish_klyron_using_command test" -l coverage -d "Coverage"
complete -c klyron -n "__fish_klyron_using_command doctor" -l fix -d "Auto-fix"
complete -c klyron -n "__fish_klyron_using_command completions" -a "bash zsh fish powershell"
complete -c klyron -n "__fish_klyron_using_command lock" -a "verify update migrate"
complete -c klyron -n "__fish_klyron_using_command telemetry" -a "enable disable status view"
complete -c klyron -n "__fish_klyron_using_command config" -a "get set list delete"
complete -c klyron -n "__fish_klyron_using_command workspace" -a "list info add remove run"
complete -c klyron -n "__fish_klyron_using_command cache" -a "clean list size"
complete -c klyron -n "__fish_klyron_using_command docker" -a "build run push pull exec"
complete -c klyron -n "__fish_klyron_using_command plugin" -a "list install remove update"
complete -c klyron -n "__fish_klyron_using_command laravel" -a "artisan serve tinker horizon telescope pulse nova"
"#,
        )
    }

    pub fn generate_powershell() -> String {
        format!(
            r#"# PowerShell completion for klyron

Register-ArgumentCompleter -Native -CommandName 'klyron' -ScriptBlock {{
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = @(
        'eval', 'run', 'repl', 'shell', 'bundle',
        'cc', 'cxx', 'ts', 'php', 'py', 'rb', 'go', 'zig', 'rs', 'js',
        'artisan', 'composer', 'blade', 'tinker',
        'create', 'create-next-app', 'create-react-app', 'create-angular',
        'create-vue', 'create-svelte', 'create-express', 'create-fastify',
        'create-nest', 'create-nuxt', 'create-remix', 'create-gatsby',
        'create-astro', 'create-adonis', 'create-laravel',
        'create-laravel-react', 'create-laravel-vue', 'create-laravel-inertia-react',
        'create-laravel-inertia-vue', 'create-laravel-livewire',
        'create-laravel-next', 'create-laravel-astro', 'create-laravel-api',
        'create-django', 'create-rails', 'create-actix', 'create-axum',
        'create-rocket', 'create-solid', 'create-qwik', 'create-preact',
        'create-lit', 'create-fast-api', 'create-flask',
        'create-go-gin', 'create-go-fiber', 'create-go-echo',
        'create-tauri', 'create-leptos', 'create-symfony',
        'create-code-igniter', 'create-word-press', 'create-sveltekit',
        'create-hono', 'create-koa', 'create-hapi',
        'dev', 'build', 'test', 'lint', 'format', 'check', 'bench',
        'start', 'run-script',
        'db', 'prisma', 'drizzle', 'typeorm', 'mikro-orm', 'sequelize',
        'mongoose', 'kysely', 'knex', 'orm',
        'install', 'add', 'remove', 'uninstall', 'outdated', 'update',
        'audit', 'dedupe', 'lock',
        'publish', 'unpublish', 'login', 'logout', 'whoami', 'search',
        'info', 'pack', 'link', 'unlink', 'dist-tag', 'why',
        'workspace', 'plugin', 'cache', 'docker', 'napi',
        'deploy', 'compat', 'ai', 'watch',
        'init', 'upgrade', 'doctor', 'info', 'version', 'clean',
        'coverage', 'telemetry', 'config',
        'laravel', 'serve', 'completions'
    )

    $globalOptions = @('-h', '--help', '-V', '--version', '-v', '--verbose',
                       '-q', '--quiet', '--engine', '--engine-pool-size',
                       '--pre-warm', '--json')

    $commandMap = @{{
        'eval' = @('--tsx', '--typescript', '--input-file')
        'run' = @('--watch')
        'bundle' = @('--output', '--minify')
        'add' = @('--dev')
        'install' = @('--frozen-lockfile')
        'dev' = @('--port', '--host', '--dir', '--watch', '--hot', '--no-hmr-inject')
        'build' = @('--out-dir', '--minify', '--sourcemap', '--watch')
        'test' = @('--watch', '--coverage', '--file', '--pattern')
        'lint' = @('--fix', '--file', '--pattern')
        'format' = @('--check', '--file', '--pattern')
        'check' = @('--files', '--all')
        'bench' = @('--output', '--compare', '--filter')
        'publish' = @('--tag', '--access', '--registry', '--otp')
        'unpublish' = @('--registry', '--force')
        'login' = @('--registry')
        'logout' = @('--registry')
        'search' = @('--registry', '--json')
        'info' = @('--json')
        'pack' = @('--output')
        'link' = @('--global', '--global-dir')
        'deploy' = @('--prod', '--preview')
        'compat' = @('--check', '--list')
        'ai' = @('--model', '--prompt', '--file', '--output')
        'watch' = @('--ext', '--ignore')
        'doctor' = @('--fix')
        'clean' = @('--yes')
        'coverage' = @('--file', '--dir', '--output', '--format', '--html', '--lcov')
        'serve' = @('--host', '--port', '--dir', '--watch')
        'completions' = @('bash', 'zsh', 'fish', 'powershell')
        'lock' = @('verify', 'update', 'migrate')
        'telemetry' = @('enable', 'disable', 'status', 'view')
        'config' = @('get', 'set', 'list', 'delete')
        'workspace' = @('list', 'info', 'add', 'remove', 'run')
        'plugin' = @('list', 'install', 'remove', 'update')
        'cache' = @('clean', 'list', 'size')
        'docker' = @('build', 'run', 'push', 'pull', 'exec')
        'napi' = @('build', 'build-release')
        'db' = @('migrate', 'seed', 'push', 'pull', 'status', 'studio', 'generate')
        'prisma' = @('generate', 'db', 'migrate', 'studio', 'validate', 'format')
        'drizzle' = @('generate', 'push', 'pull', 'migrate', 'studio', 'check', 'introspect')
        'laravel' = @('artisan', 'serve', 'tinker', 'horizon', 'telescope', 'pulse', 'nova')
        'orm' = @('prisma', 'drizzle', 'typeorm', 'mikro-orm', 'sequelize', 'mongoose', 'kysely', 'knex')
        'dist-tag' = @('add', 'remove', 'list')
    }}

    $words = $commandAst.CommandElements
    $cmdIndex = 1
    $currentCommand = $null

    for ($i = 1; $i -lt $words.Count; $i++) {{
        $w = $words[$i].Value
        if ($w -and !$w.StartsWith('-')) {{
            $currentCommand = $w
            $cmdIndex = $i
            break
        }}
    }}

    $completions = @()

    if (-not $currentCommand -or $cmdIndex -eq ($words.Count - 1)) {{
        $completions += $commands | Where-Object {{ $_ -like "$wordToComplete*" }} | ForEach-Object {{
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }}
    }}

    if ($currentCommand -and $commandMap.ContainsKey($currentCommand)) {{
        $opts = $commandMap[$currentCommand]
        $completions += $opts | Where-Object {{ $_ -like "$wordToComplete*" }} | ForEach-Object {{
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterName', $_)
        }}
    }}

    $completions += $globalOptions | Where-Object {{ $_ -like "$wordToComplete*" }} | ForEach-Object {{
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterName', $_)
    }}

    $completions | Sort-Object -Unique
}}
"#,
        )
    }

    pub fn detect_shell() -> Option<String> {
        std::env::var("SHELL").ok().and_then(|s| {
            let s = s.to_lowercase();
            if s.contains("bash") { Some("bash".into()) }
            else if s.contains("zsh") { Some("zsh".into()) }
            else if s.contains("fish") { Some("fish".into()) }
            else if s.contains("powershell") || s.contains("pwsh") { Some("powershell".into()) }
            else { None }
        })
    }

    pub fn install(shell: &str) -> Result<(), String> {
        let content = match shell {
            "bash" => Self::generate_bash(),
            "zsh" => Self::generate_zsh(),
            "fish" => Self::generate_fish(),
            "powershell" => Self::generate_powershell(),
            other => return Err(format!("Unsupported shell: {other}. Supported: bash, zsh, fish, powershell")),
        };

        let path = Self::completion_path(shell)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create dir: {e}"))?;
        }
        std::fs::write(&path, &content).map_err(|e| format!("Cannot write {path:?}: {e}"))?;

        let rc_line = Self::rc_line(shell, &path);
        let rc_file = Self::rc_file(shell);
        if let Some(rc) = rc_file {
            let existing = std::fs::read_to_string(&rc).unwrap_or_default();
            if !existing.contains(&rc_line) {
                let mut file = std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(&rc)
                    .map_err(|e| format!("Cannot open {rc:?}: {e}"))?;
                use std::io::Write;
                writeln!(file, "\n# klyron completions\n{rc_line}")
                    .map_err(|e| format!("Cannot write to {rc:?}: {e}"))?;
            }
        }

        Ok(())
    }

    pub fn uninstall(shell: &str) -> Result<(), String> {
        let path = Self::completion_path(shell)?;
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| format!("Cannot remove {path:?}: {e}"))?;
        }

        let rc_file = Self::rc_file(shell);
        if let Some(rc) = rc_file {
            if rc.exists() {
                let content = std::fs::read_to_string(&rc).map_err(|e| format!("Cannot read {rc:?}: {e}"))?;
                let lines: Vec<&str> = content.lines()
                    .filter(|l| !l.contains("klyron"))
                    .collect();
                std::fs::write(&rc, lines.join("\n")).map_err(|e| format!("Cannot write {rc:?}: {e}"))?;
            }
        }

        Ok(())
    }

    fn completion_path(shell: &str) -> Result<PathBuf, String> {
        let home = dirs::home_dir().ok_or("Cannot determine home directory")?;
        match shell {
            "bash" => Ok(home.join(".local/share/bash-completion/completions/klyron")),
            "zsh" => Ok(home.join(".zsh/completions/_klyron")),
            "fish" => Ok(home.join(".config/fish/completions/klyron.fish")),
            "powershell" => {
                let profile = std::env::var("PROFILE").unwrap_or_else(|_| {
                    home.join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1")
                        .to_string_lossy().to_string()
                });
                Ok(PathBuf::from(profile))
            }
            _ => Err(format!("Unsupported shell: {shell}")),
        }
    }

    fn rc_line(shell: &str, path: &PathBuf) -> String {
        match shell {
            "bash" => format!("source {}", path.display()),
            "zsh" => format!("source {}", path.display()),
            "fish" => String::new(),
            "powershell" => format!(". {}", path.display()),
            _ => String::new(),
        }
    }

    fn rc_file(shell: &str) -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        match shell {
            "bash" => Some(home.join(".bashrc")),
            "zsh" => Some(home.join(".zshrc")),
            "fish" => Some(home.join(".config/fish/config.fish")),
            "powershell" => None,
            _ => None,
        }
    }
}

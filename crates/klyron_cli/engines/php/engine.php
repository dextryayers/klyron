<?php

final class PhpEngine {
    private const MAX_OUTPUT = 1 << 20;
    private const TIMEOUT = 30;

    private string $projectRoot;
    private string $cacheDir;
    private array $sections = [];
    private array $sectionStack = [];
    private array $pushStack = [];
    private string $parentView = '';
    private array $viewData = [];

    public function __construct() {
        $this->projectRoot = getenv('KLYRON_PROJECT_ROOT') ?: getcwd();
        $this->cacheDir = $this->projectRoot . '/storage/framework/views';
        if (!is_dir($this->cacheDir)) {
            @mkdir($this->cacheDir, 0777, true);
        }
    }

    public function handle(string $jsonLine): void {
        try {
            $input = json_decode($jsonLine, true, 8, JSON_THROW_ON_ERROR);
        } catch (\JsonException $e) {
            $this->output('', 'Invalid JSON: ' . $e->getMessage(), 1, '');
            return;
        }

        $action = $input['action'] ?? '';
        $code = $input['code'] ?? '';
        $args = $input['args'] ?? '';
        $project = $input['project'] ?? $this->projectRoot;
        $files = $input['files'] ?? [];
        $filename = $input['filename'] ?? '';

        try {
            match (true) {
                // ── Core ──
                $action === 'exec' => $this->handleExec($code, $files, $filename),
                $action === 'file' => $this->handleFile($code ?: $args),
                $action === 'eval' => $this->handleEval($code),
                $action === 'check' => $this->handleCheck($code),
                $action === 'ping' => $this->output('pong', '', 0, 'ok'),

                // ── Blade ──
                $action === 'blade' => $this->handleBlade($code, $args, $project),
                $action === 'blade:clear' => $this->handleBladeClear($project),

                // ── Composer ──
                $action === 'composer' => $this->handleComposer($args, $project),

                // ── Artisan pass-through ──
                $action === 'artisan' => $this->handleArtisan($args, $project),
                $action === 'artisan:serve' => $this->handleArtisanServe($args, $project),
                $action === 'artisan:make' => $this->handleArtisanMake($args, $project),
                $action === 'artisan:migrate' => $this->handleArtisanMigrate($project),
                $action === 'artisan:tinker' => $this->handleArtisanTinker($project),

                // ── Artisan make:resource (special: make:controller with --resource --model) ──
                $action === 'artisan:resource' => $this->runArtisanMake('controller', $code, $project, '--resource --model=' . $code),

                // ── Generic artisan make:* commands (catches artisan:model, artisan:cast, artisan:test, etc.) ──
                str_starts_with($action, 'artisan:') => $this->runArtisanMake(substr($action, 8), $code, $project, $args),

                // ── Horizon ──
                $action === 'horizon' => $this->runArtisanCommand('horizon', $project),
                $action === 'horizon:install' => $this->runArtisanCommand('horizon:install', $project),
                $action === 'horizon:pause' => $this->runArtisanCommand('horizon:pause', $project),
                $action === 'horizon:continue' => $this->runArtisanCommand('horizon:continue', $project),
                $action === 'horizon:terminate' => $this->runArtisanCommand('horizon:terminate', $project),
                $action === 'horizon:status' => $this->runArtisanCommand('horizon:status', $project),

                // ── Telescope ──
                $action === 'telescope:install' => $this->runArtisanCommand('telescope:install', $project),
                $action === 'telescope:prune' => $this->runArtisanCommand('telescope:prune', $project),
                $action === 'telescope:clear' => $this->runArtisanCommand('telescope:clear', $project),

                // ── Sail ──
                $action === 'sail' => $this->handleSail($args, $project),

                // ── Pennant ──
                $action === 'pennant:install' => $this->runArtisanCommand('pennant:install', $project),
                $action === 'pennant:feature' => $this->runArtisanCommand('pennant:feature ' . escapeshellarg($code), $project),

                // ── Pulse ──
                $action === 'pulse:install' => $this->runArtisanCommand('pulse:install', $project),
                $action === 'pulse:check' => $this->runArtisanCommand('pulse:check', $project),

                // ── Routes ──
                $action === 'routes' => $this->runArtisanCommand('route:list', $project),

                // ── Cache Management ──
                $action === 'cache:clear' => $this->runArtisanCommand('cache:clear', $project),
                $action === 'config:clear' => $this->runArtisanCommand('config:clear', $project),
                $action === 'route:clear' => $this->runArtisanCommand('route:clear', $project),
                $action === 'view:clear' => $this->runArtisanCommand('view:clear', $project),

                // ── Queue Management ──
                $action === 'queue:work' => $this->runArtisanCommand('queue:work', $project),
                $action === 'queue:listen' => $this->runArtisanCommand('queue:listen', $project),
                $action === 'queue:restart' => $this->runArtisanCommand('queue:restart', $project),

                // ── Schedule ──
                $action === 'schedule:run' => $this->runArtisanCommand('schedule:run', $project),
                $action === 'schedule:list' => $this->runArtisanCommand('schedule:list', $project),

                // ── Format / Lint / Test ──
                $action === 'format' => $this->handleFormat($code),
                $action === 'lint' => $this->handleLint($code),
                $action === 'test' => $this->handleTest($code, $files),

                default => $this->output('', "Unknown action: $action", 1, ''),
            };
        } catch (\Throwable $e) {
            $this->output('', $e->getMessage() . "\n" . $e->getTraceAsString(), 1, '');
        }
    }

    private function output(string $stdout, string $stderr, int $exitCode, string $result, ?array $diags = null): void {
        $out = [
            'stdout' => $stdout,
            'stderr' => $stderr,
            'exit_code' => $exitCode,
            'result' => $result,
        ];
        if ($diags !== null) $out['diagnostics'] = $diags;
        echo json_encode($out, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES) . "\n";
    }

    private function runCommand(string $cmd, ?string $cwd = null): array {
        $descriptors = [
            0 => ['pipe', 'r'],
            1 => ['pipe', 'w'],
            2 => ['pipe', 'w'],
        ];
        $process = proc_open($cmd, $descriptors, $pipes, $cwd);
        if (!$process) return ['stdout' => '', 'stderr' => 'Failed to start process', 'exit_code' => -1];

        fclose($pipes[0]);
        $stdout = ''; $stderr = '';
        $running = true;
        $startTime = time();

        while ($running) {
            $read = [$pipes[1], $pipes[2]];
            $write = null; $except = null;
            if (false === stream_select($read, $write, $except, 1)) break;
            foreach ($read as $r) {
                $data = fread($r, 65536);
                if ($data === false || $data === '') continue;
                if ($r === $pipes[1]) $stdout .= $data; else $stderr .= $data;
            }
            $status = proc_get_status($process);
            if (!$status['running']) {
                $running = false;
                if ($status['exitcode'] === -1) {
                    while ($buf = fread($pipes[1], 65536)) $stdout .= $buf;
                }
            }
            if (time() - $startTime > self::TIMEOUT) {
                proc_terminate($process, 15);
                $stderr .= "\n[Timeout]";
                $killTime = time() + 3;
                while (time() < $killTime) {
                    $status = proc_get_status($process);
                    if (!$status['running']) {
                        $running = false;
                        break;
                    }
                    usleep(100000);
                }
                if ($running) {
                    proc_terminate($process, 9);
                }
                $stderr .= "\n[Killed]";
                $running = false;
            }
        }

        while ($buf = fread($pipes[1], 65536)) $stdout .= $buf;
        while ($buf = fread($pipes[2], 65536)) $stderr .= $buf;
        fclose($pipes[1]); fclose($pipes[2]);
        $exitCode = proc_close($process);

        if (strlen($stdout) > self::MAX_OUTPUT) $stdout = substr($stdout, 0, self::MAX_OUTPUT) . "\n... [truncated]";
        if (strlen($stderr) > self::MAX_OUTPUT) $stderr = substr($stderr, 0, self::MAX_OUTPUT) . "\n... [truncated]";
        return ['stdout' => $stdout, 'stderr' => $stderr, 'exit_code' => $exitCode];
    }

    // ─── Artisan helpers ─────────────────────────────────────────────────

    private function findArtisan(string $project): ?string {
        foreach (["$project/artisan", "$project/bin/artisan"] as $path) {
            if (file_exists($path)) return realpath($path) ?: $path;
        }
        return null;
    }

    private function runArtisanCommand(string $subcommand, string $project): void {
        $artisan = $this->findArtisan($project);
        if (!$artisan) { $this->output('', 'No artisan file found.', 1, ''); return; }
        $result = $this->runCommand(
            "php " . escapeshellarg($artisan) . " " . $subcommand,
            $project
        );
        $this->output($result['stdout'], $result['stderr'], $result['exit_code'], $result['stdout']);
    }

    private function runArtisanMake(string $type, string $name, string $project, string $extraArgs = ''): void {
        $artisan = $this->findArtisan($project);
        if (!$artisan) { $this->output('', 'No artisan file found.', 1, ''); return; }
        if (empty($name)) { $this->output('', "No name provided for $type.", 1, ''); return; }
        $cmd = "php " . escapeshellarg($artisan) . " make:$type $name";
        if ($extraArgs !== '') {
            $cmd .= " " . $extraArgs;
        }
        $result = $this->runCommand($cmd, $project);
        $this->output($result['stdout'], $result['stderr'], $result['exit_code'], $result['stdout']);
    }

    // ─── Execute PHP code ───────────────────────────────────────────────

    private function handleExec(string $code, array $files, string $filename): void {
        if (empty($code) && empty($files)) { $this->output('', 'No code provided', 1, ''); return; }

        if (!empty($files)) {
            $tmpDir = sys_get_temp_dir() . '/klyron_php_' . bin2hex(random_bytes(8));
            mkdir($tmpDir, 0700, true);
            foreach ($files as $f) {
                file_put_contents("$tmpDir/" . basename($f['name']), $f['content']);
            }
            if (!empty($code)) {
                file_put_contents("$tmpDir/" . ($filename ?: 'main.php'), $code);
            }
            $entry = $filename ? "$tmpDir/$filename" : "$tmpDir/main.php";
            if (file_exists($entry)) {
                $result = $this->runCommand('php ' . escapeshellarg($entry));
                $this->output($result['stdout'], $result['stderr'], $result['exit_code'], $result['stdout']);
            }
            $this->delTree($tmpDir);
            return;
        }

        ob_start();
        try {
            $result = eval($code);
            $stdout = ob_get_clean() ?: '';
            $this->output($stdout, '', 0, (string)($result ?? ''));
        } catch (\Throwable $e) {
            ob_end_clean();
            $this->output('', $e->getMessage() . "\n" . $e->getTraceAsString(), 1, '');
        }
    }

    private function handleFile(string $path): void {
        if (empty($path)) { $this->output('', 'No file path provided', 1, ''); return; }
        if (!file_exists($path)) { $this->output('', "File not found: $path", 1, ''); return; }
        $result = $this->runCommand('php ' . escapeshellarg($path));
        $this->output($result['stdout'], $result['stderr'], $result['exit_code'], $result['stdout']);
    }

    private function handleEval(string $expr): void {
        if (empty($expr)) { $this->output('', 'No expression provided', 1, ''); return; }
        try {
            $result = eval("return $expr;");
            $this->output((string)($result ?? ''), '', 0, json_encode($result));
        } catch (\Throwable $e) {
            $this->output('', $e->getMessage(), 1, '');
        }
    }

    private function handleCheck(string $code): void {
        if (empty($code)) { $this->output('', 'No code provided', 1, ''); return; }
        $tmpFile = tempnam(sys_get_temp_dir(), 'php_check_') . '.php';
        file_put_contents($tmpFile, "<?php $code");
        $result = $this->runCommand('php -l ' . escapeshellarg($tmpFile));
        unlink($tmpFile);
        $diags = [];
        if ($result['exit_code'] !== 0) {
            $diags[] = ['file' => '<eval>', 'line' => 0, 'col' => 0, 'message' => $result['stderr'], 'severity' => 'error'];
        }
        $this->output($result['stdout'], $result['stderr'], $result['exit_code'],
            $result['exit_code'] === 0 ? 'No syntax errors' : 'Syntax error', $diags);
    }

    // ─── Artisan ────────────────────────────────────────────────────────

    private function handleArtisan(string $args, string $project): void {
        $artisan = $this->findArtisan($project);
        if (!$artisan) { $this->output('', 'No artisan file found.', 1, ''); return; }
        $result = $this->runCommand("php " . escapeshellarg($artisan) . " " . escapeshellarg($args) . " 2>&1", $project);
        $this->output($result['stdout'], $result['stderr'], $result['exit_code'], $result['stdout']);
    }

    private function handleArtisanServe(string $args, string $project): void {
        $artisan = $this->findArtisan($project);
        if (!$artisan) { $this->output('', 'No artisan file found.', 1, ''); return; }
        $result = $this->runCommand("php " . escapeshellarg($artisan) . " serve " . escapeshellarg($args), $project);
        $this->output($result['stdout'], $result['stderr'], $result['exit_code'], $result['stdout']);
    }

    private function handleArtisanMake(string $args, string $project): void {
        $artisan = $this->findArtisan($project);
        if (!$artisan) { $this->output('', 'No artisan file found.', 1, ''); return; }
        $result = $this->runCommand("php " . escapeshellarg($artisan) . " make:$args", $project);
        $this->output($result['stdout'], $result['stderr'], $result['exit_code'], $result['stdout']);
    }

    private function handleArtisanMigrate(string $project): void {
        $artisan = $this->findArtisan($project);
        if (!$artisan) { $this->output('', 'No artisan file found.', 1, ''); return; }
        $result = $this->runCommand("php " . escapeshellarg($artisan) . " migrate --force", $project);
        $this->output($result['stdout'], $result['stderr'], $result['exit_code'], $result['stdout']);
    }

    private function handleArtisanTinker(string $project): void {
        $artisan = $this->findArtisan($project);
        if (!$artisan) { $this->output('', 'No artisan file found.', 1, ''); return; }
        $cmd = "php " . escapeshellarg($artisan) . " tinker";
        passthru($cmd, $exitCode);
        $this->output('', '', $exitCode, '');
    }

    // ─── Sail ────────────────────────────────────────────────────────────

    private function handleSail(string $args, string $project): void {
        $sail = $project . '/vendor/bin/sail';
        if (!file_exists($sail)) { $this->output('', 'Sail not found.', 1, ''); return; }
        $result = $this->runCommand(escapeshellarg($sail) . " " . $args, $project);
        $this->output($result['stdout'], $result['stderr'], $result['exit_code'], $result['stdout']);
    }

    private function handleComposer(string $args, string $project): void {
        $result = $this->runCommand("composer --working-dir=" . escapeshellarg($project) . " " . escapeshellarg($args) . " 2>&1", $project);
        $this->output($result['stdout'], $result['stderr'], $result['exit_code'], $result['stdout']);
    }

    // ─── Blade Compiler ─────────────────────────────────────────────────

    private function compileBlade(string $content): string {
        // Comments
        $content = preg_replace('/\{\{--[\s\S]*?--\}\}/', '', $content);

        // PHP blocks
        $content = preg_replace('/@php\s*/', '<?php ', $content);
        $content = preg_replace('/@endphp/', '?>', $content);

        // Echo
        $content = preg_replace('/\{\{\s*(.+?)\s*\}\}/', '<?= htmlspecialchars($1 ?? "", ENT_QUOTES, \'UTF-8\') ?>', $content);
        $content = preg_replace('/\{\{!\s*(.+?)\s*!\}\}/', '<?= $1 ?>', $content);

        // Raw PHP echo
        $content = preg_replace('/@verbatim\s*([\s\S]*?)\s*@endverbatim/', '$1', $content);

        // Control structures
        $content = preg_replace('/@if\s*\((.+?)\)/', '<?php if ($1): ?>', $content);
        $content = preg_replace('/@elseif\s*\((.+?)\)/', '<?php elseif ($1): ?>', $content);
        $content = preg_replace('/@else/', '<?php else: ?>', $content);
        $content = preg_replace('/@endif/', '<?php endif; ?>', $content);

        $content = preg_replace('/@unless\s*\((.+?)\)/', '<?php if (!($1)): ?>', $content);
        $content = preg_replace('/@endunless/', '<?php endif; ?>', $content);

        $content = preg_replace('/@foreach\s*\((.+?)\)/', '<?php foreach ($1): ?>', $content);
        $content = preg_replace('/@endforeach/', '<?php endforeach; ?>', $content);
        $content = preg_replace('/@for\s*\((.+?)\)/', '<?php for ($1): ?>', $content);
        $content = preg_replace('/@endfor/', '<?php endfor; ?>', $content);
        $content = preg_replace('/@while\s*\((.+?)\)/', '<?php while ($1): ?>', $content);
        $content = preg_replace('/@endwhile/', '<?php endwhile; ?>', $content);

        // Include
        $content = preg_replace_callback('/@include\s*\([\'"](.+?)[\'"]\s*(?:,\s*(\[.*?\]|\$.*?))?\s*\)/', function($m) {
            $view = str_replace('.', '/', $m[1]);
            $data = $m[2] ?? '[]';
            return '<?php $__env->startInclude(' . var_export($view, true) . ', ' . $data . '); ?>';
        }, $content);

        // Section / Layout
        $content = preg_replace('/@section\([\'"](.+?)[\'"]\s*(?:,\s*(.+?))?\)/', '<?php $__env->startSection(\'$1\', $2 ?? null); ?>', $content);
        $content = preg_replace('/@show/', '<?php $__env->endSection(); ?><?php echo $__env->yieldContent(); ?>', $content);
        $content = preg_replace('/@yield\([\'"](.+?)[\'"]\s*(?:,\s*(.+?))?\)/', '<?= $__env->yieldContent(\'$1\', $2 ?? \'\') ?>', $content);
        $content = preg_replace('/@endsection/', '<?php $__env->stopSection(); ?>', $content);
        $content = preg_replace('/@overwrite/', '<?php $__env->stopSection(true); ?>', $content);
        $content = preg_replace('/@append/', '<?php $__env->appendSection(); ?>', $content);
        $content = preg_replace('/@extends\([\'"](.+?)[\'"]\s*\)/', '<?php $__env->setParent(\'$1\'); ?>', $content);

        // Stacks
        $content = preg_replace('/@push\([\'"](.+?)[\'"]\s*\)/', '<?php $__env->startPush(\'$1\'); ?>', $content);
        $content = preg_replace('/@endpush/', '<?php $__env->stopPush(); ?>', $content);
        $content = preg_replace('/@prepend\([\'"](.+?)[\'"]\s*\)/', '<?php $__env->startPrepend(\'$1\'); ?>', $content);
        $content = preg_replace('/@endprepend/', '<?php $__env->stopPrepend(); ?>', $content);
        $content = preg_replace('/@stack\([\'"](.+?)[\'"]\s*\)/', '<?= $__env->yieldPushContent(\'$1\') ?>', $content);

        // Components
        $content = preg_replace_callback('/@component\([\'"](.+?)[\'"]\s*(?:,\s*(\[.*?\]))?\s*\)/', function($m) {
            $name = $m[1]; $data = $m[2] ?? '[]';
            return '<?php $__env->startComponent(' . var_export($name, true) . ', ' . $data . '); ?>';
        }, $content);
        $content = preg_replace('/@endcomponent/', '<?php $__env->endComponent(); ?>', $content);
        $content = preg_replace('/@slot\([\'"](.+?)[\'"]\s*\)/', '<?php $__env->slot(\'$1\'); ?>', $content);
        $content = preg_replace('/@endslot/', '<?php $__env->endSlot(); ?>', $content);

        // Auth
        $content = preg_replace('/@auth/', '<?php if (auth()->check()): ?>', $content);
        $content = preg_replace('/@endauth/', '<?php endif; ?>', $content);
        $content = preg_replace('/@guest/', '<?php if (auth()->guest()): ?>', $content);
        $content = preg_replace('/@endguest/', '<?php endif; ?>', $content);

        // Misc
        $content = preg_replace('/@isset\s*\((.+?)\)/', '<?php if (isset($1)): ?>', $content);
        $content = preg_replace('/@endisset/', '<?php endif; ?>', $content);
        $content = preg_replace('/@empty\s*\((.+?)\)/', '<?php if (empty($1)): ?>', $content);
        $content = preg_replace('/@endempty/', '<?php endif; ?>', $content);
        $content = preg_replace('/@csrf/', '<?php echo csrf_field(); ?>', $content);
        $content = preg_replace('/@method\([\'"](.+?)[\'"]\s*\)/', '<?php echo method_field(\'$1\'); ?>', $content);
        $content = preg_replace('/@json\((.+?)\)/', '<?php echo json_encode($1); ?>', $content);

        // Production
        $content = preg_replace('/@production/', '<?php if (app()->environment(\'production\')): ?>', $content);
        $content = preg_replace('/@endproduction/', '<?php endif; ?>', $content);
        $content = preg_replace('/@env\([\'"](.+?)[\'"]\s*\)/', '<?php if (app()->environment(\'$1\')): ?>', $content);
        $content = preg_replace('/@endenv/', '<?php endif; ?>', $content);

        // Debug
        $content = preg_replace('/@dump\((.+?)\)/', '<?php dump($1); ?>', $content);
        $content = preg_replace('/@dd\((.+?)\)/', '<?php dd($1); ?>', $content);

        return $content;
    }

    // ─── Blade View Engine ──────────────────────────────────────────────

    private function getCachedPath(string $view, string $project): string {
        $hash = md5($view);
        $cacheDir = "$project/storage/framework/views";
        @mkdir($cacheDir, 0777, true);
        return "$cacheDir/$hash.php";
    }

    private function delTree(string $dir): void {
        if (!is_dir($dir)) return;
        foreach (scandir($dir) as $f) {
            if ($f === '.' || $f === '..') continue;
            $p = "$dir/$f";
            is_dir($p) ? $this->delTree($p) : unlink($p);
        }
        rmdir($dir);
    }

    private function handleBlade(string $view, string $dataJson, string $project): void {
        $data = json_decode($dataJson, true) ?? [];
        $viewsPath = "$project/resources/views";
        $viewFile = "$viewsPath/" . str_replace('.', '/', $view) . ".blade.php";
        if (!file_exists($viewFile)) { $this->output('', "Blade view not found: $view", 1, ''); return; }

        if (class_exists('\Illuminate\Support\Facades\View')) {
            try {
                $html = \Illuminate\Support\Facades\View::make($view, $data)->render();
                $this->output($html, '', 0, $html); return;
            } catch (\Throwable $e) {
                $this->output('', 'Laravel error: ' . $e->getMessage(), 1, ''); return;
            }
        }

        $cachedPath = $this->getCachedPath($view, $project);
        $sourceMtime = filemtime($viewFile);

        if (!file_exists($cachedPath) || filemtime($cachedPath) < $sourceMtime) {
            $source = file_get_contents($viewFile);
            $compiled = $this->compileBlade($source);
            file_put_contents($cachedPath, $compiled);
        }

        $this->sections = [];
        $this->sectionStack = [];
        $this->pushStack = [];
        $this->parentView = '';
        $this->viewData = $data;

        $viewFactory = $this;

        ob_start();
        try {
            extract($data);
            require $cachedPath;
            $html = ob_get_clean() ?: '';

            if (!empty($this->parentView)) {
                $parentFile = "$viewsPath/" . str_replace('.', '/', $this->parentView) . ".blade.php";
                if (file_exists($parentFile)) {
                    $parentCache = $this->getCachedPath($this->parentView, $project);
                    if (!file_exists($parentCache) || filemtime($parentCache) < filemtime($parentFile)) {
                        $parentSource = file_get_contents($parentFile);
                        file_put_contents($parentCache, $this->compileBlade($parentSource));
                    }
                    ob_start();
                    extract($data);
                    require $parentCache;
                    $html = ob_get_clean() ?: '';
                }
            }

            $this->output($html, '', 0, $html);
        } catch (\Throwable $e) {
            ob_end_clean();
            $this->output('', "Blade error: " . $e->getMessage(), 1, '');
        }
    }

    // ─── View Engine Methods (used by compiled Blade) ──────────────────

    public function setParent(string $view): void { $this->parentView = $view; }

    public function startSection(string $name, $value = null): void {
        if ($value !== null) { $this->sections[$name] = $value; return; }
        $this->sectionStack[] = $name;
        ob_start();
    }

    public function stopSection(bool $overwrite = false): void {
        $name = array_pop($this->sectionStack);
        if ($name === null) return;
        $content = ob_get_clean() ?: '';
        if ($overwrite || !isset($this->sections[$name])) $this->sections[$name] = $content;
        else $this->sections[$name] .= $content;
    }

    public function appendSection(): void { $this->stopSection(false); }

    public function yieldContent(string $name, string $default = ''): string {
        return $this->sections[$name] ?? $default;
    }

    public function startInclude(string $view, array $data = []): void {}

    public function startPush(string $stack): void {
        $this->sectionStack[] = '__push_' . $stack;
        ob_start();
    }

    public function stopPush(): void {
        $name = array_pop($this->sectionStack);
        if ($name === null) return;
        $stack = substr($name, 7);
        $content = ob_get_clean() ?: '';
        if (!isset($this->pushStack[$stack])) $this->pushStack[$stack] = [];
        $this->pushStack[$stack][] = $content;
    }

    public function startPrepend(string $stack): void {
        $this->sectionStack[] = '__prepend_' . $stack;
        ob_start();
    }

    public function stopPrepend(): void {
        $name = array_pop($this->sectionStack);
        if ($name === null) return;
        $stack = substr($name, 10);
        $content = ob_get_clean() ?: '';
        if (!isset($this->pushStack[$stack])) $this->pushStack[$stack] = [];
        array_unshift($this->pushStack[$stack], $content);
    }

    public function yieldPushContent(string $stack): string {
        return implode("\n", $this->pushStack[$stack] ?? []);
    }

    public function startComponent(string $name, array $data = []): void { ob_start(); }

    public function endComponent(): void { ob_end_clean(); }

    public function slot(string $name): void {
        $this->sectionStack[] = '__slot_' . $name;
        ob_start();
    }

    public function endSlot(): void {
        $name = array_pop($this->sectionStack);
        if ($name === null) return;
        ob_end_clean();
    }

    private function handleBladeClear(string $project): void {
        $cacheDir = "$project/storage/framework/views";
        if (!is_dir($cacheDir)) { $this->output('', '', 0, 'Cache directory does not exist'); return; }
        $files = glob("$cacheDir/*.php");
        $count = 0;
        foreach ($files as $f) { if (is_file($f)) { unlink($f); $count++; } }
        $this->output("Cleared $count cached views", '', 0, "ok");
    }

    // ─── Format ─────────────────────────────────────────────────────────

    private function handleFormat(string $code): void {
        if (empty($code)) { $this->output('', 'No code provided', 1, ''); return; }
        $tmpDir = sys_get_temp_dir() . '/klyron_phpfmt_' . bin2hex(random_bytes(4));
        mkdir($tmpDir, 0700, true);
        $tmpFile = "$tmpDir/input.php";
        file_put_contents($tmpFile, "<?php\n" . $code);

        $this->runCommand("php-cs-fixer fix " . escapeshellarg($tmpFile) . " --rules=@PSR12 --allow-risky=no 2>&1 || true");

        $formatted = file_get_contents($tmpFile);
        if (str_starts_with($formatted, "<?php\n")) {
            $formatted = substr($formatted, 6);
        }
        $this->delTree($tmpDir);
        $this->output($formatted, '', 0, 'ok');
    }

    // ─── Lint ───────────────────────────────────────────────────────────

    private function handleLint(string $code): void {
        if (empty($code)) { $this->output('', 'No code provided', 1, ''); return; }
        $tmpFile = tempnam(sys_get_temp_dir(), 'php_lint_') . '.php';
        file_put_contents($tmpFile, "<?php\n" . $code);
        $result = $this->runCommand('php -l ' . escapeshellarg($tmpFile) . ' 2>&1');
        unlink($tmpFile);
        $diags = [];
        if ($result['exit_code'] !== 0) {
            $diags[] = ['file' => '<eval>', 'line' => 0, 'col' => 0, 'message' => $result['stderr'], 'severity' => 'error'];
        }
        $this->output($result['stdout'], $result['stderr'], $result['exit_code'],
            $result['exit_code'] === 0 ? 'No syntax errors' : 'Syntax error', $diags);
    }

    // ─── Test ───────────────────────────────────────────────────────────

    private function handleTest(string $code, array $files): void {
        $tmpDir = sys_get_temp_dir() . '/klyron_phptest_' . bin2hex(random_bytes(4));
        mkdir($tmpDir, 0700, true);

        if (!empty($files)) {
            foreach ($files as $f) {
                file_put_contents("$tmpDir/" . basename($f['name']), $f['content']);
            }
        }
        if (!empty($code)) {
            file_put_contents("$tmpDir/Test.php", $code);
        }

        $result = $this->runCommand("phpunit $tmpDir 2>&1");
        $this->delTree($tmpDir);
        $this->output($result['stdout'], $result['stderr'], $result['exit_code'], $result['stdout']);
    }
}

$engine = new PhpEngine();
if (function_exists('ob_implicit_flush')) ob_implicit_flush(true);

while (($line = fgets(STDIN)) !== false) {
    $line = trim($line);
    if ($line === '') continue;
    $engine->handle($line);
}

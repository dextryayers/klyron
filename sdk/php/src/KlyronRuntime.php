<?php

namespace Klyron;

class KlyronRuntime
{
    private array $options;

    public function __construct(array $options = [])
    {
        $this->options = $options;
    }

    public function getFs(): KlyronFS
    {
        return new KlyronFS();
    }

    public function getHttp(): KlyronHTTP
    {
        return new KlyronHTTP();
    }

    public function getEnv(): KlyronEnv
    {
        return new KlyronEnv();
    }

    public function version(): string
    {
        return '0.1.0';
    }

    public function eval(string $code, string $lang = 'php'): string
    {
        switch ($lang) {
            case 'php':
                ob_start();
                eval($code);
                return ob_get_clean() ?: '';
            case 'js':
            case 'ts':
                $descriptors = [
                    ['pipe', 'r'],
                    ['pipe', 'w'],
                    ['pipe', 'w'],
                ];
                $proc = proc_open('node -e "' . addslashes($code) . '"', $descriptors, $pipes);
                if (is_resource($proc)) {
                    fclose($pipes[0]);
                    $output = stream_get_contents($pipes[1]);
                    fclose($pipes[1]);
                    fclose($pipes[2]);
                    proc_close($proc);
                    return $output ?: '';
                }
                throw new \RuntimeException("Failed to execute JS code");
            case 'py':
                $output = shell_exec("python3 -c " . escapesarg($code));
                return $output ?: '';
            default:
                $output = shell_exec($code);
                return $output ?: '';
        }
    }
}

class KlyronFS
{
    public function read(string $path): string
    {
        $content = @file_get_contents($path);
        if ($content === false) {
            throw new \RuntimeException("Cannot read file: $path");
        }
        return $content;
    }

    public function write(string $path, string $content): void
    {
        $dir = dirname($path);
        if (!is_dir($dir)) {
            mkdir($dir, 0755, true);
        }
        $result = @file_put_contents($path, $content);
        if ($result === false) {
            throw new \RuntimeException("Cannot write file: $path");
        }
    }

    public function exists(string $path): bool
    {
        return file_exists($path);
    }

    public function list(string $dir = '.'): array
    {
        $items = @scandir($dir);
        if ($items === false) {
            throw new \RuntimeException("Cannot list directory: $dir");
        }
        return array_values(array_diff($items, ['.', '..']));
    }

    public function mkdir(string $dir): void
    {
        if (!is_dir($dir)) {
            mkdir($dir, 0755, true);
        }
    }

    public function remove(string $path): void
    {
        if (is_dir($path)) {
            $this->rmdirRecursive($path);
        } else {
            @unlink($path);
        }
    }

    private function rmdirRecursive(string $dir): void
    {
        $items = @scandir($dir);
        if ($items === false) return;
        foreach ($items as $item) {
            if ($item === '.' || $item === '..') continue;
            $path = $dir . DIRECTORY_SEPARATOR . $item;
            if (is_dir($path)) {
                $this->rmdirRecursive($path);
            } else {
                @unlink($path);
            }
        }
        @rmdir($dir);
    }
}

class KlyronHTTP
{
    public function get(string $url, array $headers = []): array
    {
        return $this->request('GET', $url, null, $headers);
    }

    public function post(string $url, $body = null, array $headers = []): array
    {
        return $this->request('POST', $url, $body, $headers);
    }

    public function put(string $url, $body = null, array $headers = []): array
    {
        return $this->request('PUT', $url, $body, $headers);
    }

    public function del(string $url, array $headers = []): array
    {
        return $this->request('DELETE', $url, null, $headers);
    }

    public function request(string $method, string $url, $body = null, array $headers = []): array
    {
        $ch = curl_init();
        curl_setopt($ch, CURLOPT_URL, $url);
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
        curl_setopt($ch, CURLOPT_CUSTOMREQUEST, $method);
        curl_setopt($ch, CURLOPT_HTTPHEADER, $headers);

        if ($body !== null) {
            curl_setopt($ch, CURLOPT_POSTFIELDS, $body);
        }

        $response = curl_exec($ch);
        $httpCode = curl_getinfo($ch, CURLINFO_HTTP_CODE);
        curl_close($ch);

        return [
            'status' => $httpCode,
            'body' => $response ?: '',
        ];
    }
}

class KlyronEnv
{
    public function get(string $key): ?string
    {
        return getenv($key) ?: null;
    }

    public function set(string $key, string $value): void
    {
        putenv("$key=$value");
        $_ENV[$key] = $value;
    }

    public function getAll(): array
    {
        return $_ENV;
    }

    public function has(string $key): bool
    {
        return getenv($key) !== false;
    }
}

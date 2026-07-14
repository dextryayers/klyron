const std = @import("std");
const json = std.json;
const fs = std.fs;
const process = std.process;

const Input = struct {
    action: []const u8,
    code: ?[]const u8 = null,
    args: ?[]const u8 = null,
    filename: ?[]const u8 = null,
};

const Output = struct {
    stdout: []const u8,
    stderr: []const u8,
    exit_code: i32,
    result: []const u8,
};

fn writeOutput(out: Output, writer: anytype) !void {
    const string = try json.stringifyAlloc(std.heap.page_allocator, out, .{});
    defer std.heap.page_allocator.free(string);
    try writer.print("{s}\n", .{string});
}

fn execCode(code: []const u8, writer: anytype) !void {
    var dir_buf: [64]u8 = undefined;
    const template = "/tmp/klyron-zig-XXXXXX";
    @memcpy(dir_buf[0..template.len], template);
    const real_path = std.os.mkdtemp(dir_buf[0..template.len]) orelse return error.TempDirFailed;
    const path_len = std.mem.len(real_path);

    defer fs.cwd().deleteTree(dir_buf[0..path_len]) catch {};

    const dir = try fs.cwd().openDir(dir_buf[0..path_len], .{});
    defer dir.close();

    const src_path = try dir.realpathAlloc(std.heap.page_allocator, "main.zig");
    defer std.heap.page_allocator.free(src_path);

    const bin_path = try dir.realpathAlloc(std.heap.page_allocator, "prog");
    defer std.heap.page_allocator.free(bin_path);

    try dir.writeFile("main.zig", code);

    const compile_result = try process.Child.run(.{
        .allocator = std.heap.page_allocator,
        .argv = &[_][]const u8{ "zig", "build-exe", src_path, "--name", "prog", "-O", "ReleaseFast" },
    });
    defer std.heap.page_allocator.free(compile_result.stdout);
    defer std.heap.page_allocator.free(compile_result.stderr);

    if (compile_result.term != .Exited or compile_result.term.Exited != 0) {
        try writeOutput(Output{
            .stdout = "",
            .stderr = compile_result.stderr,
            .exit_code = 1,
            .result = "Compilation failed",
        }, writer);
        return;
    }

    const run_result = try process.Child.run(.{
        .allocator = std.heap.page_allocator,
        .argv = &[_][]const u8{dir_buf[0..path_len]},
    });
    defer std.heap.page_allocator.free(run_result.stdout);
    defer std.heap.page_allocator.free(run_result.stderr);

    try writeOutput(Output{
        .stdout = run_result.stdout,
        .stderr = run_result.stderr,
        .exit_code = if (run_result.term == .Exited) @as(i32, @intCast(run_result.term.Exited)) else -1,
        .result = run_result.stdout,
    }, writer);
}

pub fn main() !void {
    const stdin = std.io.getStdIn().reader();
    const stdout = std.io.getStdOut().writer();
    var buf: [65536]u8 = undefined;

    while (try stdin.readUntilDelimiterOrEof(&buf, '\n')) |line| {
        const trimmed = std.mem.trim(u8, line, " \n\r");
        if (trimmed.len == 0) continue;

        var parser = json.Parser.init(std.heap.page_allocator, false);
        defer parser.deinit();

        const input = parser.parse(trimmed) catch {
            try writeOutput(Output{ .stdout = "", .stderr = "Invalid JSON", .exit_code = 1, .result = "" }, stdout);
            continue;
        };

        const action = input.object.get("action") orelse continue;
        const action_str = action.string orelse "";

        if (std.mem.eql(u8, action_str, "exec") or std.mem.eql(u8, action_str, "run")) {
            const code = input.object.get("code") orelse continue;
            try execCode(code.string orelse "", stdout);
        } else if (std.mem.eql(u8, action_str, "ping") or action_str.len == 0) {
            try writeOutput(Output{ .stdout = "pong", .stderr = "", .exit_code = 0, .result = "ok" }, stdout);
        } else {
            try writeOutput(Output{ .stdout = "", .stderr = "Unknown action", .exit_code = 1, .result = "" }, stdout);
        }
    }
}

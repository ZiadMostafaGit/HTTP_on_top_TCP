const std = @import("std");
const net = std.net;
const fs = std.fs;
const mem = std.mem;
const json = std.json;
const heap = std.heap;
const c = @cImport({
    @cInclude("sqlite3.h");
});

// Define our custom error set for SQLite operations
const InsertError = error{
    DatabaseOpenFailed,
    PrepareFailed,
    BindFailed,
    QueryStepFailed,
    InsertFailed,
    EmailExists,
};

// A simple user_info struct
const UserInfo = struct {
    name: []const u8,
    email: []const u8,
};

// Fixed SQLite insertion function
fn insertUserInfo(user: UserInfo) !void {
    var db_ptr: ?*c.sqlite3 = null;
    if (c.sqlite3_open("user_info.db", &db_ptr) != c.SQLITE_OK) {
        return InsertError.DatabaseOpenFailed;
    }

    std.debug.print("untile now everthing is working\n", .{});

    const db = db_ptr.?;
    defer _ = c.sqlite3_close(db);

    var stmt_ptr: ?*c.sqlite3_stmt = null;
    const countSql = "SELECT COUNT(*) FROM users WHERE email = ?1";
    if (c.sqlite3_prepare_v2(db, countSql, -1, &stmt_ptr, null) != c.SQLITE_OK) {
        return InsertError.PrepareFailed;
    }
    const stmt = stmt_ptr.?;
    defer _ = c.sqlite3_finalize(stmt);

    // Bind the email parameter
    const size: i32 = @intCast(user.email.len);
    if (c.sqlite3_bind_text(stmt, 1, user.email.ptr, size, c.SQLITE_TRANSIENT) != c.SQLITE_OK) {
        return InsertError.BindFailed;
    }
    if (c.sqlite3_step(stmt) != c.SQLITE_ROW) {
        return InsertError.QueryStepFailed;
    }
    const count = c.sqlite3_column_int(stmt, 0);

    if (count == 0) {
        // No user with this email, so insert the new user
        var insert_stmt_ptr: ?*c.sqlite3_stmt = null;
        const insertSql = "INSERT INTO users (name, email) VALUES (?1, ?2)";
        if (c.sqlite3_prepare_v2(db, insertSql, -1, &insert_stmt_ptr, null) != c.SQLITE_OK) {
            return InsertError.PrepareFailed;
        }
        const insert_stmt = insert_stmt_ptr.?;
        defer _ = c.sqlite3_finalize(insert_stmt);

        const size2: i32 = @intCast(user.email.len);
        if (c.sqlite3_bind_text(insert_stmt, 1, user.name.ptr, size2, c.SQLITE_TRANSIENT) != c.SQLITE_OK) {
            return InsertError.BindFailed;
        }
        if (c.sqlite3_bind_text(insert_stmt, 2, user.email.ptr, size2, c.SQLITE_TRANSIENT) != c.SQLITE_OK) {
            return InsertError.BindFailed;
        }
        if (c.sqlite3_step(insert_stmt) != c.SQLITE_DONE) {
            return InsertError.InsertFailed;
        }
    } else {
        return InsertError.EmailExists;
    }
}

// Fixed URL to file mapping function
fn mapUrlToFile(allocator: mem.Allocator, base_dir: []const u8, url_path: []const u8) !?[]const u8 {
    var safe_path: []const u8 = url_path;
    if (mem.startsWith(u8, safe_path, "/")) {
        safe_path = safe_path[1..];
    }
    if (safe_path.len == 0) {
        safe_path = "index.html";
    }
    if (safe_path[safe_path.len - 1] == '/') {
        safe_path = "index.html";
    }
    return try std.fmt.allocPrint(allocator, "{s}{s}", .{ base_dir, safe_path });
}

// Content type helper remains mostly the same
fn getContentType(path: []const u8) []const u8 {
    if (mem.endsWith(u8, path, ".html")) return "text/html";
    if (mem.endsWith(u8, path, ".css")) return "text/css";
    if (mem.endsWith(u8, path, ".js")) return "application/javascript";
    if (mem.endsWith(u8, path, ".png")) return "image/png";
    if (mem.endsWith(u8, path, ".jpg") or mem.endsWith(u8, path, ".jpeg")) return "image/jpeg";
    return "text/plain";
}

fn handlePostForStoreUserData(allocator: mem.Allocator, stream: *net.Stream, body: []const u8) !void {
    var tree = try json.parseFromSlice(json.Value, allocator, body, .{});
    defer tree.deinit();

    const root = tree.value;
    const name = root.object.get("name") orelse return error.MissingField;
    const email = root.object.get("email") orelse return error.MissingField;

    // if (name.* != .string or email.* != .string) {
    //     return error.InvalidJSON;
    // }
    //
    const user = UserInfo{
        .name = name.string,
        .email = email.string,
    };

    std.debug.print("name is {s} and email is {s}\n", .{ user.name, user.email });

    const responseMessage = if (insertUserInfo(user)) |_|
        "Your data is saved successfully"
    else |err| if (err == InsertError.EmailExists)
        "The email is already there try new email"
    else
        "Error saving data";

    const jsonResponse = try std.fmt.allocPrint(allocator, "{{\"message\": \"{s}\"}}", .{responseMessage});
    defer allocator.free(jsonResponse);

    const header = try std.fmt.allocPrint(allocator, "HTTP/1.1 200 OK\r\nContent-Length: {d}\r\nContent-Type: application/json\r\n\r\n", .{jsonResponse.len});
    defer allocator.free(header);

    try stream.writer().writeAll(header);
    try stream.writer().writeAll(jsonResponse);
}
// Fixed GET handler
fn handleGet(allocator: mem.Allocator, stream: *net.Stream, url: []const u8) !void {
    const base_path = "/home/ziad/git/HTTP_on_top_TCP/zig/";
    const file_path = try mapUrlToFile(allocator, base_path, url) orelse {
        const notFound = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
        try stream.writer().writeAll(notFound);
        return;
    };
    defer allocator.free(file_path);

    const file = try fs.cwd().openFile(file_path, .{});
    defer file.close();

    const file_size = try file.getEndPos();
    const file_data = try file.readToEndAlloc(allocator, file_size);
    defer allocator.free(file_data);

    const content_type = getContentType(file_path);
    const header = try std.fmt.allocPrint(allocator, "HTTP/1.1 200 OK\r\nContent-Type: {s}\r\nContent-Length: {d}\r\n\r\n", .{ content_type, file_data.len });
    defer allocator.free(header);

    try stream.writer().writeAll(header);
    try stream.writer().writeAll(file_data);
}

// Fixed request handler
fn handleRequest(allocator: mem.Allocator, stream: *net.Stream) !void {
    var buffer: [4096]u8 = undefined;
    const n = try stream.read(&buffer);
    const request_str = mem.trim(u8, buffer[0..n], "\r\n");

    var lines = mem.split(u8, request_str, "\r\n");
    const first_line = lines.first();
    var parts = mem.split(u8, first_line, " ");

    const method = parts.next() orelse return error.InvalidRequest;
    const url = parts.next() orelse return error.InvalidRequest;
    const version = parts.next() orelse return error.InvalidRequest;

    if (!mem.startsWith(u8, version, "HTTP/")) {
        std.debug.print("Not an HTTP request; closing connection.\n", .{});
        return;
    }

    // Find the body after empty line for POST requests
    var body: []const u8 = "";
    if (mem.eql(u8, method, "POST")) {
        var found_empty_line = false;
        while (lines.next()) |line| {
            if (line.len == 0) {
                found_empty_line = true;
                if (lines.next()) |body_line| {
                    body = body_line;
                }
                break;
            }
        }
    }

    if (mem.eql(u8, method, "GET")) {
        try handleGet(allocator, stream, url);
    } else if (mem.eql(u8, method, "POST") and mem.eql(u8, url, "/send_form")) {
        try handlePostForStoreUserData(allocator, stream, body);
    }
}

pub fn main() !void {
    var gpa_alloc = std.heap.GeneralPurposeAllocator(.{}){};
    defer std.debug.assert(gpa_alloc.deinit() == .ok);
    const gpa = gpa_alloc.allocator();

    const addr = std.net.Address.initIp4(.{ 0, 0, 0, 0 }, 5000);
    var server = try addr.listen(.{});

    std.log.info("Server listening on port 5000", .{});

    while (true) {
        var client = try server.accept();
        defer client.stream.close();

        // Handle the HTTP request
        handleRequest(gpa, &client.stream) catch |err| {
            std.log.err("Failed to handle request: {}", .{err});
        };
    }
}

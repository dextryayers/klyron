import json
from http.server import HTTPServer, BaseHTTPRequestHandler

USERS = [
    {"id": 1, "name": "Alice", "email": "alice@example.com"},
    {"id": 2, "name": "Bob", "email": "bob@example.com"},
]


class UserHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/users":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(USERS).encode())
        elif self.path.startswith("/users/"):
            uid = int(self.path.split("/")[-1])
            user = next((u for u in USERS if u["id"] == uid), None)
            if user:
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps(user).encode())
            else:
                self.send_response(404)
                self.end_headers()
                self.wfile.write(b'{"error": "not found"}')
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        print(f"[Users Service] {args[0]} {args[1]} {args[2]}")


if __name__ == "__main__":
    server = HTTPServer(("0.0.0.0", 4001), UserHandler)
    print("Users service running on port 4001")
    server.serve_forever()

import http.server
import socketserver
import os

PORT = 8001
DIRECTORY = os.path.join(os.path.dirname(os.path.abspath(__file__)), "output")


class MyHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIRECTORY, **kwargs)

    def do_GET(self):
        if self.path == "/":
            self.path = "/index.html"
        return super().do_GET()


if __name__ == "__main__":
    with socketserver.TCPServer(("", PORT), MyHandler) as httpd:
        print(f"Serving at http://localhost:{PORT}")
        print(f"Serving from directory: {DIRECTORY}")
        httpd.serve_forever()

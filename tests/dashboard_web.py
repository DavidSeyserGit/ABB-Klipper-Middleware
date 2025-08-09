from flask import Flask, render_template
import requests
import os

API_BASE = "http://localhost:8080"  # Change to your machine's LAN IP if accessing from another PC

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
TEMPLATE_DIR = os.path.join(BASE_DIR)
LOG_FILE = "log/log.log"

app = Flask(__name__, template_folder=TEMPLATE_DIR)


def tail(file_path, lines=10):
    """Read the last N lines of a file efficiently."""
    try:
        with open(file_path, "rb") as f:
            f.seek(0, os.SEEK_END)
            buffer = bytearray()
            pointer = f.tell()
            while pointer >= 0 and lines > 0:
                f.seek(pointer)
                byte = f.read(1)
                if byte == b"\n":
                    lines -= 1
                    if lines == 0:
                        break
                buffer.extend(byte)
                pointer -= 1
            return buffer[::-1].decode("utf-8", errors="ignore").splitlines()
    except FileNotFoundError:
        return ["Log file not found."]
    except Exception as e:
        return [f"Error reading log file: {e}"]


@app.route("/")
def dashboard():
    # Get stats from API
    try:
        stats = requests.get(f"{API_BASE}/stats").json()
    except Exception as e:
        stats = {"data_sent": 0, "data_received": 0, "last_command": f"Error: {e}"}

    # Get clients from API
    try:
        clients = requests.get(f"{API_BASE}/clients").json()
    except Exception as e:
        clients = []

    # Read last 50 lines from log file
    logs = tail(LOG_FILE, lines=10)

    return render_template("dashboard.html", stats=stats, clients=clients, logs=logs)


if __name__ == "__main__":
    # Run on all network interfaces so it can be accessed on LAN
    app.run(host="0.0.0.0", port=5050, debug=True)
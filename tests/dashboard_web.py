from flask import Flask, render_template
import requests

API_BASE = "http://localhost:8080"  # Change to your machine's LAN IP if accessing from another PC

import os

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
TEMPLATE_DIR = os.path.join(BASE_DIR)

app = Flask(__name__, template_folder=TEMPLATE_DIR)


@app.route("/")
def dashboard():
    try:
        stats = requests.get(f"{API_BASE}/stats").json()
    except Exception as e:
        stats = {"data_sent": 0, "data_received": 0, "last_command": f"Error: {e}"}
    try:
        clients = requests.get(f"{API_BASE}/clients").json()
    except Exception as e:
        clients = []
    return render_template("dashboard.html", stats=stats, clients=clients)

if __name__ == "__main__":
    # Run on all network interfaces so it can be accessed on LAN
    app.run(host="0.0.0.0", port=5050, debug=True)

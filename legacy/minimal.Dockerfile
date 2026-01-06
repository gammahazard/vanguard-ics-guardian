# ============================================================
# Legacy Minimal Sensor Driver
# ============================================================
# This is the "villain" in our security comparison.
# A typical Python-based sensor driver with minimal dependencies.
#
# Expected size: ~200 MB
# Expected cold start: ~3 seconds
#
# To build and measure:
#   docker build -f minimal.Dockerfile -t legacy-minimal .
#   docker inspect legacy-minimal --format='{{.Size}}'
# ============================================================

FROM python:3.9-slim

WORKDIR /app

# install common sensor communication libraries
RUN pip install --no-cache-dir \
    pyserial==3.5 \
    pymodbus==3.0.0

# copy the sensor driver script
COPY sensor_minimal.py .

# run the driver
CMD ["python", "sensor_minimal.py"]

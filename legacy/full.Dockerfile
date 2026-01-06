# ============================================================
# Legacy Full Processing Suite
# ============================================================
# A Python-based sensor driver with data processing capabilities.
# Includes pandas, numpy for typical industrial data analytics.
#
# Expected size: ~800 MB
# Expected cold start: ~8 seconds
#
# To build and measure:
#   docker build -f full.Dockerfile -t legacy-full .
#   docker inspect legacy-full --format='{{.Size}}'
# ============================================================

FROM python:3.9-slim

WORKDIR /app

# install data processing libraries - this is where size explodes
RUN pip install --no-cache-dir \
    pandas==2.0.0 \
    numpy==1.24.0 \
    scipy==1.10.0 \
    pyserial==3.5 \
    pymodbus==3.0.0 \
    requests==2.31.0

# copy the processing script
COPY sensor_full.py .

CMD ["python", "sensor_full.py"]

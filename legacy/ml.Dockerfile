# ============================================================
# Legacy ML Inference Engine
# ============================================================
# A Python-based ML inference container for predictive maintenance.
# Includes TensorFlow/ONNX - the heavyweight champion of bloat.
#
# Expected size: ~2 GB
# Expected cold start: ~15 seconds
#
# To build and measure:
#   docker build -f ml.Dockerfile -t legacy-ml .
#   docker inspect legacy-ml --format='{{.Size}}'
# ============================================================

FROM python:3.9-slim

WORKDIR /app

# install ML libraries - this is where things get REALLY big
RUN pip install --no-cache-dir \
    tensorflow-cpu==2.13.0 \
    onnxruntime==1.15.0 \
    pandas==2.0.0 \
    numpy==1.24.0 \
    scikit-learn==1.3.0 \
    pymodbus==3.0.0

# copy the inference script
COPY sensor_ml.py .

CMD ["python", "sensor_ml.py"]

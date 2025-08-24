FROM ubuntu:noble AS base

# Install dependencies
RUN apt-get update && apt-get install -y \
    python3-pip \
    python3-venv \
    curl \
    && rm -rf /var/lib/apt/lists/*

FROM base AS builder

USER ubuntu:ubuntu

RUN python3 -m venv /home/ubuntu/.venv
ENV PATH="/home/ubuntu/.venv/bin:$PATH"

COPY --chown=ubuntu:ubuntu requirements.txt /app/requirements.txt

WORKDIR /app

RUN pip3 install --no-cache-dir -r /app/requirements.txt

USER 0

# Copy the source code
WORKDIR /app

COPY run_server.py run_server.py

RUN chown -R ubuntu:ubuntu /app/run_server.py

FROM builder AS runtime

EXPOSE 28080

WORKDIR /app

USER 1000:1000

VOLUME [ "/data" ]

# Run the server
CMD ["python3", "run_server.py", "--host", "0.0.0.0", "--port", "28080", "--database_url", "sqlite:////data/vizier.db"]
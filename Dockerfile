FROM --platform=linux/amd64 ubuntu:jammy AS base
ARG USERNAME=ubuntu
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    git \
    libprotobuf-dev \
    protobuf-compiler \
    python3-pip \
    python3-venv \
    python3-grpcio \
    curl \
    && rm -rf /var/lib/apt/lists/*

FROM base AS builder

WORKDIR /app

USER $USER_UID:$USER_GID

RUN python3 -m venv /home/$USERNAME/.venv
ENV PATH="/home/$USERNAME/.venv/bin:$PATH"

COPY requirements.txt requirements.txt

RUN pip3 install --no-cache-dir -r requirements.txt

USER 0

# Copy the source code
WORKDIR /app

COPY run_server.py run_server.py

RUN chown -R $USER_UID:$USER_GID /app/run_server.py

FROM builder AS runtime

EXPOSE 28080

WORKDIR /app

USER $USER_UID:$USER_GID

# Run the server
CMD ["python3", "run_server.py"]
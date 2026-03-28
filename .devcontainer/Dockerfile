FROM mcr.microsoft.com/devcontainers/rust:1-bullseye

# Install Rust 1.85 (Edition 2024)
RUN rustup toolchain install 1.85 --component rustfmt clippy \
    && rustup default 1.85

# Install Node.js (required for some Copilot CLI variants)
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

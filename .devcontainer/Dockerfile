FROM mcr.microsoft.com/devcontainers/base:ubuntu24.04

# Installiere USB-Tools und Build-Dependencies als root
RUN apt-get update && apt-get install -y \
    usbutils \
    pkg-config \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/*

# Füge vscode User zu dialout und plugdev Gruppen für USB-Zugriff hinzu
RUN usermod -a -G dialout,plugdev vscode

# Wechsle zu vscode User
USER vscode

# Installiere rustup und Rust als vscode User
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Füge cargo zum PATH hinzu
ENV PATH="/home/vscode/.cargo/bin:${PATH}"

# Füge Rust target für ESP32-C6 (RISC-V) hinzu
RUN rustup target add riscv32imac-unknown-none-elf

# Installiere ESP32 Tools
RUN cargo install espflash
RUN cargo install esp-generate
RUN cargo install esp-config --features=tui --locked
RUN cargo install probe-rs-tools --locked

# Installiere cargo-binutils für Binäranalyse
RUN rustup component add llvm-tools && \
    cargo install cargo-binutils

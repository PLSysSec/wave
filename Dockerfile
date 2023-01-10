FROM ubuntu:latest
# Full Docker build will take 15-20 minutes

# Misc dependencies
RUN apt-get update
RUN apt-get install -y curl git unzip build-essential pkg-config libssl-dev cmake ninja-build clang

# Java (for Prusti)
RUN apt-get install default-jre -y

# Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo install --force cbindgen

# add ssh private key so we can download private repos
# TODO: remove once repos are public
# TODO: once ssh key is stripped out, add prebuilt container to repo
ARG SSH_PRIVATE_KEY
RUN mkdir /root/.ssh/
RUN echo "${SSH_PRIVATE_KEY}" > /root/.ssh/id_rsa
RUN chmod 0600 /root/.ssh/id_rsa
RUN ssh-keyscan github.com > /root/.ssh/known_hosts

# WaVe
RUN git clone git@github.com:PLSysSec/wave.git
WORKDIR /wave
RUN make bootstrap
RUN make build


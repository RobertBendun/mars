FROM ubuntu:20.04

RUN apt update && apt install -y curl gcc make build-essential

# Install rustup
RUN set -eux; \
		curl --location --fail \
			"https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init" \
			--output rustup-init; \
		chmod +x rustup-init; \
		./rustup-init -y --no-modify-path --default-toolchain stable; \
		rm rustup-init;

# Add rustup to path, check that it works
ENV PATH=${PATH}:/root/.cargo/bin

# Enable cargo sparse algorithm
RUN mkdir -p "$HOME/.cargo"; \
	echo "[registries.crates-io]" > "$HOME/.cargo/config.toml" \
	echo 'protocol = "sparse"' >> "$HOME/.cargo/config.toml"

WORKDIR . /movierate
RUN cargo build

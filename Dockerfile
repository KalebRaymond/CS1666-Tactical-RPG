FROM rust:1.57
# Update crates.io index here (so it gets cached in the image)
RUN cargo install lazy_static || true
# Install dependencies
RUN apt-get update
RUN apt-get install \
	uuid-dev \
	libssl-dev \
	libsdl2-dev \
	libsdl2-gfx-dev \
	libsdl2-image-dev \
	libsdl2-mixer-dev \
	libsdl2-net-dev \
	libsdl2-ttf-dev \
	-qqy

# Copy source files to the Docker image
COPY ./ ./

# Compile & run the binary
RUN cargo build

CMD ./target/debug/castle_quest --server

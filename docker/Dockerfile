# Use a base Debian image
FROM dziedzicgrzegorz/rusty_beagle_silicon

# Set up the working directory
WORKDIR /usr/src/rust

# Copy your project into the Docker container
COPY .. .

# Build the Rust project for ARM architecture
RUN cargo build --target arm-unknown-linux-gnueabihf --release

RUN cargo test

#Copy the output
CMD ["sh", "-c", "cp -r /usr/src/rust/target/arm-unknown-linux-gnueabihf/release/rusty_beagle /output"]
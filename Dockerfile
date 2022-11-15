from rust

WORKDIR /app
RUN apt-get update && apt-get install -y imagemagick
COPY src/ src/
COPY Cargo.toml Cargo.toml
FROM rust:1.58.1

WORKDIR "/sulfur"
COPY . .

RUN cargo build --release

CMD ["cargo", "run"]

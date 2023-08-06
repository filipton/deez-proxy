FROM debian:bookworm-slim
WORKDIR /app

RUN echo "deb http://security.debian.org/debian-security bullseye-security main" > /etc/apt/sources.list
RUN apt update && apt install -y libssl1.1 ca-certificates
RUN apt clean

COPY ./target/release/deno-test /app/deno-test
COPY ./main.js /app/main.js

ENTRYPOINT ["/app/deno-test"]

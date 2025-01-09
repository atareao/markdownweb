###############################################################################
## Builder
###############################################################################
FROM rust:alpine3.21 AS builder

LABEL maintainer="Lorenzo Carbonell <a.k.a. atareao>"

RUN apk add --update --no-cache \
            autoconf \
            gcc \
            gdb \
            #git \
            #libdrm-dev \
            #libepoxy-dev \
            #make \
            #mesa-dev \
            #strace \
            libressl-dev \
            musl-dev && \
    rm -rf /var/cache/apk && \
    rm -rf /var/lib/app/lists

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src src

RUN cargo build --release && \
    cp /app/target/release/markdownweb /app/markdownweb

###############################################################################
## Final image
###############################################################################
FROM alpine:3.21

ENV USER=app \
    UID=1000

RUN apk add --update --no-cache \
            tzdata~=2024 \
            && \
    rm -rf /var/cache/apk && \
    rm -rf /var/lib/app/lists && \
    mkdir -p /app/db && \
    mkdir -p /app/rss

# Copy our build
COPY --from=builder /app/markdownweb /app/
COPY ./assets /app/assets/
COPY ./templates /app/templates/

# Create the user
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/${USER}" \
    --shell "/sbin/nologin" \
    --uid "${UID}" \
    "${USER}" && \
    chown -R app:app /app

WORKDIR /app
USER app

CMD ["/app/markdownweb"]

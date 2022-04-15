## Build Stage
# Pull base image and update
FROM ekidd/rust-musl-builder:stable AS builder

USER root

RUN update-ca-certificates

ENV TZ=America/New_York
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt update
RUN apt install -y nodejs npm git libpq-dev postgresql
RUN apt update
RUN npm install -g n
RUN n stable
RUN npm install -g npm

# Create app user
ARG USER=backend
ARG UID=10001

ENV USER=$USER
ENV UID=$UID

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

COPY . /app

RUN chown -R "${USER}":"${USER}" /app

# Move to repo
WORKDIR /app

# Build app
RUN cargo build --release --target x86_64-unknown-linux-musl

# Build frontend
WORKDIR /app/frontend

RUN npm i
RUN npm run build

####################################################################################################
## Final image
####################################################################################################
FROM alpine:latest

ARG USER=backend
ARG UID=10001

ENV USER=$USER
ENV UID=$UID

RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app

# Copy our build
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/vidmeste /app/vidmeste
COPY --from=builder /app/frontend/dist/ /app/static
COPY --from=builder /app/entrypoint.sh /app/entrypoint.sh

RUN chown -R "${USER}":"${USER}" /app

RUN chmod +x /app/entrypoint.sh
RUN apk add --no-cache gettext
RUN apk add --no-cache --upgrade bash
RUN rm -rf /var/cache/apk/*

USER $USER:$USER

# Expose web http port
EXPOSE 9999

ENTRYPOINT ["sh", "/app/entrypoint.sh"]
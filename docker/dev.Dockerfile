FROM rust:latest as compiler

WORKDIR /compiler

ARG     COMMIT_SHA
ARG     COMMIT_DATE
ARG     GIT_TAG
ENV     VERGEN_GIT_SHA=${COMMIT_SHA} VERGEN_GIT_COMMIT_TIMESTAMP=${COMMIT_DATE} VERGEN_GIT_SEMVER_LIGHTWEIGHT=${GIT_TAG}
ENV     RUSTFLAGS="-C target-feature=-crt-static"

COPY    . .
RUN     set -eux; \
        cargo build --release

FROM alpine:3.16

#RUN     #apk update --quiet \
#        && apk add -q --no-cache libgcc tini curl

COPY    --from=compiler /compiler/target/release/meilisearch /bin/meilisearch

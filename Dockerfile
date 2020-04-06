FROM alpine:latest
COPY target/release/ram /usr/bin/
ENTRYPOINT /usr/bin/ram

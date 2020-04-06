FROM ubuntu:latest
COPY target/release/ram /
WORKDIR /src
CMD [""]
ENTRYPOINT ["/ram"]

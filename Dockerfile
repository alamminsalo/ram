FROM ubuntu
COPY target/release/ram /
WORKDIR /src
CMD [""]
ENTRYPOINT ["/ram"]

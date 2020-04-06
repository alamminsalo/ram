FROM ubuntu:latest
COPY target/release/ram /
CMD /ram

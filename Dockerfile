FROM ubuntu:22.04
 
RUN apt-get update && apt-get install -y curl
RUN apt-get install build-essential -y
 
RUN mkdir -p /user/mysteriousbotbuilder/src
WORKDIR /user/mysteriousbotbuilder/src
 
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

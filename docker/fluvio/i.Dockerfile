FROM ubuntu:20.04

RUN apt-get update && \
    apt-get install -y curl unzip && \
    curl -fsS https://hub.infinyon.cloud/install/install.sh?ctx=dc | bash  && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

ENV PATH="$PATH:/root/.fluvio/bin"
ENV PATH="$PATH:/root/.fvm/bin"
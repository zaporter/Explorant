FROM ubuntu:14.04

RUN apt update
RUN apt install -y git
RUN apt install -y openjdk-7-jdk
RUN apt install -y ant
RUN apt install -y graphviz

RUN git clone --depth 1 https://github.com/ModelInference/synoptic
RUN cd synoptic; ant synoptic

# ENTRYPOINT cd synoptic; ./synoptic-jar.sh -h
ENTRYPOINT cd synoptic; ./synoptic-jar.sh --exportAsGML -o /shared/out -j /shared/test.log

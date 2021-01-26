FROM debian:stable-slim

RUN apt-get update && apt-get -y install ca-certificates libssl-dev \
 && rm -rf /var/lib/apt/lists/* \
 && groupadd --gid 1000 -r opahelper \
 && useradd --uid 1000 -r -g opahelper opahelper

COPY ../target/linux/release/opahelper /home/opahelper/opahelper

RUN chown -R opahelper:opahelper /home/opahelper \
 && chmod +x /home/opahelper/opahelper

WORKDIR /home/opahelper/

USER 1000
FROM opensuse/tumbleweed:latest
MAINTAINER william@blackhats.net.au

LABEL "Name"="lifx"

# RUN echo HTTP_PROXY="http://proxy-bne1.net.blackhats.net.au:3128" > /etc/sysconfig/proxy

COPY . /home/lifx/

WORKDIR /home/lifx/

RUN zypper install -y timezone cargo rust rust-std && \
    RUSTC_BOOTSTRAP=1 cargo build --release && \
    zypper rm -u -y cargo rust rust-std && \
    zypper clean

RUN cd /etc && \
    ln -sf ../usr/share/zoneinfo/Australia/Brisbane localtime

RUN useradd -m -r lifx
USER lifx

ENV RUST_BACKTRACE 1
CMD ["/home/lifx/target/release/lifx_ctl"]


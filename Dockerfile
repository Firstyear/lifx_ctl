FROM opensuse/tumbleweed:latest
MAINTAINER william@blackhats.net.au

LABEL "Name"="lifx"

# /usr/bin/docker run --restart always --name lifx registry.blackhats.net.au/lifx
COPY . /home/lifx/

WORKDIR /home/lifx/

RUN zypper install -y timezone cargo rust rust-std gcc && \
    RUSTC_BOOTSTRAP=1 cargo build --release && \
    zypper rm -u -y cargo rust rust-std gcc && \
    zypper clean

RUN cd /etc && \
    ln -sf ../usr/share/zoneinfo/Australia/Brisbane localtime

RUN useradd -m -r lifx
USER lifx

ENV RUST_BACKTRACE 1
CMD ["/home/lifx/target/release/lifx_ctl"]


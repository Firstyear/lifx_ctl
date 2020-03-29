FROM opensuse/tumbleweed:latest AS builder
MAINTAINER william@blackhats.net.au

RUN zypper mr -d repo-non-oss && \
    zypper mr -d repo-oss && \
    zypper mr -d repo-update && \
    zypper ar https://download.opensuse.org/update/tumbleweed/ repo-update-https && \
    zypper ar https://download.opensuse.org/tumbleweed/repo/oss/ repo-oss-https && \
    zypper ar https://download.opensuse.org/tumbleweed/repo/non-oss/ repo-non-oss-https && \
    zypper install -y cargo rust gcc sqlite3-devel libopenssl-devel


COPY . /home/lifx/
RUN mkdir /home/lifx/.cargo
WORKDIR /home/lifx/

RUN cp cargo_vendor.config .cargo/config && \
    cargo build --release

# == end builder setup, we now have static artifacts.
FROM opensuse/tumbleweed:latest
EXPOSE 8081
WORKDIR /
COPY --from=builder /home/lifx/target/release/lifx_ctl /bin/
COPY --from=builder /home/lifx/static /static
RUN zypper install -y sqlite3 openssl timezone

RUN cd /etc && \
    ln -sf ../usr/share/zoneinfo/Australia/Brisbane localtime

ENV RUST_BACKTRACE 1
CMD ["/bin/lifx_ctl"]

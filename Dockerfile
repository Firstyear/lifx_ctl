FROM opensuse/tumbleweed:latest AS ref_repo
RUN zypper mr -d repo-non-oss && \
    zypper mr -d repo-oss && \
    zypper mr -d repo-update && \
    zypper ar https://download.opensuse.org/update/tumbleweed/ repo-update-https && \
    zypper ar https://download.opensuse.org/tumbleweed/repo/oss/ repo-oss-https && \
    zypper ar https://download.opensuse.org/tumbleweed/repo/non-oss/ repo-non-oss-https && \
    zypper ref

# // setup the builder pkgs
FROM ref_repo AS build_base
RUN zypper install -y cargo rust gcc sqlite3-devel libopenssl-devel

# // setup the runner pkgs
FROM ref_repo AS run_base
RUN zypper install -y sqlite3 openssl timezone

# // build artifacts
FROM build_base AS builder

COPY . /home/lifx/
RUN mkdir /home/lifx/.cargo
WORKDIR /home/lifx/

RUN cp cargo_vendor.config .cargo/config && \
    cargo build --release

# == end builder setup, we now have static artifacts.
FROM run_base
MAINTAINER william@blackhats.net.au
EXPOSE 8081
WORKDIR /

RUN cd /etc && \
    ln -sf ../usr/share/zoneinfo/Australia/Brisbane localtime

COPY --from=builder /home/lifx/target/release/lifx_ctl /bin/
COPY --from=builder /home/lifx/static /static
COPY --from=builder /home/lifx/pkg /pkg

ENV RUST_BACKTRACE 1
CMD ["/bin/lifx_ctl"]

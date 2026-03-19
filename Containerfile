FROM rockylinux:9 AS builder

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:${PATH} \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=gcc \
    LIBRARY_PATH=/usr/lib64:/lib64 \
    LD_LIBRARY_PATH=/usr/lib64:/lib64 \
    PKG_CONFIG_PATH=/usr/lib64/pkgconfig:/usr/share/pkgconfig

RUN dnf install -y \
    ca-certificates \
    gcc \
    gcc-c++ \
    glibc-devel \
    krb5-libs \
    make \
    openldap \
    openssl-libs \
    perl \
    pkgconf-pkg-config \
    postgresql-devel \
    openssl-devel \
    sqlite-devel \
    xz

RUN update-ca-trust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal --default-toolchain stable

WORKDIR /app

COPY . .

RUN cargo build --release --bin projects_backend_database \
    --config target.aarch64-unknown-linux-gnu.linker='"gcc"' \
    --config target.aarch64-unknown-linux-gnu.rustflags=[]

FROM scratch AS artifact

COPY --from=builder /app/target/release/projects_backend_database /out/projects_backend_database

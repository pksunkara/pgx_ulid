# use 12, 13, 14, 15, 15
ARG PG_MAJOR

FROM postgres:${PG_MAJOR} as build

RUN apt-get update

ENV build_deps ca-certificates \
  git \
  build-essential \
  libpq-dev \
  postgresql-server-dev-${PG_MAJOR} \
  curl \
  libreadline6-dev \
  zlib1g-dev

RUN apt-get install -y --no-install-recommends $build_deps pkg-config cmake

WORKDIR /home/postgres

ENV HOME=/home/postgres
ENV PATH=/home/postgres/.cargo/bin:$PATH

RUN chown postgres:postgres /home/postgres

USER postgres

RUN \
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain 1.74.0 && \
  rustup --version && \
  rustc --version && \
  cargo --version

# pgrx
RUN cargo install cargo-pgrx --version 0.11.1 --locked

# init postgress dev env for target version
RUN cargo pgrx init --pg${PG_MAJOR} $(which pg_config)

# Compile as ROOT to avoid a permission denied when copying to /usr/share/postgresql
# USER root

COPY . .

RUN cargo pgrx package


# RUN chown -R postgres:postgres /home/postgres
# RUN chown -R postgres:postgres /usr/share/postgresql/${PG_MAJOR}/extension
# RUN chown -R postgres:postgres /usr/lib/postgresql/${PG_MAJOR}/lib

# multi-stage - let's start clean
FROM postgres:${PG_MAJOR}

COPY --from=build /home/postgres/target/release/ulid-pg${PG_MAJOR}/ /

# # COPY --from=build /home/postgres/${PG_MAJOR}/extension/ulid.control /home/postgres/${PG_MAJOR}/extension/ulid.control
# COPY --from=build /usr/share/postgresql/${PG_MAJOR}/extension/ulid*.* /usr/share/postgresql/${PG_MAJOR}/extension/
# COPY --from=build /usr/lib/postgresql/${PG_MAJOR}/lib/ulid.so /usr/lib/postgresql/${PG_MAJOR}/lib/ulid.so
# # why? convenient, but should stick to upstream behaviors.
# USER postgres
# # allow deployment without a password
ENV POSTGRES_HOST_AUTH_METHOD=trust
ENV USER=postgres

# add comma for more shared_preload_libraries=A,B,C
CMD ["postgres","-c","shared_preload_libraries=ulid"]
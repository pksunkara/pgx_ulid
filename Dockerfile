ARG PG_MAJOR

FROM postgres:${PG_MAJOR}

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
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain nightly-2023-03-06 && \
  rustup --version && \
  rustc --version && \
  cargo --version

# PGX
RUN cargo install cargo-pgx --version 0.7.3 --locked

RUN cargo pgx init --pg${PG_MAJOR} $(which pg_config)

USER root

COPY . .

RUN cargo pgx install

RUN chown -R postgres:postgres /home/postgres
RUN chown -R postgres:postgres /usr/share/postgresql/${PG_MAJOR}/extension
RUN chown -R postgres:postgres /usr/lib/postgresql/${PG_MAJOR}/lib

USER postgres

ENV POSTGRES_HOST_AUTH_METHOD=trust
ENV USER=postgres

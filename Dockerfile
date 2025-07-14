ARG PG_MAJOR
ARG NEXT

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
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain 1.85.0 && \
  rustup --version && \
  rustc --version && \
  cargo --version

ARG NEXT
RUN if [ "$NEXT" = "true" ] ; then \
  echo "Installing pgrx from GitHub repository" && \
  cargo install --git https://github.com/pgcentralfoundation/pgrx --branch develop cargo-pgrx --locked ; \
  else \
  echo "Installing pgrx from crates.io" && \
  cargo install cargo-pgrx --version 0.12.7 --locked ; \
  fi

RUN cargo pgrx init --pg${PG_MAJOR} $(which pg_config)

USER root

COPY . .

RUN if [ "$NEXT" = "true" ] ; then \
  # update Rust edition from what exists in Cargo.toml
  sed -i 's/edition[[:space:]]*=[[:space:]]*"[^"]*"/edition = "2024"/g' Cargo.toml && \
  # update Rust version from what exists in Cargo.toml
  sed -i 's/rust-version[[:space:]]*=[[:space:]]*"[^"]*"/rust-version = "1.85.0"/g' Cargo.toml && \
  # update pgrx versions from what exists in Cargo.toml
  sed -i 's/pgrx[[:space:]]*=[[:space:]]*"[^"]*"/pgrx = "0.15.0"/g' Cargo.toml && \
  sed -i 's/pgrx-tests[[:space:]]*=[[:space:]]*"[^"]*"/pgrx-tests = "0.15.0"/g' Cargo.toml && \
  # add required features for next PG
  sed -i '/pg17[[:space:]]*=[[:space:]]*\[.*\]/a pg18    = ["pgrx-tests/pg18", "pgrx/pg18"]' Cargo.toml && \
  echo "=== Updated Cargo.toml contents ===" && \
  cat Cargo.toml ; \
  fi

RUN cargo pgrx install

RUN chown -R postgres:postgres /home/postgres
RUN chown -R postgres:postgres /usr/share/postgresql/${PG_MAJOR}/extension
RUN chown -R postgres:postgres /usr/lib/postgresql/${PG_MAJOR}/lib

USER postgres

ENV POSTGRES_HOST_AUTH_METHOD=trust
ENV USER=postgres

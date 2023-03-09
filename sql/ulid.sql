-- complain if script is sourced in psql, rather than via CREATE EXTENSION
\echo Use "CREATE EXTENSION ulid;" to load this file. \quit

-- type

-- CREATE FUNCTION ulid_typmod_in(cstring[]) RETURNS integer
--   AS 'MODULE_PATHNAME' LANGUAGE C IMMUTABLE STRICT PARALLEL SAFE;

-- CREATE FUNCTION ulid_recv(internal, oid, integer) RETURNS ulid
--   AS 'MODULE_PATHNAME' LANGUAGE C IMMUTABLE STRICT PARALLEL SAFE;

-- CREATE FUNCTION ulid_send(ulid) RETURNS bytea
--   AS 'MODULE_PATHNAME' LANGUAGE C IMMUTABLE STRICT PARALLEL SAFE;

-- CREATE TYPE ulid (
--   INPUT     = ulid_in,
--   OUTPUT    = ulid_out,
--   TYPMOD_IN = ulid_typmod_in,
--   RECEIVE   = ulid_recv,
--   SEND      = ulid_send,
--   STORAGE   = extended
-- );

-- CREATE FUNCTION ulid_to_timestamp(ulid, integer, boolean) RETURNS timestamp
--   AS 'MODULE_PATHNAME' LANGUAGE C IMMUTABLE STRICT PARALLEL SAFE;

-- CREATE CAST (ulid AS timestamp)
--   WITH FUNCTION ulid_to_timestamp(ulid, integer, boolean) AS ASSIGNMENT;

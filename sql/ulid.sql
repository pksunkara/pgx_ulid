-- complain if script is sourced in psql, rather than via CREATE EXTENSION
\echo Use "CREATE EXTENSION ulid;" to load this file. \quit

-- type

-- CREATE TYPE ulid;

-- CREATE FUNCTION ulid_in(cstring, oid, integer) RETURNS ulid
--   AS 'MODULE_PATHNAME' LANGUAGE C IMMUTABLE STRICT PARALLEL SAFE;

-- CREATE FUNCTION ulid_out(ulid) RETURNS cstring
--   AS 'MODULE_PATHNAME' LANGUAGE C IMMUTABLE STRICT PARALLEL SAFE;

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

-- functions

-- CREATE FUNCTION generate_ulid() RETURNS ulid
--   AS 'MODULE_PATHNAME' LANGUAGE C IMMUTABLE STRICT PARALLEL SAFE;

-- cast functions

-- CREATE FUNCTION ulid(ulid, integer, boolean) RETURNS ulid
--   AS 'MODULE_PATHNAME' LANGUAGE C IMMUTABLE STRICT PARALLEL SAFE;

-- CREATE FUNCTION ulid_to_text(ulid, integer, boolean) RETURNS text
--   AS 'MODULE_PATHNAME' LANGUAGE C IMMUTABLE STRICT PARALLEL SAFE;

-- CREATE FUNCTION text_to_ulid(text, integer, boolean) RETURNS ulid
--   AS 'MODULE_PATHNAME' LANGUAGE C IMMUTABLE STRICT PARALLEL SAFE;

-- CREATE FUNCTION ulid_to_timestamp(ulid, integer, boolean) RETURNS timestamp
--   AS 'MODULE_PATHNAME' LANGUAGE C IMMUTABLE STRICT PARALLEL SAFE;

-- casts

-- CREATE CAST (ulid AS ulid)
--   WITH FUNCTION ulid(ulid, integer, boolean) AS IMPLICIT;

-- CREATE CAST (ulid AS text)
--   WITH FUNCTION ulid_to_text(ulid, integer, boolean) AS IMPLICIT;

-- CREATE CAST (text AS ulid)
--   WITH FUNCTION text_to_ulid(text, integer, boolean) AS ASSIGNMENT;

-- CREATE CAST (ulid AS timestamp)
--   WITH FUNCTION ulid_to_timestamp(ulid, integer, boolean) AS ASSIGNMENT;

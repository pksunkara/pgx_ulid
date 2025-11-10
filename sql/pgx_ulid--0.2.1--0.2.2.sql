CREATE FUNCTION "ulid_recv"(
  "internal" internal
) RETURNS ulid
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ulid_recv_wrapper';

CREATE FUNCTION "ulid_send"(
  "input" ulid
) RETURNS bytea
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ulid_send_wrapper';

ALTER TYPE ulid SET (
    SEND = ulid_send,
    RECEIVE = ulid_recv
);

CREATE FUNCTION "ulid_to_timestamptz"(
  "input" ulid
) RETURNS timestamp with time zone
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ulid_to_timestamptz_wrapper';

CREATE CAST (ulid AS timestamptz) WITH FUNCTION ulid_to_timestamptz(ulid) AS IMPLICIT;

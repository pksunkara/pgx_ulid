CREATE FUNCTION "ulid_max_ulid_max_combine"(
  "this" ulid,
  "v" ulid
) RETURNS ulid
STRICT
LANGUAGE c
AS 'MODULE_PATHNAME', 'ulid_max_ulid_max_combine_wrapper';

CREATE FUNCTION "ulid_max_ulid_max_state"(
  "this" ulid,
  "arg_one" ulid
) RETURNS ulid
STRICT
LANGUAGE c
AS 'MODULE_PATHNAME', 'ulid_max_ulid_max_state_wrapper';

CREATE FUNCTION "ulid_min_ulid_min_combine"(
  "this" ulid,
  "v" ulid
) RETURNS ulid
STRICT
LANGUAGE c
AS 'MODULE_PATHNAME', 'ulid_min_ulid_min_combine_wrapper';

CREATE FUNCTION "ulid_min_ulid_min_state"(
  "this" ulid,
  "arg_one" ulid
) RETURNS ulid
STRICT
LANGUAGE c
AS 'MODULE_PATHNAME', 'ulid_min_ulid_min_state_wrapper';

CREATE AGGREGATE max(ulid) (
  SFUNC = "ulid_max_ulid_max_state",
  STYPE = ulid,
  COMBINEFUNC = "ulid_max_ulid_max_combine"
);

CREATE AGGREGATE min(ulid) (
  SFUNC = "ulid_min_ulid_min_state",
  STYPE = ulid,
  COMBINEFUNC = "ulid_min_ulid_min_combine"
);

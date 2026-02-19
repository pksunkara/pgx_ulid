use core::ffi::CStr;
use std::time::{Duration, SystemTime};

use inner_ulid::Ulid as InnerUlid;
use pg_sys::Datum as SysDatum;
use pgrx::callconv::{ArgAbi, BoxRet};
use pgrx::datum::Datum;
use pgrx::{
    pg_shmem_init, pg_sys::Oid, prelude::*, rust_regtypein, PgLwLock, PgSharedMemoryInitialization,
    StringInfo, Uuid,
};
use serde::{Deserialize, Serialize};

::pgrx::pg_module_magic!();

static SHARED_ULID: PgLwLock<u128> = unsafe { PgLwLock::new(c"pgx_ulid_shared") };

#[pg_guard]
pub extern "C-unwind" fn _PG_init() {
    pg_shmem_init!(SHARED_ULID);
}

#[allow(non_camel_case_types)]
#[derive(
    PostgresType, PostgresEq, PostgresHash, PostgresOrd, Debug, PartialEq, PartialOrd, Eq, Hash, Ord,
)]
#[inoutfuncs]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
#[pg_binary_protocol]
#[bikeshed_postgres_type_manually_impl_from_into_datum]
pub struct ulid(u128);

impl InOutFuncs for ulid {
    #[inline]
    fn input(input: &CStr) -> Self
    where
        Self: Sized,
    {
        let val = input.to_str().unwrap();

        match InnerUlid::from_string(val) {
            Ok(inner) => ulid(inner.0),
            Err(err) => {
                ereport!(
                    ERROR,
                    PgSqlErrorCode::ERRCODE_INVALID_TEXT_REPRESENTATION,
                    format!("invalid input syntax for type ulid: \"{val}\": {err}")
                );
            }
        }
    }

    #[inline]
    fn output(&self, buffer: &mut StringInfo) {
        buffer.push_str(&InnerUlid(self.0).to_string())
    }
}

impl IntoDatum for ulid {
    #[inline]
    fn into_datum(self) -> Option<SysDatum> {
        self.0.to_ne_bytes().into_datum()
    }

    #[inline]
    fn type_oid() -> Oid {
        rust_regtypein::<Self>()
    }
}

impl FromDatum for ulid {
    #[inline]
    unsafe fn from_polymorphic_datum(datum: SysDatum, is_null: bool, typoid: Oid) -> Option<Self>
    where
        Self: Sized,
    {
        let bytes: &[u8] = FromDatum::from_polymorphic_datum(datum, is_null, typoid)?;

        let mut len_bytes = [0u8; 16];
        len_bytes.copy_from_slice(bytes);

        Some(ulid(u128::from_ne_bytes(len_bytes)))
    }
}

unsafe impl<'fcx> ArgAbi<'fcx> for ulid
where
    Self: 'fcx,
{
    unsafe fn unbox_arg_unchecked(arg: ::pgrx::callconv::Arg<'_, 'fcx>) -> Self {
        unsafe { arg.unbox_arg_using_from_datum().unwrap() }
    }
}

unsafe impl BoxRet for ulid {
    unsafe fn box_into<'fcx>(self, fcinfo: &mut pgrx::callconv::FcInfo<'fcx>) -> Datum<'fcx> {
        unsafe { fcinfo.return_raw_datum(self.into_datum().unwrap()) }
    }
}

#[pg_extern]
fn gen_monotonic_ulid() -> ulid {
    let mut shared_bytes = SHARED_ULID.exclusive();
    let shared_ulid = InnerUlid::from(*shared_bytes);
    let new_ulid = if shared_ulid.is_nil()
        || SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis()
            > u128::from(shared_ulid.timestamp_ms())
    {
        InnerUlid::new()
    } else {
        shared_ulid.increment().unwrap()
    };
    *shared_bytes = u128::from(new_ulid);
    ulid(*shared_bytes)
}

#[pg_extern]
fn gen_ulid() -> ulid {
    ulid(InnerUlid::new().0)
}

#[pg_extern(immutable, parallel_safe)]
fn ulid_from_uuid(input: Uuid) -> ulid {
    let mut bytes = *input.as_bytes();
    bytes.reverse();
    ulid(u128::from_ne_bytes(bytes))
}

#[pg_extern(immutable, parallel_safe)]
fn ulid_to_uuid(input: ulid) -> Uuid {
    let mut bytes = input.0.to_ne_bytes();
    bytes.reverse();
    Uuid::from_bytes(bytes)
}

#[pg_extern(immutable, parallel_safe)]
fn ulid_to_bytea(input: ulid) -> Vec<u8> {
    let mut bytes = input.0.to_ne_bytes();
    bytes.reverse();
    bytes.to_vec()
}

#[pg_extern(immutable, parallel_safe)]
fn ulid_to_timestamp(input: ulid) -> Timestamp {
    ulid_to_timestamptz(input).into()
}

#[pg_extern(immutable, parallel_safe)]
fn timestamp_to_ulid(input: Timestamp) -> ulid {
    let epoch: f64 = input
        .extract_part(DateTimeParts::Epoch)
        .unwrap()
        .try_into()
        .unwrap();

    let milliseconds = (epoch * 1000.0) as u64;

    let inner = InnerUlid::from_parts(milliseconds, 0);

    ulid(inner.0)
}

#[pg_extern(immutable, parallel_safe)]
fn ulid_to_timestamptz(input: ulid) -> TimestampWithTimeZone {
    let inner_seconds = (InnerUlid(input.0).timestamp_ms() as f64) / 1000.0;
    to_timestamp(inner_seconds)
}

#[pg_extern(immutable, parallel_safe)]
fn timestamptz_to_ulid(input: TimestampWithTimeZone) -> ulid {
    let epoch: f64 = input
        .extract_part(DateTimeParts::Epoch)
        .unwrap()
        .try_into()
        .unwrap();

    let milliseconds = (epoch * 1000.0) as u64;

    let inner = InnerUlid::from_parts(milliseconds, 0);

    ulid(inner.0)
}

/// State transition for MIN(ulid) aggregate: returns the smaller of state and value.
/// Accepts NULL state (first row) and returns the value.
#[pg_extern(immutable, parallel_safe)]
fn ulid_smaller(state: Option<ulid>, value: ulid) -> ulid {
    match state {
        None => value,
        Some(s) => if s <= value { s } else { value },
    }
}

/// State transition for MAX(ulid) aggregate: returns the larger of state and value.
/// Accepts NULL state (first row) and returns the value.
#[pg_extern(immutable, parallel_safe)]
fn ulid_larger(state: Option<ulid>, value: ulid) -> ulid {
    match state {
        None => value,
        Some(s) => if s >= value { s } else { value },
    }
}

extension_sql!(
    r#"
CREATE CAST (uuid AS ulid) WITH FUNCTION ulid_from_uuid(uuid) AS IMPLICIT;
CREATE CAST (ulid AS uuid) WITH FUNCTION ulid_to_uuid(ulid) AS ASSIGNMENT;
CREATE CAST (ulid AS bytea) WITH FUNCTION ulid_to_bytea(ulid) AS ASSIGNMENT;
CREATE CAST (ulid AS timestamp) WITH FUNCTION ulid_to_timestamp(ulid) AS ASSIGNMENT;
CREATE CAST (timestamp AS ulid) WITH FUNCTION timestamp_to_ulid(timestamp) AS IMPLICIT;
CREATE CAST (ulid AS timestamptz) WITH FUNCTION ulid_to_timestamptz(ulid) AS ASSIGNMENT;
CREATE CAST (timestamptz AS ulid) WITH FUNCTION timestamptz_to_ulid(timestamptz) AS IMPLICIT;
"#,
    name = "ulid_casts",
    requires = [
        ulid_from_uuid,
        ulid_to_uuid,
        ulid_to_bytea,
        ulid_to_timestamp,
        timestamp_to_ulid,
        ulid_to_timestamptz,
        timestamptz_to_ulid
    ]
);

extension_sql!(
    r#"
CREATE AGGREGATE min(ulid) (
    SFUNC = ulid_smaller,
    STYPE = ulid
);

CREATE AGGREGATE max(ulid) (
    SFUNC = ulid_larger,
    STYPE = ulid
);
"#,
    name = "ulid_aggregates",
    requires = [ulid_smaller, ulid_larger]
);

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    const INT: u128 = 2029121117734015635515926905565997019;
    const TEXT: &str = "01GV5PA9EQG7D82Q3Y4PKBZSYV";
    const UUID: &str = "0186cb65-25d7-81da-815c-7e25a6bfe7db";
    const BYTEA: &[u8] = &[
        1, 134, 203, 101, 37, 215, 129, 218, 129, 92, 126, 37, 166, 191, 231, 219,
    ];
    const TIMESTAMP: &str = "2023-03-10 12:00:49.111";
    const TIMESTAMPTZ: &str = "2023-03-10 12:00:49.111+00";

    #[pg_test]
    fn test_null_to_ulid() {
        let result = Spi::get_one::<ulid>("SELECT NULL::ulid;").unwrap();
        assert_eq!(None, result);
    }

    #[pg_test]
    fn test_string_to_ulid() {
        let result = Spi::get_one::<ulid>(&format!("SELECT '{TEXT}'::ulid;")).unwrap();
        assert_eq!(Some(ulid(INT)), result);
    }

    #[pg_test]
    fn test_ulid_to_string() {
        let result = Spi::get_one::<&str>(&format!("SELECT '{TEXT}'::ulid::text;")).unwrap();
        assert_eq!(Some(TEXT), result);
    }

    #[pg_test]
    fn test_string_to_ulid_lowercase() {
        let result = Spi::get_one::<ulid>(&format!("SELECT LOWER('{TEXT}')::ulid;")).unwrap();
        assert_eq!(Some(ulid(INT)), result);
    }

    #[pg_test]
    #[should_panic = "invalid input syntax for type ulid: \"01GV5PA9EQG7D82Q3Y4PKBZSY\": invalid length"]
    fn test_string_to_ulid_invalid_length() {
        let _ = Spi::get_one::<ulid>("SELECT '01GV5PA9EQG7D82Q3Y4PKBZSY'::ulid;");
    }

    #[pg_test]
    #[should_panic = "invalid input syntax for type ulid: \"01GV5PA9EQG7D82Q3Y4PKBZSYU\": invalid character"]
    fn test_string_to_ulid_invalid_char() {
        let _ = Spi::get_one::<ulid>("SELECT '01GV5PA9EQG7D82Q3Y4PKBZSYU'::ulid;");
    }

    #[pg_test]
    fn test_ulid_to_timestamp() {
        let result = Spi::get_one::<&str>(&format!(
            "SET TIMEZONE TO 'UTC'; SELECT '{TEXT}'::ulid::timestamp::text;"
        ))
        .unwrap();
        assert_eq!(Some(TIMESTAMP), result);
    }

    #[pg_test]
    fn test_timestamp_to_ulid() {
        let result = Spi::get_one::<&str>(&format!(
            "SET TIMEZONE TO 'UTC'; SELECT '{TIMESTAMP}'::timestamp::ulid::text;"
        ))
        .unwrap();
        assert_eq!(Some("01GV5PA9EQ0000000000000000"), result);
    }

    #[pg_test]
    fn test_ulid_to_timestamptz() {
        let result = Spi::get_one::<&str>(&format!(
            "SET TIMEZONE TO 'UTC'; SELECT '{TEXT}'::ulid::timestamptz::text;"
        ))
        .unwrap();
        assert_eq!(Some(TIMESTAMPTZ), result);
    }

    #[pg_test]
    fn test_timestamptz_to_ulid() {
        let result = Spi::get_one::<&str>(&format!(
            "SET TIMEZONE TO 'UTC'; SELECT '{TIMESTAMP}'::timestamptz::ulid::text;"
        ))
        .unwrap();
        assert_eq!(Some("01GV5PA9EQ0000000000000000"), result);
    }

    #[pg_test]
    fn test_ulid_to_uuid() {
        let result = Spi::get_one::<&str>(&format!("SELECT '{TEXT}'::ulid::uuid::text;")).unwrap();
        assert_eq!(Some(UUID), result);
    }

    #[pg_test]
    fn test_ulid_to_bytea() {
        let result = Spi::get_one::<&[u8]>(&format!("SELECT '{TEXT}'::ulid::bytea;")).unwrap();

        assert_eq!(Some(BYTEA), result);
    }

    #[pg_test]
    fn test_uuid_to_ulid() {
        let result = Spi::get_one::<ulid>(&format!("SELECT '{UUID}'::uuid::ulid;")).unwrap();
        assert_eq!(Some(ulid(INT)), result);
    }

    #[pg_test]
    fn test_generate() {
        let result = Spi::get_one::<ulid>("SELECT gen_ulid();").unwrap();
        assert!(result.is_some());
    }

    #[pg_test]
    fn test_hash() {
        Spi::run(
            "CREATE TABLE foo (
                id ulid,
                data TEXT
            );

            CREATE TABLE bar (
                id ulid,
                foo_id ulid
            );

            INSERT INTO foo DEFAULT VALUES;
            INSERT INTO bar DEFAULT VALUES;

            SELECT *
            FROM bar
            JOIN foo ON bar.id = foo.id;",
        )
        .unwrap();
    }

    #[pg_test]
    fn test_commutator() {
        Spi::run(
            "CREATE TABLE foo (
                id ulid,
                data TEXT
            );

            CREATE TABLE bar (
                id ulid
            );

            SELECT *
            FROM bar
            JOIN foo ON bar.id = foo.id;",
        )
        .unwrap();
    }

    #[pg_test]
    fn test_ulid_equals_ulid_operator_unique() {
        // Operator = (ulid, ulid) must be unique so expressions like a = b and joins work
        // without "operator is not unique" (no resolution via ulid->bytea or ulid->uuid).
        let result = Spi::get_one::<bool>(
            "SELECT CAST('01HTHV582X9B5YBFP7GRBWXRHX' AS ulid) = CAST('01HTHV582X9B5YBFP7GRBWXRHX' AS ulid);",
        )
        .unwrap();
        assert_eq!(Some(true), result);
        let result = Spi::get_one::<bool>("SELECT gen_ulid() = gen_ulid();").unwrap();
        assert_eq!(Some(false), result);
    }

    #[pg_test]
    fn test_min_ulid_aggregate() {
        // min(ulid) works without explicit cast
        let result = Spi::get_one::<String>(
            "SELECT min(id)::text FROM (VALUES ('01GV5PA9EQG7D82Q3Y4PKBZSYV'::ulid), ('01GV5PA9EQ0000000000000000'::ulid)) AS t(id);",
        )
        .unwrap();
        assert_eq!(Some("01GV5PA9EQ0000000000000000".to_string()), result);
    }

    #[pg_test]
    fn test_max_ulid_aggregate() {
        // max(ulid) works without explicit cast
        let result = Spi::get_one::<String>(
            "SELECT max(id)::text FROM (VALUES ('01GV5PA9EQG7D82Q3Y4PKBZSYV'::ulid), ('01GV5PA9EQ0000000000000000'::ulid)) AS t(id);",
        )
        .unwrap();
        assert_eq!(Some("01GV5PA9EQG7D82Q3Y4PKBZSYV".to_string()), result);
    }

    #[pg_test]
    fn test_min_max_ulid_group_by_order_by() {
        // Reproduces the user scenario: group by ... order by max(id) without cast
        Spi::run(
            r#"
            CREATE TABLE contacts (
                id ulid,
                bucket int
            );
            INSERT INTO contacts (id, bucket) VALUES
                ('01GV5PA9EQ0000000000000000'::ulid, 1),
                ('01GV5PA9EQG7D82Q3Y4PKBZSYV'::ulid, 1),
                ('01GV5PA9EQG7D82Q3Y4PKBZSYW'::ulid, 2);
            "#,
        )
        .unwrap();
        // ORDER BY max(id) and SELECT max(id) work without ::bytea cast
        let result = Spi::get_one::<String>(
            "SELECT max(id)::text FROM contacts GROUP BY bucket ORDER BY max(id) DESC LIMIT 1;",
        )
        .unwrap();
        assert_eq!(Some("01GV5PA9EQG7D82Q3Y4PKBZSYW".to_string()), result);
        let result_min = Spi::get_one::<String>(
            "SELECT min(id)::text FROM contacts GROUP BY bucket ORDER BY min(id) LIMIT 1;",
        )
        .unwrap();
        assert_eq!(Some("01GV5PA9EQ0000000000000000".to_string()), result_min);
    }

    // TODO: Add tests for binary protocol
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    #[must_use]
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}

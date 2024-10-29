use core::ffi::CStr;
use inner_ulid::Ulid as InnerUlid;
use pgrx::callconv::{ArgAbi, BoxRet};
use pgrx::datum::Datum;
use pgrx::pgrx_sql_entity_graph::metadata::{
    ArgumentError, Returns, ReturnsError, SqlMapping, SqlTranslatable,
};
use pgrx::{
    pg_shmem_init, pg_sys::Oid, prelude::*, rust_regtypein, shmem::*, PgLwLock, StringInfo, Uuid,
};
use std::error::Error;
use std::time::{Duration, SystemTime};

::pgrx::pg_module_magic!();

static SHARED_ULID: PgLwLock<u128> = PgLwLock::new();

#[pg_guard]
pub extern "C" fn _PG_init() {
    pg_shmem_init!(SHARED_ULID);
}

#[allow(non_camel_case_types)]
#[derive(PostgresEq, PostgresHash, PostgresOrd, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub struct ulid {
    numeric: u128,
}

#[pg_extern(immutable, parallel_safe, requires = [ "shell_type" ])]
fn ulid_in(input: &CStr) -> Result<ulid, Box<dyn Error>> {
    let val = input.to_str().unwrap();
    match InnerUlid::from_string(val) {
        Ok(inner) => Ok(ulid { numeric: inner.0 }),
        Err(err) => {
            ereport!(
                ERROR,
                PgSqlErrorCode::ERRCODE_INVALID_TEXT_REPRESENTATION,
                format!("invalid input syntax for type ulid: \"{val}\": {err}")
            );
        }
    }
}

#[pg_extern(immutable, parallel_safe, requires = [ "shell_type" ])]
fn ulid_out(value: ulid) -> &'static CStr {
    let mut s = StringInfo::new();
    s.push_str(&InnerUlid(value.numeric).to_string());
    // SAFETY: We just constructed this StringInfo ourselves
    unsafe { s.leak_cstr() }
}

impl IntoDatum for ulid {
    #[inline]
    fn into_datum(self) -> Option<pg_sys::Datum> {
        self.numeric.to_ne_bytes().into_datum()
    }

    #[inline]
    fn type_oid() -> Oid {
        rust_regtypein::<Self>()
    }
}

impl FromDatum for ulid {
    #[inline]
    unsafe fn from_polymorphic_datum(
        datum: pg_sys::Datum,
        is_null: bool,
        typoid: Oid,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        let bytes: &[u8] = FromDatum::from_polymorphic_datum(datum, is_null, typoid)?;

        let mut len_bytes = [0u8; 16];
        len_bytes.copy_from_slice(bytes);

        Some(ulid {
            numeric: u128::from_ne_bytes(len_bytes),
        })
    }
}

unsafe impl SqlTranslatable for ulid {
    fn argument_sql() -> Result<SqlMapping, ArgumentError> {
        // this is what the SQL type is called when used in a function argument position
        Ok(SqlMapping::As("ulid".into()))
    }

    fn return_sql() -> Result<Returns, ReturnsError> {
        // this is what the SQL type is called when used in a function return type position
        Ok(Returns::One(SqlMapping::As("ulid".into())))
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
    ulid {
        numeric: *shared_bytes,
    }
}

#[pg_extern]
fn gen_ulid() -> ulid {
    ulid {
        numeric: InnerUlid::new().0,
    }
}

#[pg_extern(immutable, parallel_safe)]
fn ulid_from_uuid(input: Uuid) -> ulid {
    let mut bytes = *input.as_bytes();
    bytes.reverse();
    ulid {
        numeric: u128::from_ne_bytes(bytes),
    }
}

#[pg_extern(immutable, parallel_safe)]
fn ulid_to_uuid(input: ulid) -> Uuid {
    let mut bytes = input.numeric.to_ne_bytes();
    bytes.reverse();
    Uuid::from_bytes(bytes)
}

#[pg_extern(immutable, parallel_safe)]
fn ulid_to_bytea(input: ulid) -> Vec<u8> {
    let mut bytes = input.numeric.to_ne_bytes();
    bytes.reverse();
    bytes.to_vec()
}

#[pg_extern(immutable, parallel_safe)]
fn ulid_to_timestamp(input: ulid) -> Timestamp {
    let inner_seconds = (InnerUlid(input.numeric).timestamp_ms() as f64) / 1000.0;
    to_timestamp(inner_seconds).into()
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

    ulid { numeric: inner.0 }
}

// Creates the `ulid` shell type, which is essentially a type placeholder so that the
// input and output functions can be created
extension_sql!(
    r#"
CREATE TYPE ulid; -- Shell type
"#,
    name = "shell_type",
    creates = [Type(ulid)],
    bootstrap // Declare this extension_sql block as the "bootstrap" block so that it happens first in SQL generation
);

// Create the actual type, specifying the input and output functions
extension_sql!(
    r#"
CREATE TYPE ulid (
  INPUT = ulid_in,
  OUTPUT = ulid_out,
  LIKE = uuid
);
"#,
    name = "concrete_type",
    creates = [Type(ulid)],
    requires = ["shell_type", ulid_in, ulid_out], // So that we won't be created until the shell type and input and output functions have
);

extension_sql!(
    r#"
CREATE CAST (uuid AS ulid) WITH FUNCTION ulid_from_uuid(uuid) AS IMPLICIT;
CREATE CAST (ulid AS uuid) WITH FUNCTION ulid_to_uuid(ulid) AS IMPLICIT;
CREATE CAST (ulid AS bytea) WITH FUNCTION ulid_to_bytea(ulid) AS IMPLICIT;
CREATE CAST (ulid AS timestamp) WITH FUNCTION ulid_to_timestamp(ulid) AS IMPLICIT;
CREATE CAST (timestamp AS ulid) WITH FUNCTION timestamp_to_ulid(timestamp) AS IMPLICIT;
"#,
    name = "ulid_casts",
    requires = [
        ulid_from_uuid,
        ulid_to_uuid,
        ulid_to_bytea,
        ulid_to_timestamp,
        timestamp_to_ulid
    ]
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

    #[pg_test]
    fn test_null_to_ulid() {
        let result = Spi::get_one::<ulid>("SELECT NULL::ulid;").unwrap();
        assert_eq!(None, result);
    }

    #[pg_test]
    fn test_string_to_ulid() {
        let result = Spi::get_one::<ulid>(&format!("SELECT '{TEXT}'::ulid;")).unwrap();
        assert_eq!(Some(ulid { numeric: INT }), result);
    }

    #[pg_test]
    fn test_ulid_to_string() {
        let result = Spi::get_one::<&str>(&format!("SELECT '{TEXT}'::ulid::text;")).unwrap();
        assert_eq!(Some(TEXT), result);
    }

    #[pg_test]
    fn test_string_to_ulid_lowercase() {
        let result = Spi::get_one::<ulid>(&format!("SELECT LOWER('{TEXT}')::ulid;")).unwrap();
        assert_eq!(Some(ulid { numeric: INT }), result);
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
        assert_eq!(Some(ulid { numeric: INT }), result);
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

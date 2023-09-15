use core::ffi::CStr;
use inner_ulid::Ulid as InnerUlid;
use pgrx::{
    pg_shmem_init,
    pg_sys::{Datum, Oid},
    prelude::*,
    rust_regtypein,
    shmem::*,
    PgLwLock, StringInfo, Uuid,
};
use std::time::{Duration, SystemTime};

pgrx::pg_module_magic!();

static SHARED_ULID: PgLwLock<u128> = PgLwLock::new();

#[pg_guard]
pub extern "C" fn _PG_init() {
    pg_shmem_init!(SHARED_ULID);
}

#[allow(non_camel_case_types)]
#[derive(
    PostgresType, PostgresEq, PostgresHash, PostgresOrd, Debug, PartialEq, PartialOrd, Eq, Hash, Ord,
)]
#[inoutfuncs]
pub struct ulid(u128);

impl InOutFuncs for ulid {
    #[inline]
    fn input(input: &CStr) -> Self
    where
        Self: Sized,
    {
        let val = input.to_str().unwrap();
        let inner = InnerUlid::from_string(val)
            .unwrap_or_else(|err| panic!("invalid input syntax for type ulid: \"{val}\": {err}"));

        ulid(inner.0)
    }

    #[inline]
    fn output(&self, buffer: &mut StringInfo) {
        buffer.push_str(&InnerUlid(self.0).to_string())
    }
}

impl IntoDatum for ulid {
    #[inline]
    fn into_datum(self) -> Option<Datum> {
        self.0.to_ne_bytes().into_datum()
    }

    #[inline]
    fn type_oid() -> Oid {
        rust_regtypein::<Self>()
    }
}

impl FromDatum for ulid {
    #[inline]
    unsafe fn from_polymorphic_datum(datum: Datum, is_null: bool, typoid: Oid) -> Option<Self>
    where
        Self: Sized,
    {
        let bytes: &[u8] = FromDatum::from_polymorphic_datum(datum, is_null, typoid)?;

        let mut len_bytes = [0u8; 16];
        len_bytes.copy_from_slice(bytes);

        Some(ulid(u128::from_ne_bytes(len_bytes)))
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
fn ulid_to_timestamp(input: ulid) -> Timestamp {
    // 946684800000 is the number of milliseconds between 1970-01-01 and 2000-01-01
    let inner = InnerUlid(input.0).timestamp_ms() as i64 - 946_684_800_000;
    Timestamp::try_from(inner * 1000).unwrap()
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

extension_sql!(
    r#"
CREATE CAST (uuid AS ulid) WITH FUNCTION ulid_from_uuid(uuid) AS IMPLICIT;
CREATE CAST (ulid AS uuid) WITH FUNCTION ulid_to_uuid(ulid) AS IMPLICIT;
CREATE CAST (ulid AS timestamp) WITH FUNCTION ulid_to_timestamp(ulid) AS IMPLICIT;
CREATE CAST (timestamp AS ulid) WITH FUNCTION timestamp_to_ulid(timestamp) AS IMPLICIT;
"#,
    name = "ulid_casts"
);

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    const INT: u128 = 2029121117734015635515926905565997019;
    const TEXT: &str = "01GV5PA9EQG7D82Q3Y4PKBZSYV";
    const UUID: &str = "0186cb65-25d7-81da-815c-7e25a6bfe7db";
    const TIMESTAMP: &str = "2023-03-10 12:00:49.111";

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
        let result =
            Spi::get_one::<&str>(&format!("SELECT '{TEXT}'::ulid::timestamp::text;")).unwrap();
        assert_eq!(Some(TIMESTAMP), result);
    }

    #[pg_test]
    fn test_timestamp_to_ulid() {
        let result =
            Spi::get_one::<&str>(&format!("SELECT '{TIMESTAMP}'::timestamp::ulid::text;")).unwrap();
        assert_eq!(Some("01GV5PA9EQ0000000000000000"), result);
    }

    #[pg_test]
    fn test_ulid_to_uuid() {
        let result = Spi::get_one::<&str>(&format!("SELECT '{TEXT}'::ulid::uuid::text;")).unwrap();
        assert_eq!(Some(UUID), result);
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
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}

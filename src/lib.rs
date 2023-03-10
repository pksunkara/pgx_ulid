use ::ulid::Ulid as InnerUlid;
use core::ffi::CStr;
use pgx::{
    pg_sys::{Datum, Oid},
    prelude::*,
    StringInfo,
};

pgx::pg_module_magic!();

#[allow(non_camel_case_types)]
#[derive(PostgresType, PostgresEq, PostgresOrd, PartialEq, PartialOrd, Eq, Ord)]
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
            .expect(&format!("invalid input syntax for type ulid: \"{val}\""));

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
        pg_sys::BYTEAOID
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
fn gen_ulid() -> ulid {
    ulid(InnerUlid::new().0)
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgx::prelude::*;

    #[pg_test]
    fn test_string_to_ulid() {
        assert_eq!(1, 1);
    }
}

/// This module is required by `cargo pgx test` invocations.
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

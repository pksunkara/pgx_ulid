<!-- omit from toc -->
# pgx_ulid

A postgres extension to support [ulid][].

1. [Why should I use this?](#why-should-i-use-this)
2. [Why should I use ulid over uuid?](#why-should-i-use-ulid-over-uuid)
3. [Monotonicity](#monotonicity)
4. [Usage](#usage)
5. [Installation](#installation)

## Why should I use this?

There are several different postgres extensions for [ulid][], but all of them have feature gaps. A good extension should have:

- **Generator**: A generator function to generate [ulid][] identifiers.
- **Binary**: Data be stored as binary and not text.
- **Type**: A postgres type `ulid` which is displayed as [ulid][] text.
- **Uuid**: Support for casting between UUID and [ulid][]
- **Timestamp**: Support to cast an [ulid][] to a timestamp
- **Monotonic**: Support [monotonicity][]

|                             Name                              | Language | Generator | Binary | Type |  UUID  | Timestamp | Monotonic |
| :-----------------------------------------------------------: | :------: | :-------: | :----: | :--: | :----: | :-------: | :-------: |
|      [`pgx_ulid`](https://github.com/pksunkara/pgx_ulid)      |   Rust   |    ✔️      |   ✔️    |  ✔️   |   ✔️    |    ✔️      |    ✔️      |
|       [`pgulid`](https://github.com/geckoboard/pgulid)        | PL/pgSQL |    ✔️      |   ❌   |  ❌  |   ❌   |    ❌     |    ❌     |
|      [`pg_idkit`](https://github.com/VADOSWARE/pg_idkit)      |   Rust   |    ✔️      |   ❌   |  ❌  |   ❌   |    ❌     |    ❌     |
|   [`uids-postgres`](https://github.com/spa5k/uids-postgres)   |   Rust   |    ✔️      | ⁉️[^1]  |  ❌  | ⁉️[^2]  |    ❌     |    ❌     |
|    [`pgsql_ulid`](https://github.com/scoville/pgsql-ulid)     | PL/pgSQL |    ❌     | ⁉️[^1]  |  ❌  |   ✔️    |    ❌     |    ❌     |
|        [`pg-ulid`](https://github.com/edoceo/pg-ulid)         |    C     |    ✔️      |   ❌   |  ❌  |   ❌   |    ❌     |    ❌     |
| [`ulid-postgres`](https://github.com/schinckel/ulid-postgres) | PL/pgSQL |    ✔️      |   ❌   |  ✔️   |   ❌   |    ✔️      |    ❌     |
|       [`pg_ulid`](https://github.com/iCyberon/pg_ulid)        |    Go    |    ✔️      |   ❌   |  ❌  |   ❌   |    ✔️      |    ❌     |
|        [`pg_ulid`](https://github.com/RPG-18/pg_ulid)         |   C++    |    ✔️      | ⁉️[^1]  |  ❌  |   ✔️    |    ❌     |    ❌     |

[^1]: You can convert the [ulid][] into `uuid` or `bytea` and store it like that.
[^2]: Supports casting indirectly through `bytea`.

## Why should I use ulid over uuid?

The main advantages are:

* Indexes created over ULIDs are less fragmented compared to UUIDs due to the timestamp and [monotonicity][] that was encoded in the ULID when it was created.
* ULIDs don't use special characters, so they can be used in URLs or even HTML.
* ULIDs are shorter than UUIDs as they are comprised of 26 characters compared to UUIDs' 36 characters.

This extension is approximately **30% faster** than both `pgcrypto`'s UUID and `pg_uuidv7`'s UUIDv7 when generating a million identifiers.

<details>

```
ulid=# EXPLAIN ANALYSE SELECT gen_random_uuid() FROM generate_series(1, 1000000);
                                                            QUERY PLAN
-----------------------------------------------------------------------------------------------------------------------------------
 Function Scan on generate_series  (cost=0.00..12500.00 rows=1000000 width=16) (actual time=46.630..1401.638 rows=1000000 loops=1)
 Planning Time: 0.020 ms
 Execution Time: 1430.364 ms
(3 rows)

ulid=# EXPLAIN ANALYSE SELECT uuid_generate_v7() FROM generate_series(1, 1000000);
                                                            QUERY PLAN
-----------------------------------------------------------------------------------------------------------------------------------
 Function Scan on generate_series  (cost=0.00..12500.00 rows=1000000 width=16) (actual time=46.977..1427.477 rows=1000000 loops=1)
 Planning Time: 0.031 ms
 Execution Time: 1456.333 ms
(3 rows)

ulid=# EXPLAIN ANALYSE SELECT gen_ulid() FROM generate_series(1, 1000000);
                                                            QUERY PLAN
-----------------------------------------------------------------------------------------------------------------------------------
 Function Scan on generate_series  (cost=0.00..12500.00 rows=1000000 width=32) (actual time=46.820..1070.447 rows=1000000 loops=1)
 Planning Time: 0.020 ms
 Execution Time: 1098.086 ms
(3 rows)
```

</details>

This extension is approximately **20% faster** than both `pgcrypto`'s UUID and `pg_uuidv7`'s UUIDv7 when generating and inserting a million identifiers.

<details>

```
ulid=# EXPLAIN ANALYSE INSERT INTO uuid_keys(id) SELECT gen_random_uuid() FROM generate_series(1, 1000000);
                                                               QUERY PLAN
-----------------------------------------------------------------------------------------------------------------------------------------
 Insert on uuid_keys  (cost=0.00..22500.00 rows=0 width=0) (actual time=2006.633..2006.634 rows=0 loops=1)
   ->  Function Scan on generate_series  (cost=0.00..12500.00 rows=1000000 width=16) (actual time=46.846..1459.869 rows=1000000 loops=1)
 Planning Time: 0.029 ms
 Execution Time: 2008.195 ms
(4 rows)

ulid=# EXPLAIN ANALYSE INSERT INTO uuid7_keys(id) SELECT uuid_generate_v7() FROM generate_series(1, 1000000);
                                                               QUERY PLAN
-----------------------------------------------------------------------------------------------------------------------------------------
 Insert on uuid7_keys  (cost=0.00..22500.00 rows=0 width=0) (actual time=2030.731..2030.731 rows=0 loops=1)
   ->  Function Scan on generate_series  (cost=0.00..12500.00 rows=1000000 width=16) (actual time=46.894..1479.223 rows=1000000 loops=1)
 Planning Time: 0.030 ms
 Execution Time: 2032.296 ms
(4 rows)

ulid=# EXPLAIN ANALYSE INSERT INTO ulid_keys(id) SELECT gen_ulid() FROM generate_series(1, 1000000);
                                                               QUERY PLAN
-----------------------------------------------------------------------------------------------------------------------------------------
 Insert on ulid_keys  (cost=0.00..22500.00 rows=0 width=0) (actual time=1665.380..1665.380 rows=0 loops=1)
   ->  Function Scan on generate_series  (cost=0.00..12500.00 rows=1000000 width=32) (actual time=46.719..1140.979 rows=1000000 loops=1)
 Planning Time: 0.029 ms
 Execution Time: 1666.867 ms
(4 rows)
```

</details>

## Monotonicity

This extension supports [monotonicity][] through `gen_monotonic_ulid()` function. To achive this, it uses PostgreSQL's shared memory and LWLock to store last generated ULID.

To be able to use [monotonic][monotonicity] ULID's, it is necessary to add this extension to `postgresql.conf`'s `shared_preload_libraries` configuration setting.

```conf
shared_preload_libraries = 'ulid'	# (change requires restart)
```

<details>

```
ulid=# EXPLAIN ANALYSE SELECT gen_ulid() FROM generate_series(1, 1000000);
                                                            QUERY PLAN
-----------------------------------------------------------------------------------------------------------------------------------
 Function Scan on generate_series  (cost=0.00..12500.00 rows=1000000 width=32) (actual time=47.207..2908.978 rows=1000000 loops=1)
 Planning Time: 0.035 ms
 Execution Time: 4053.482 ms
(3 rows)

ulid=# EXPLAIN ANALYSE SELECT gen_monotonic_ulid() FROM generate_series(1, 1000000);
                                                            QUERY PLAN
-----------------------------------------------------------------------------------------------------------------------------------
 Function Scan on generate_series  (cost=0.00..12500.00 rows=1000000 width=32) (actual time=46.479..2586.654 rows=1000000 loops=1)
 Planning Time: 0.037 ms
 Execution Time: 3693.901 ms
(3 rows)
```

</details>

<details>

```
ulid=# EXPLAIN ANALYZE INSERT INTO users (name) SELECT 'Client 1' FROM generate_series(1, 1000000);
                                                               QUERY PLAN
-----------------------------------------------------------------------------------------------------------------------------------------
 Insert on users  (cost=0.00..12500.00 rows=0 width=0) (actual time=8418.257..8418.261 rows=0 loops=1)
   ->  Function Scan on generate_series  (cost=0.00..12500.00 rows=1000000 width=64) (actual time=99.804..3013.333 rows=1000000 loops=1)
 Planning Time: 0.066 ms
 Execution Time: 8419.571 ms
(4 rows)

ulid=# EXPLAIN ANALYZE INSERT INTO users (name) SELECT 'Client 2' FROM generate_series(1, 1000000);
                                                               QUERY PLAN
-----------------------------------------------------------------------------------------------------------------------------------------
 Insert on users  (cost=0.00..12500.00 rows=0 width=0) (actual time=8359.558..8359.561 rows=0 loops=1)
   ->  Function Scan on generate_series  (cost=0.00..12500.00 rows=1000000 width=64) (actual time=64.449..2976.754 rows=1000000 loops=1)
 Planning Time: 0.090 ms
 Execution Time: 8360.840 ms
(4 rows)
```

</details>

<!-- omit from toc -->
### Pros

1. Monotonic ULIDs are better for indexing, as they are sorted by default.
2. Monotonic ULIDs slightly faster than `gen_ulid()` when generating lots of ULIDs within one millisecond. Because, in this case, there is no need to generate random component of ULID. Instead it is just incremented.

<!-- omit from toc -->
### Cons

1. Previously generated ULID is saved in shmem and accessed via LWLock. i.e. it is exclusive for function invocation within database. Theoretically this can lead to slowdowns.

    *...But, in practice (at least in our testing) `gen_monotonic_ulid()` is slightly faster than `gen_ulid()`.*

2. Extensions that use shared memory must be loaded via `postgresql.conf`'s `shared_preload_libraries` configuration setting.

    *...But, it only affects `gen_monotonic_ulid()` function. Other functions of this extension will work normally even without this config.*

3. Monotonic ULIDs may overflow and throw an error.

    *...But, chances are negligible.*

## Usage

Use the extension in the database:

```sql
CREATE EXTENSION ulid;
```

Create a table with [ulid][] as a primary key:

```sql
CREATE TABLE users (
  id ulid NOT NULL DEFAULT gen_ulid() PRIMARY KEY,
  name text NOT NULL
);
```

Or, create a table with [monotonic][monotonicity] [ulid][] as a primary key:

```sql
CREATE TABLE users (
  id ulid NOT NULL DEFAULT gen_monotonic_ulid() PRIMARY KEY,
  name text NOT NULL
);
```

Operate it normally with text in queries:

```sql
SELECT * FROM users WHERE id = '01ARZ3NDEKTSV4RRFFQ69G5FAV';
```

Cast [ulid][] to timestamp:

```sql
ALTER TABLE users
ADD COLUMN created_at timestamp GENERATED ALWAYS AS (id::timestamp) STORED;
```

Cast timestamp to [ulid][], this generates a zeroed ULID with the timestamp prefixed (TTTTTTTTTT0000000000000000):

```sql
-- gets all users where the ID was created on 2023-09-15, without using another column and taking advantage of the index
SELECT * FROM users WHERE id BETWEEN '2023-09-15'::timestamp::ulid AND '2023-09-16'::timestamp::ulid;
```

## Installation

Use [pgrx][]. You can clone this repo and install this extension locally by following [this guide](https://github.com/tcdi/pgrx/blob/master/cargo-pgrx/README.md#installing-your-extension-locally).

You can also download relevant files from [releases](https://github.com/pksunkara/pgx_ulid/releases) page.

<!-- omit from toc -->
## Contributors

Here is a list of [Contributors](http://github.com/pksunkara/pgx_ulid/contributors)

<!-- omit from toc -->
### TODO

<!-- omit from toc -->
## License

MIT/X11

<!-- omit from toc -->
## Bug Reports

Report [here](http://github.com/pksunkara/pgx_ulid/issues).

<!-- omit from toc -->
## Creator

Pavan Kumar Sunkara (pavan.sss1991@gmail.com)

Follow me on [github](https://github.com/users/follow?target=pksunkara), [twitter](http://twitter.com/pksunkara)

[ulid]: https://github.com/ulid/spec
[pgrx]: https://github.com/tcdi/pgrx
[monotonicity]: https://github.com/ulid/spec#monotonicity

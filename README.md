<!-- omit from toc -->
# pgx_ulid

A postgres extension to support [ulid][].

1. [Why should I use this?](#why-should-i-use-this)
2. [Why should I use ulid over uuid?](#why-should-i-use-ulid-over-uuid)
3. [Monotonicity](#monotonicity)
4. [Installation](#installation)
5. [Usage](#usage)
6. [Recommendation](#recommendation)
7. [Building](#building)

## Why should I use this?

The use of GUID in a database is a trade off between performance and security. GUIDs are typically used in OLTP to get around id predictability and distributed scalability. 

There are several different postgres extensions for [ulid][], but all of them have feature gaps. A good extension should have:

- **Generator**: A generator function to generate [ulid][] identifiers. crypto secure as it uses rand::thread_rng()
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

## Why should I use ULID over UUID?

The main advantages of ULID are:

* Indexes generated using ULIDs exhibit lower fragmentation compared to UUIDs thanks to the encoded timestamp and monotonicity.
* ULID are K-ordered, which means you can used to sort the column by time order
* ULIDs don't use special characters, so they can be used in URLs or even HTML.
* ULIDs are shorter than UUIDs as they are comprised of 26 characters compared to UUIDs' 36 characters.
* ULID are more secure than UUIDv7, their randomness is 80 bits as opposed to 62 bits.
* UUID v1/v2 is impractical in many environments, as it requires access to a unique, stable MAC address
* UUID v3/v5 requires a unique seed and produces randomly distributed IDs, which can cause fragmentation in many data structures
* UUID v4 provides no other information than randomness which can cause fragmentation in many data structures

This extension is approximately **30% faster** than both `pgcrypto`'s UUID and `pg_uuidv7`'s UUIDv7 when generating a million identifiers while leveraging a crypto secure random generator.

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

### Monotonicity

Monotony ensures guarantees k-sorting order on the same postgres instance.

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
#### Pros

1. Monotonic ULIDs are better for indexing, as they are sorted by default.
2. Monotonic ULIDs slightly faster than `gen_ulid()` when generating lots of ULIDs within one millisecond. Because, in this case, there is no need to generate random component of ULID. Instead it is just incremented.

<!-- omit from toc -->
#### Cons

1. Previously generated ULID is saved in shmem and accessed via LWLock. i.e. it is exclusive for function invocation within database. Theoretically this can lead to slowdowns.

    *...But, in practice (at least in our testing) `gen_monotonic_ulid()` is slightly faster than `gen_ulid()`.*

2. Extensions that use shared memory must be loaded via `postgresql.conf`'s `shared_preload_libraries` configuration setting.

    *...But, it only affects `gen_monotonic_ulid()` function. Other functions of this extension will work normally even without this config.*

3. Monotonic ULIDs may overflow and throw an error.

    *...But, chances are negligible.*

## Installation

The extension consist of 3 files

1. **ulid--0.1.4.sql** & **ulid.control** - the extension configuration file, to deploy in SHAREDIR
2. **ulid.so** - the extension itself, to deploy in LIBDIR

edit *postgresql.conf*, add the following line:

```conf
shared_preload_libraries = 'ulid'	# (change requires restart)
```

> Note: None of these configuration are required if you use the custom docker image

## Usage

Use the extension in the database:

```sql
CREATE EXTENSION ulid;
```

Test Generation speed

```SQL
# gen
EXPLAIN ANALYSE SELECT gen_ulid() FROM generate_series(1, 1000000);
# gen and insert
EXPLAIN ANALYSE INSERT INTO ulid_keys(id) SELECT gen_ulid() FROM generate_series(1, 1000000);

# same as above but monotonic
EXPLAIN ANALYSE SELECT gen_monotonic_ulid() FROM generate_series(1, 1000000);
EXPLAIN ANALYSE INSERT INTO ulid_keys(id) SELECT gen_monotonic_ulid() FROM generate_series(1, 1000000);
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

Insert records

```SQL
INSERT INTO users values (DEFAULT, 'Olivier');
```



Operate it normally with text in queries:

```sql
SELECT * FROM users WHERE id = '01ARZ3NDEKTSV4RRFFQ69G5FAV';
```

Cast [ulid][] to timestamp:
```SQL
SELECT id::timestamp FROM users WHERE id;
```

or to uuid
```SQL
SELECT id::uuid FROM users WHERE id;
```

## Recommendation

### Do not confuse ULID's internal date with the record creation date

They are indeed quite similar at first glance, yet the dates have different connotations and, more significantly, a distinct life cycle.

**I would strongly advise against** using ulid as a create_date column for the following reasons:

* First an index is faster on a date column than on a random-date-ordered guid. thanks to its randomness.
* Shit happens - loss of data, code mistakes, migrations - you may have to change one of these dates without impacting the other.
* You may decide to create ULIDs asynchronously or in advence, therefore dissociating generation from record creation.
* In the end they are two different things: the **id's creation date** vs the **record's creation date**. Typically, in IT we get much better results when by spliting concerns.

## Building

 You may build and deploy the extension locally:  

```shell
$ cargo install cargo-pgrx --version 0.11.1 --locked
# on osx only, because we need pg_config
$ brew install postgresql
```

[pgrx][] is a friendly framework to deploy postgresql extensions in rust, to install a local dev environment, use

```shell
# if postgresql is not installed
# the following command will install and configure each version
$ cargo pgrx init
```

or use your own running instance:
```shell
# if you need to reuse a pre-installed postgresql
# make sure postgresql/bin is in the PATH
# ie. fish_add_path /opt/homebrew/opt/postgresql@16/bin
$ cargo pgrx init --pg16 (which pg_config)
```

From there, your may run the unit tests, interact with a test instance or compile the delivery package.

```shell
# run the unit tests
$ cargo pgrx test
```

```shell
# interact with a test instance
$ cargo pgrx start
$ cargo pgrx connect
```

```shell
# compile the delivery package
$ cargo pgrx install --release
$ cargo pgrx package
```

Last, buid a postgres distribution with builtin ulid support

```shell
# ensure docker is up
# to build the docker image
make
# to run it
make run
```

Further details can be found by following [this guide](https://github.com/tcdi/pgrx/blob/master/cargo-pgrx/README.md#installing-your-extension-locally).

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

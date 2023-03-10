<!-- omit from toc -->
# pg_ulid

A postgres extension to support [ulid][].

1. [Why should I use this?](#why-should-i-use-this)
2. [Why should I use ulid over uuid?](#why-should-i-use-ulid-over-uuid)

#### Why should I use this?

#### Why should I use ulid over uuid?

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

[ulid]: https://github.com/ulid/spec

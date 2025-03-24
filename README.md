# goflow2_loader
Rust implementation of a JSON to SQL from goflow2














## Tuning

Current test:
json file of 537M with 852,381 records.

Just to isolate the JSON parsing if we remove the inserts
```
2025-03-23T13:05:32.267132Z  INFO goflow_loader: Starting
2025-03-23T13:05:54.290909Z  INFO goflow_loader: Done
```
0:22 elapsed

Even storing every record in a vector
```
2025-03-23T13:15:00.399288Z  INFO goflow_loader: Starting
2025-03-23T13:15:21.978510Z  INFO goflow_loader: Done
```
0:21 elapsed

Pre-allocating vector
```
2025-03-23T20:12:22.681374Z  INFO goflow_loader: Starting
2025-03-23T20:12:44.954359Z  INFO goflow_loader: Done
```

2025-03-23T20:17:27.797369Z  INFO goflow_loader: Starting
2025-03-23T20:17:50.220115Z  INFO goflow_loader: Done

### Individual INSERTs using PgPool
4.2k transactions per second

```
2025-03-22T23:30:23.532983Z  INFO goflow_loader: Starting
2025-03-22T23:47:27.847646Z  INFO goflow_loader: Done
```
17:04 elapsed

### Individual INSERTs using PgConnection
7.2k transactions

```
2025-03-23T00:31:07.642942Z  INFO goflow_loader: Starting
2025-03-23T00:42:07.159931Z  INFO goflow_loader: Done
```
11:00 elapsed

### Individual INSERTs using PgConnection with log messages and counter

```
2025-03-23T00:58:15.191669Z  INFO goflow_loader: Starting
2025-03-23T01:10:13.657385Z  INFO goflow_loader: Done
```
11:58 elapsed

### 800k transaction

The entire file as one massive transaction (postgres is going to curse me, but somehow it works...).
```
2025-03-23T03:30:29.719320Z  INFO goflow_loader: Starting
2025-03-23T03:37:33.674590Z  INFO goflow_loader: Done

```
7:04 elapsed

### 10k transactions

Every 10k records we commit the transaction
```
2025-03-23T04:04:50.961879Z  INFO goflow_loader: Starting
2025-03-23T04:11:59.669130Z  INFO goflow_loader: Done
```
7:09 elapsed - I'm confused...

High and very consistent block I/O

### 50k transactions

Every 50k records we commit the transaction
```
2025-03-23T04:13:58.503754Z  INFO goflow_loader: Starting
2025-03-23T04:21:16.928801Z  INFO goflow_loader: Done
```
7:18 elapsed

### Bulk-insert (no transactions)

| Block size | Elapsed time |
| ---------- | ------------ |
| 1k | 0:38.00 |
| 2k | 0:37.00 |
| 3k | 0:36.04 |
| 4k | 0:35.79 |

At 5k blocks or larger the program panics with the following error:
```
Failed to insert rows: Protocol("PgConnection::run(): too many arguments for query: 80000 (sqlx_postgres::connection::executor:216)")
```
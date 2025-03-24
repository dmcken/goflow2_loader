# goflow2_loader
Rust implementation of a JSON to SQL from goflow2


**Warning: this is me learning rust so using this for production or even learning is probably a bad idea**
**Yes I know a postgres password is in the code this is a test box not exposed to the internet. As soon as I learn how to move it out to an env file I will.**







## Testing + Optimization

Current test: json file of 537M with 852,381 records.

Run against a simple postgres in a container (only properties in .env are passwords):
```
services:
  db:
    image: postgres
    restart: always
    # set shared memory limit when using docker-compose
    shm_size: 128mb
    # or set shared memory limit when deploy via swarm stack
    env_file: ".env"
    volumes:
      - ./pg_data:/var/lib/postgresql/data
    #  - type: tmpfs
    #    target: /dev/shm
    #    tmpfs:
    #      size: 134217728 # 128*2^20 bytes = 128Mb
    ports:
      - 5432:5432

  pgadmin:
    image: dpage/pgadmin4
    env_file: ".env"
    ports:
      - 16543:80
```


| Test | Sub-test | elapsed | notes | 
| ---- | -------- | ------- | ----- |
| Baseline | JSON parsing | 0:22 |  |
| Baseline | JSON Parsing + storing all records in vector | 0:21 |  |
| Baseline | JSON Parsing + storing all records in a pre-allocated vector | 0:22 |  |
| Basic | INSERTs to PgPool | 17:04 | 4.2k transactions per second |
| Basic | INSERTs to PgConnection |  11:00 | 7.2k transactions per second |
| Basic | INSERTs to PgConnection + log + counter | 11:58 |  |
| Transaction | One massive transaction | 7:04 | I am surprised that postgres accepted this in its default state |
| Transaction | 10k inserts per transaction | 7:09 | High and very consistent block I/O in PgAdmin |
| Transaction | 50k inserts per transaction | 7:18 | Clearly not helping |
| Bulk-Insert (no-transaction) | 1k blocks per insert | 0:38.00 |  |
| Bulk-Insert (no-transaction) | 2k blocks per insert | 0:37.00 |  |
| Bulk-Insert (no-transaction) | 3k blocks per insert | 0:36.04 |  |
| Bulk-Insert (no-transaction) | 4k blocks per insert | 0:35.79 |  |
| Bulk-Insert (no-transaction) | 5k blocks per insert | failed | At 5k blocks or larger the program panics with the error[^1] |


[^1]: `Failed to insert rows: Protocol("PgConnection::run(): too many arguments for query: 80000 (sqlx_postgres::connection::executor:216)")`

# goflow2_loader
Rust implementation of a JSON to SQL from goflow2














## Tuning

Current test:
json file of 537M with 852,381 records.

### Individual INSERTs using PgPool
4.1 to 4.5k transactions per second

```
2025-03-22T23:30:23.532983Z  INFO goflow_loader: Starting
2025-03-22T23:47:27.847646Z  INFO goflow_loader: Done
```

So about 17:04 elapsed

### Individual INSERTs using PgConnection

### Transaction


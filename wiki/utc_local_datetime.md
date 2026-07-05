# UTC to Local

```rust
if let Ok(data) = data.as_ref() {
    let first = data.get(0).unwrap();
    let dt = chrono::DateTime::from_timestamp_nanos(first.utc_ns);
    let local_dt: DateTime<Local> = DateTime::from(dt);
}
```

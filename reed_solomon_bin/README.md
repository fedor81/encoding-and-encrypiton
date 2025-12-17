# reed_solomon_bin

Исполняемый файл для reed_solomon

## Использование

```sh
cargo run --bin=reed_solomon_bin  -- --controls=20 encode "Hello World" --output-format=bytes

243 36 192 41 21 204 168 6 115 241 25 244 46 11 250 86 45 134 128 164 72 101 108 108 111 32 87 111 114 108 100
```

Может исправить до `control_count / 2` ошибок:

```sh
cargo run --bin=reed_solomon_bin  -- decode cc57946b60ce57fedc9f48656c6c6f20576f700000 # Пять ошибок в конце

Hello World
```

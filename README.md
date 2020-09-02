# IoT Protocol Abstraction Layer (PAL)

[![GitHub tag](https://img.shields.io/github/tag/ryankurte/rust-iot-pal.svg)](https://github.com/ryankurte/rust-iot-pal)
[![Crates.io](https://img.shields.io/crates/v/iot-pal.svg)](https://crates.io/crates/iot-pal)
[![Docs.rs](https://docs.rs/iot-pal/badge.svg)](https://docs.rs/iot-pal)


Abstractions for building IoT utilities, designed to simplify the implementation of IoT server applications and roughly unify the configuration and behaviours of these.

By default all components are built, use `default-features = false` to disable this behaviour.


Clients:

- [MQTT]() enabled with `client_mqtt`
- [CoAP]() enabled with `client_coap`

Stores:
- [ElasticSearch]() enabled with `store_elastic`


Features:

- `serde` enables serialization/deserialization on `*Options` configuration objects
- `structopt` enables `derive(StructOpt)` on `*Options` configuration objects


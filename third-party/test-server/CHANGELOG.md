# Changelog

## 0.9.0

* Replace `failure` with `anyhow`
* update dependencies

## 0.8.0

* Copy of Request is based on crate *http*
* `TestServer::new` first argument is ToSocketAddrs
* header function exports actix HeaderMap
* added helper function to read response/request bodies
* update dependencies (including majar actix versions)

## 0.7.0

* use actix_web 1.0
* update dependencies

## 0.6.0

* library maintenance
* use failure crate and return results
* update dependencies

## 0.5.6

* remove extern crates

## 0.5.5

* switch to edition "2018"
* update dependencies

## 0.5.3/0.5.4

* library maintenance
* update dependencies

## 0.5.2

* fix TestServer addr field type

## 0.5.1

* adjust imports and mod naming

## 0.5.0

* replace static mutex vector of requests by a crossbeam-channel based implementation
* adjust API and mod structure
* update dependencies
* add cargo audit test to CI

## 0.4.0

* add public [helper](https://github.com/ChriFo/test-server-rs/blob/master/src/helper.rs) mod
* update dependencies

## 0.3.0

* remove `TestServer::received_request`
* add `TestServer::requests` returning all requests in vector

## 0.2.4

* upgrade to [actix-web](https://github.com/actix/actix-web) version 0.7.2
* restructure lib
* [clippy](https://github.com/rust-lang-nursery/rust-clippy) compliance
* add one more test

## 0.2.3

* deliver request body

## 0.2.2

* adjust API
* cleanup dependencies
* add more tests

## 0.2.1

* fix visibility of received request

## 0.2.0

* reimplement server with [actix-web](https://github.com/actix/actix-web)

## 0.1.1

* fix retreiving of requests

## 0.1.0

* first version based on [iron](https://github.com/iron/iron)

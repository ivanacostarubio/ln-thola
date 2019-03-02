# THOLA

Ever wonder how many nodes you can pay some satoshis? 

A tool to know how well connected your node is. 



## Example

```shell

./lnd_thola -- %satoshis% %path_to_cert% %path_to_macaroon% %socket e.g. 127.0.0.1:100500%

./thola 100000 ./tls.cert ./readonly.macaroon 192.168.1.128:10009

```


### Install

You'll need rust to compile it from source. Otherwise, you can use the binaries.



####  Rust

```shell
curl https://sh.rustup.rs -sSf | sh

```

Make sure you also have the development packages of openssl installed.
For example, `libssl-dev` on Ubuntu or `openssl-devel` on Fedora.

You may also need to install pkg-config.



#### Compiling and Running 

```shell

cargo run 100000 ./tls.cert ./readonly.macaroon 192.168.1.128:10009

``

# `tracer`

## Usage

The best way to use the `tracer` utility locally is to compile it using the latest version of Rust. Use the [`rustup`](https://rustup.rs/) utility to install Rust locally on your machine. Once you have Rust installed you can compile and run it:

``` sh
cargo build
./target/debug/tracer -h
sudo ./target/debug/tracer trace 8.8.8.8
./target/debug/tracer export 8.8.8.8
```

To actually do a trace you probably will require root privileges (at least on macOS that is the case). Use `sudo` when calling `tracer`.

The GeoIP lookup uses https://ipapi.co as a service and the `TRACER_IPAPI_KEY` environment variable must be set. The best way to do so is to copy the example `env.example` file to `.env` and set the API key in there.

``` sh
cp env.example .env
echo "TRACER_IPAPI_KEY=SECRETKEY" >> .env
```

## CLI interface

``` sh
# Trace a single route
tracer [trace|export] <target IP address>
```

The `tracer` utility understands the following commands:

- `init`: Initialize the database. The location of the database can be set using the `-d/--db` command flag.
- `trace`: Trace a route to a target IP address.
- `export`: Export a CSV containing all hops and paths for a route.

The command can be modified using the following flags:
 
- `-c/--count`: Number of traces to the destination. Defaults to 1.
- `-n/--num-fails`: Number of failures for any hop along the way before giving up. Defaults to 10.
- `-D/--db`: Path to database file. Defaults to `./tracer.db`.

## Example

``` sh
./tracer init
sudo ./tracer trace 8.8.8.8
./tracer export 8.8.8.8 | tee 8.8.8.8.csv
```

## Example output

```
$ sudo ./target/debug/tracer trace 8.8.8.8
1: 192.168.2.1 (2ms)  192.168.2.1 (0ms)  192.168.2.1 (0ms)
2: 62.155.240.149 (5ms)  62.155.240.149 (5ms)  62.155.240.149 (5ms)
3: 217.0.203.130 (13ms)  62.154.32.58 (13ms)  62.154.32.50 (13ms)
4: 72.14.202.10 (12ms)  72.14.202.10 (13ms)  72.14.202.10 (13ms)
5: 10.252.64.126 (12ms)  10.252.194.62 (13ms)  10.252.194.62 (13ms)
6: 8.8.8.8 (11ms)  8.8.8.8 (11ms)  8.8.8.8 (12ms)
```

This is a reproduction of the Vuvuzela system for COS 518 at Princeton University.

Vuvuzela [1] is a secure messaging system that protects client metadata. The system maintains very strong privacy guarantees compared to the anonymity system we learned about in class. Even if all but one of the servers are controlled by a malicious party, Vuvuzela completely hides client data and metadata.

# Building and running
This code uses the cargo build system. To build, run
```
$ cargo build
```
This produces six binaries: `setup`, `head_server`, `intermediate_server`, `deaddrop_server`, `testclient`, and `client`, each located in the `target/debug` subdirectory.

## Setup and key distribution
The `setup` binary must be run prior to using the system. It produces private and public keys and places them in the `keys` subdirectory. To run the system, the appropriate key files must be present. In particular:
* All clients and servers need server public keys `keys/server/*.pk`
* Each client needs the private and public keys `keys/client/<client id>.*`
* Each client needs the public keys of its conversants `keys/client/<conversant id>.pk`
We do not provide a means to distribute keys to different parties, but it suffices to copy the files.

## Running the server
All three server binaries must be run. For a list of options, run
```
$ head_server -h
```

The servers must be started in reverse order, starting with the deaddrop server, then the intermediate server, and lastly the head server.

## Running the client
The `testclient` binary can be used to simulate many users and reproduce our data. It too has options
```
$ testclient -h
```
The `client` binary is still experimental, but is intended to provide command line input and output to the messaging system.
As of right now, responses are printed to stdout instead of the TUI, since getting a 100% functioning GUI was not a priority of this project.

# Tests
Unit and integration tests can be run with
```
$ cargo test
```

# References

[1] Jelle van den Hooff, David Lazar, Matei Zaharia, and Nickolai Zeldovich. Vuvuzela: Scalable private messaging resistant to traffic analysis. In Proceedings of the 25th Symposium on Operating Systems Principles, SOSP ’15, pages 137–152. ACM, 2015.

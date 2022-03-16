# NimbOS 2022

A experimental real-time operating system (RTOS) written in Rust.

🚧 Working In Progress.

## Features

* Multi-architecture support: aarch64, x86_64 (WIP)
* Preemptive scheduler
* User/kernel space isolation

## TODO

* [ ] Support x86_64
* [ ] More effective thread sleeping
* [ ] Kernel mutex/semaphore/condvar
* [ ] Run with [RVM1.5](https://github.com/rvm-rtos/RVM1.5)
* [ ] SMP

## Build & Run (in QEMU)

```sh
cd kernel
make env    # for first time
make run
```

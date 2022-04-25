# NimbOS 2022

An experimental real-time operating system (RTOS) written in Rust.

🚧 Working In Progress.

## Features

* Multi-architecture support: x86_64, aarch64, riscv64
* Preemptive scheduler
* User/kernel space isolation

## TODO

* [x] More effective thread sleeping
* [ ] Kernel mutex/semaphore/condvar
* [x] Run with [RVM1.5](https://github.com/rvm-rtos/RVM1.5)
* [ ] SMP

## Build & Run (in QEMU)

```sh
cd kernel
make env    # for first time
make run ARCH=x86_64 LOG=warn
```

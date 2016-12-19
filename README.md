# AsyncOS

[![Join the chat at https://gitter.im/AsyncOS/Lobby](https://badges.gitter.im/AsyncOS/Lobby.svg)](https://gitter.im/AsyncOS/Lobby?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

This is the repository for AsyncOS, a small hobbyist operating system which is largely unique in it's attempt at being
as asyncronous as possible - nearly all system calls are non-blocking, and communication between user processes and the
kernel is handled through event polling (as well as communication between kernel modules); this typically devolves into
a normal, blocking kernel implementation when there are few cores on the machine, but can result in significantly
improved throughput on higher-core systems.

For backwards compatability and testing purposes, AsyncOS aims to be largely POSIX compatible (though it also aims to offer
much more async-friendly primitives for process control and other operations, as some fundamental POSIX operations like
fork() and exec() can be very heavyweight and cannot be made asynchronous).

AsyncOS is developed in a mixture of assembly (as all operating systems are) and the Rust programming language.

# Attributions

The start of the project was heavily helped through Phillip Oppermann's wonderful blog, which you can find at
http://os.phil-opp.com/.

# Roadmap

- [ ] Core booting code
- [ ] Memory Management
    - [ ] Simple Page abstraction (to get code running in 64-bit mode)
    - [ ] Kernel Heap
    - [ ] Rust support for using kernel heap, allow for Box, Vec, other dynamic allocations
    - [ ] Frame Allocator
    - [ ] Pagefault Handler (Basic implementation will just map page)
- [ ] Multitasking
    - [ ] Thread abstraction, context switching
    - [ ] Blocking queues
    - [ ] Lock-free channels
    - [ ] "Actor" systems or thread groups
- [ ] Filesystem
    - [ ] ext2
    - [ ] ext3
    - [ ] FAT32
    - [ ] Partition Support
    - [ ] Filesystem systemcall abstractions
- [ ] Networking (Will be expanded later)
    - [ ] Networking systemcall abstractions
- [ ] Userland
    - [ ] Systemcall Interface (sysenter?)
    - [ ] Process abstraction
    - [ ] User systemcall queues (discussions on ensuring memory-boundedness)
    - [ ] Development of sample user programs
    - [ ] AsyncOS Shell

# Build Instructions

For the short forseeable future, until we have a chance to set up everything to use either the Rust build tool or
another, less esoteric build tool, we use Make (but absolutely no recursive make!). Building is just

```
make [build]
```

Running with qemu is

```
make run [kvm=true|false]
```

Note that KVM is on by default; as the above implies, you can disable it by `kvm=false`.
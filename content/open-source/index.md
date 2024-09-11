---
title: Open-source Contribution
summary: A summary of Zhiyao's open-source contribution.
date: "2024-09-10T00:00:00"

reading_time: false  # Show estimated reading time?
share: false  # Show social sharing links?
profile: false  # Show author profile?
comments: false  # Show comments?

# Optional header image (relative to `assets/media/` folder).
header:
  caption: ""
  image: ""
---

## Open-source Projects

### [Hopter](https://github.com/hopter-project/hopter)

Hopter is a Rust-based embedded operating system built to enable memory-safe, efficient, reliable, and responsive applications. It is co-designed with a [customized compiler](https://github.com/hopter-project/hopter-compiler-toolchain) that guarantees additional invariants beyond what Rust can express. Also, Hopter does not rely on any hardware protection mechanisms, providing safety purely through software.

Currently, Hopter supports the STM32F4 microcontroller family with Arm Cortex-M4F cores. Contributions to port Hopter to other microcontrollers are highly welcome and appreciated.

- Memory safety: Hopter prevents stack overflows on top of other memory safety aspects guaranteed by Rust.
- Memory efficiency: Hopter can allocate stacks on-demand in small chunks called stacklets, time-multiplexing the stack memory among tasks.
- Reliability: Hopter is not afraid of panic. Tasks can be spwaned as restartable tasks, which automatically restart if they panic.
- Responsiveness: Hopter supports zero-latency IRQ handling. The kernel never disables IRQs, ensuring that pending interrupts are handled immediately.

### [Hadusos](https://github.com/ZhiyaoMa98/hadusos-protocol)

Hadusos is a session protocol allowing reliable communication over serial devices. Two participants of the protocol either assumes the sender or the receiver role at a time, thus half-duplex. Hadusos has the following features:

- Simple: The simple design of the protocol leads to slim implementation code and thus small binary overhead which is especially important for storage constrained embedded devices.
- Zero-copy: No dynamic memory allocation is needed. Most received bytes goes directly into the client buffer without any copying, and all bytes to be sent goes directly to the serial devices.
- Panic-free: The protocol implementation code never panics.

Documentation: https://docs.rs/hadusos/0.2.1/hadusos/

## Other Contributions

### [stm32f4xx-hal](https://github.com/stm32-rs/stm32f4xx-hal)

**A Rust embedded-hal HAL for all MCUs in the STM32F4 family.**

Features and bug fix patches merged:
- https://github.com/stm32-rs/stm32f4xx-hal/pull/662
- https://github.com/stm32-rs/stm32f4xx-hal/pull/736
- https://github.com/stm32-rs/stm32f4xx-hal/pull/737
- https://github.com/stm32-rs/stm32f4xx-hal/pull/738
- https://github.com/stm32-rs/stm32f4xx-hal/pull/743


### [LLVM](https://github.com/llvm/llvm-project)

**A collection of modular and reusable compiler and toolchain technologies.**

Features and bug fix patches merged:
- https://github.com/llvm/llvm-project/commit/adc26b4eaedc50f1b99d5af5c7e248966fced660
- https://github.com/llvm/llvm-project/commit/bd606afe26f258d081fbe025b20a71b277c1edde
- https://github.com/llvm/llvm-project/commit/7e8af2fc0c068de8bb47d8046b8483234fab3b13
- https://github.com/llvm/llvm-project/commit/1d0ccebcd725309399262af346494242b064e2ed


### [owning-ref](https://github.com/noamtashma/owning-ref-rs)

**A library for creating references that carry their owner with them.**

Feature submitted as pull request:
- https://github.com/noamtashma/owning-ref-rs/pull/6

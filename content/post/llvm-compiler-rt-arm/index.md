---
# Documentation: https://wowchemy.com/docs/managing-content/

title: "Building LLVM Compiler-RT for ARM MCU"
subtitle: "Resolving undefined symbols to compiler intrinsic functions (`__aeabi_*`)."
summary: "We introduce compiler instrinsic functions and resolve undefined symbols to `__aeabi_*` when cross-compiling for ARM microcontrollers."
authors: []
tags: []
categories: []
date: 2022-01-25T13:30:06-05:00
lastmod: 2022-01-25T13:30:06-05:00
featured: false
draft: false

# image:
#   caption: 'Image credit: [**Unsplash**](https://unsplash.com/photos/CpkOjOcXdUY)'
#   focal_point: ""
#   placement: 2
#   preview_only: false

authors:
- Zhiyao Ma

tags:
- LLVM
- Compiler Intrinsics

categories:
- Language

# Featured image
# To use, add an image named `featured.jpg/png` to your page's folder.
# Focal points: Smart, Center, TopLeft, Top, TopRight, Left, Right, BottomLeft, Bottom, BottomRight.
image:
  caption: ""
  focal_point: ""
  preview_only: false

# Projects (optional).
#   Associate this post with one or more of your projects.
#   Simply enter your project's folder or file name without extension.
#   E.g. `projects = ["internal-project"]` references `content/project/deep-learning/index.md`.
#   Otherwise, set `projects = []`.
projects: []
---

## What are compiler intrinsic functions?

Modern compilers perform extensive optimizations during code generation. They pick the instructions that run the fastest, or are the smallest in size, while preserving the semantics of the source code written in high level languages. Surprisingly, compilers may even substitude a sequence of instructions with a function call. They assume a set of functions are at their discretion, namely [intrinsic functions or built-in functions](https://en.wikipedia.org/wiki/Intrinsic_function).

When the compilers deem profitable, they insert calls to intrinsic functions, as they presume that intrinsic functions have highly optimized and thoroughly tested implementation.

The code piece below shows a concrete example.

```c
/* Source file: temp.c */

void my_memset(void *ptr, char c, unsigned len) {
    char *cptr = (char *) ptr;
    for (; len != 0; --len)
        *cptr++ = c;
}
```

Let us compile it with optimization and disassemble the generated object file. One could choose any optimization level among `Og`, `O1`, `O2`, `O3` and `Os`. Below we show the result compiled with `clang -Os -nostdlib --target=armv7em-none-eabi -mcpu=cortex-m4 -c -o temp.o temp.c`, and disassembled by `arm-none-eabi-objdump -d temp.o`.

```text
00000000 <my_memset>:
  0:  b142        cbz r2, 14 <my_memset+0x14> # r2 holds "len"
                                              # if "len" is 0, goto 14:
  2:  b580        push {r7, lr}               # preserve registers
  4:  466f        mov r7, sp                  # redundant instruction
  6:  460b        mov r3, r1                  # shuffle arguments so to meet
  8:  4611        mov r1, r2                  # __aeabi_memset's expectation
  a:  461a        mov r2, r3                  #
  c:  f7ff fffe   bl  0 <__aeabi_memset>      # *call intrinsic function*
 10:  e8bd 4080   ldmia.w sp!, {r7, lr}       # restore registers
 14:  4770        bx  lr                      # return
```

Clang delegates the heavy work to `__aeabi_memset()`, an intrinsic function assumed to exist. Unfortunately, the assumption fails sometimes, especially when we cross-compile like above. To satisfy the assumption, we must compile the library containing all instinsic functions and link them manually.

___

## Building LLVM `compiler-rt`

Clone the LLVM project to our local machine.
```bash
git clone https://github.com/llvm/llvm-project.git
```

Create and change to a build directory.
```bash
cd llvm-project
mkdir build-compiler-rt
cd build-compiler-rt
```

Configure the build. Provide the variables according to your environment. 
```bash
cmake ../compiler-rt \
    -DCMAKE_INSTALL_PREFIX=${DIR_TO_INSTALL} \
    -DCMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY \
    -DCOMPILER_RT_OS_DIR="baremetal" \
    -DCOMPILER_RT_BUILD_BUILTINS=ON \
    -DCOMPILER_RT_BUILD_SANITIZERS=OFF \
    -DCOMPILER_RT_BUILD_XRAY=OFF \
    -DCOMPILER_RT_BUILD_LIBFUZZER=OFF \
    -DCOMPILER_RT_BUILD_PROFILE=OFF \
    -DCMAKE_C_COMPILER=${LLVM_BIN_PATH}/clang \
    -DCMAKE_C_COMPILER_TARGET="arm-none-eabi" \
    -DCMAKE_ASM_COMPILER_TARGET="arm-none-eabi" \
    -DCMAKE_AR=${LLVM_BIN_PATH}/llvm-ar \
    -DCMAKE_NM=${LLVM_BIN_PATH}/llvm-nm \
    -DCMAKE_RANLIB=${LLVM_BIN_PATH}/llvm-ranlib \
    -DCOMPILER_RT_BAREMETAL_BUILD=ON \
    -DCOMPILER_RT_DEFAULT_TARGET_ONLY=ON \
    -DLLVM_CONFIG_PATH=${LLVM_BIN_PATH}/llvm-config \
    -DCMAKE_C_FLAGS="--target=arm-none-eabi -march=armv7em" \
    -DCMAKE_ASM_FLAGS="--target=arm-none-eabi -march=armv7em"
```

Compile and install the library.
```bash
make && make install
```

If everything goes on well, we should now see the compiled library at
```bash
${DIR_TO_INSTALL}/lib/baremetal/libclang_rt.builtins-arm.a
```

Linking to it should resolve all missing definitions of `__aeabi_*`.

___

## Now undefined symbol to `memset()`?

Quoting from the [LLVM mailing list](https://lists.llvm.org/pipermail/llvm-dev/2016-April/098187.html):
> In a nutshell, Compiler-RT may assume there is a C library underneath.
> [...]
> This also works on free-standing environments (ex. the Linux kernel) because those environments assume the compiler library will do so, and thus implement "memcpy", "memset", etc.

So we just need to provide additionally the definitions of `memcpy()`, `memmove()`, `memset()`, and `memclr()`.

A prudent reader might now worry that if we write the definition of `memset()` in C, the compiler may transform our code to call `__aeabi_memset()`, which in turn calls `memset()`, thus forming a dead loop. But clang is smart enough to detect that we *are providing* the definition of `memset()` so it refrains from generating intrinsic function calls. Brilliant!

___

## What about GCC?

Up until now we have been talking about LLVM/Clang. Things in GCC are quite similar. GCC generates calls to intrinsic functions defined in `libgcc`. It also assumes the existence of `libc`. More information [here](https://gcc.gnu.org/onlinedocs/gccint/Libgcc.html).

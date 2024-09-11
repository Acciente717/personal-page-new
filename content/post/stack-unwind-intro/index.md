---
# Documentation: https://wowchemy.com/docs/managing-content/

title: "An Introduction to Stack Unwinding and Exception Handling"
subtitle: "The 2-phase unwinding procedure and compiler generated auxiliary data."
summary: "We give an overview of the stack unwinding and exception handling for C++ like languages on x86_64. We describe the use of `.eh_frame` and `.gcc_except_table` section and the 2-phase unwinding procedure."
authors: []
tags: []
categories: []
date: 2022-03-28T23:00:00
lastmod: 2022-03-28T23:00:00
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
- GCC
- Stack Unwinding

categories:
- Language
- Stack Unwinding

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

Many modern high level programming languages feature exception handling mechanism. For example, in C++ we may have a `try {...} catch (...) {...}` block, and in Rust a `catch_unwind(...)` block. The Rust's `catch_unwind` is very similar to a try-and-catch-all block in C++. Below we will call this language construct the try-catch block in general, and use the name *exception* for both the C++ exception and Rust panic, just for the sake of simplicity.

The exception introduces a "forced-return" semantic. When the code throws an exception, it examines the current active function and see if it is inside a `try` block. If no, the current active function is forced to return immediately to its caller, along with all allocated resource inside this function freed. This forced-return continues until it find itself inside a `try` block. It will then examines the `catch` list in sequence. If there is a type match, the execution will jump to the matching `catch` clause and resume back to the normal execution flow. If there isn't any matching type, the forced-return will continue. The program terminates if the `main` function is forced to return.

Below is an example written in C++ to demonstrate the notion of forced-return.

```c++
struct ExcpA {};
struct ExcpB {};

void throw_func() {
    throw ExcpA(); // -----------------| throwing, start exception
    // some code (skipped)             | propagation
}   // <-------------------------------| a forced return

void no_try_cant_catch() {
    std::vector<int> vec{0, 1};
    throw_func();               // ----| exception propagation continues
    // some code (skipped)             |
}   // <-------------------------------| a forced return (must free `vec`)

void try_but_not_catch() {
    try { no_try_cant_catch(); }  // --| exception propagation continues
    catch (ExcpB &e) {}           //   | type don't match
    // some code (skipped)        //   |
}   // <-------------------------------| a forced return

void try_and_catch() {
    try { try_but_not_catch(); } // ---| exception propagation continues
    catch (ExcpB &e) {}          //    | type don't match
    catch (ExcpA &e) {           //    | type match
        // caught here           // <--| so jump here
        // some code (executed)
    }
    // some code (executed)
}
```

Above is only a rough description of the way stack unwinding works. There are some inaccuracies, but for now let's focus on some more fundamental questions:
- (K1, C1) How to force a return when the next CPU instruction to execute is not `ret`?
- (K2, C2) What need to be freed and how to free them when we force a return?
- (K3, C3) How to decide if the code is running inside a `try` block?
- (K4, C4) How to match against the `catch` list?

To answer each question, some knowledge (K) must be encoded and some code (C) must leverage the knowledge to take actions. There are many ways to encode the knowledge and run the logic, but in this post we will focus on one particular implementation, called *zero-cost exception handling*, which put the exception handling data and logic completely outside the normal execution path.

With zero-cose exception handling, C1 is commonly the `libunwind` library. C2-4 are commonly inside the `libcxx` library. For x86_64, K1 is encoded in the `.eh_frame` section and K2-4 are in the `.gcc_except_table` section in the compiled ELF executable. For ARM, K1-4 are typically encoded in the `.ARM.extab` section, but if they are small enough, they can be inlined into the `.ARM.exidx` section.

We will start from discussing `.eh_frame` and `.gcc_except_table` on x86_64. The ARM counterpart is more complicated because it includes lots of alternative encodings as optimizations.

### Forced Return with `.eh_frame`

The exact format of `.eh_frame` can be found [here](https://refspecs.linuxfoundation.org/LSB_3.0.0/LSB-Core-generic/LSB-Core-generic/ehframechpt.html). It tells for each function how to force a return.

Conceptually, the compiler will generate several entries in the `.eh_frame` section for each function. The entry describes the [stack frame](https://en.wikipedia.org/wiki/Call_stack#STACK-FRAME) layout of the function. The stack frame layout of a function may change during its execution, so that's why we need several entries for one function. Given the current instruction pointer (IP), we can search through the `.eh_frame` section to see which function we are in, and can know the stack frame layout.

The stack frame layout conveys the following information:
- Where the preserved callee-saved register values are stored.
- Where is the return address stored.

Those values might be on the stack or might be in other registers. But as long as we know where they are, we can restore them and subsequently perform a jump to the return address. That's the forced return.

Yet another question is how we can inform the assembler about the stack frame layout when we generate the binary executable. In x86_64 assembly, this is achieved by the [CFI directives](https://sourceware.org/binutils/docs/as/CFI-directives.html). For ARM, this is done through the [ARM directives](https://sourceware.org/binutils/docs/as/ARM-Directives.html). Both forms appear as pseudo-instructions in the assembly file. These pseudo-instructions are typically generated by the compiler.

For example, the following C++ function is compiled to a sequence of instructions with interleaving CFI directives. In this example, the CFI directives convey how we can find the previous stack frame in the call stack. On x86_64, the return address is the 8 bytes stored right below the previous stack frame. The CFI directives thus help us to locate the return address.

```c++
void cfi_demo() {
    callee();
    volatile int x = 42;
}
```

```asm
_Z8cfi_demov:
    .cfi_startproc          // Mark function start
    endbr64
    subq    $24, %rsp
    .cfi_def_cfa_offset 32  // Now the previous stackframe
                            // starts from %rsp + 32
                            // Return address at %rsp + 32 - 8
    call    _Z6calleev@PLT
    movl    $42, 12(%rsp)
    addq    $24, %rsp
    .cfi_def_cfa_offset 8   // Now the previous stackframe
                            // starts from %rsp + 8
                            // Return address at %rsp + 8 - 8 == %rsp
    ret
    .cfi_endproc            // Mark function end
```

In this simple example we don't need to track preserved registers. But whenever a callee-saved register must be pushed onto the stack, the compiler will emit a `.cfi_offset register, offset` pseudo-instruction, indicating that the preserved original value of `register` can be found at `offset` bytes from the end of previous stack frame. The assembler generates the `.eh_frame` section based on these directives.

`libunwind` is a handy library to consume the `.eh_frame` section. It maintains a set of virtual registers and provides an iterator interface to iterate through the call stack. It allows us to step or iterate through the call stack. Each step is basically a forced-return by updating its internal virtual registers. `libunwind` further allows us to resume execution at any stack frame in the call stack. It achieves so by overwriting real hardware registers with the virtual registers.

I hope this rings a bell. Recall that in C++, when an exception is thrown, we should keep forcing-return until we land in a `try` block with a capable `catch`. We now know how to force the return and how to stop at a desired stack frame, but there are still a few problems waiting to be resolved:

- When we force a return, we must also free allocated resources.
- When we step into a stack frame, we must know if we are in a try block.
- When we are in a try block, we must know what types it can catch.

We will next see how `.gcc_except_table` facilitates us to resolve these problems.

### Cleanup and Catch by `.gcc_except_table`

We just saw how we can use the information stored in the `.eh_frame` section to perform a forced-return. From now on let's call the table inside `.eh_frame` the *unwinding table*, and call the rules that allow us to restore register values *unwinding rules*. Note that the unwinding rules are programming language agnostic, meaning that we use the same way to interpret the unwinding table no matter which programming language we use.

On the contrary, we need language specific knowledge to perform resource cleanup and exception catching. These pieces of information are stored in the `.gcc_except_table` section. Each entry in the unwinding table will have a pointer pointing to this language specific section. The pointer is often named LSDA, language specific data area.

The `.gcc_except_table` section contains lots of tables. Usually, each table matches a function. Given a function, we can find its matching table in `.gcc_except_table` by following the LSDA pointer in the function's unwinding table entry. From now on let's call this language specific table the *LSDA table*. The LSDA pointer essentially links together the language agnostic unwinding table and the language specific LSDA table.

Recall that `libunwind` is responsible for parsing the unwinding table entry. It delegates the job of parsing the corresponding LSDA table to a language specific function, which is called the *personality function*. Personality functions are typically defined in the runtime library of each programming language.

So the entry in the unwinding table actually stores two more pointers: an LSDA pointer and a pointer to the personality function that is capable to parse the LSDA information.

Each LSDA table stores multiple call-site entries. These entries cover distinct address range inside a function body.
```c
struct {
    ip_range_start   // The entry is relevant if IP falls into 
    ip_range_len     // the range [start, start+len)
    landing_pad_ip   // Jump to here if the action also matches
    action_num       // Describes the condition when we should
                     // jump to the landing pad
};
```

If the `action_num` is 0, it means the `landing_pad_ip` points to a cleanup routine for this function and we should always jump to it. Otherwise, the `action_num` represents the index into the action table, a sub-table within the LSDA table. In this case, the `landing_pad_ip` points to a catch block. By following the index into the action table, we can figure out what exception type the catch block is able to catch. If the throwing exception type matches it, or more precisely is a subtype of it, we should jump to the landing pad.

### 2-Phase Unwinding

Astute readers might notice that we mentioned earlier that `libunwind` can start executing code at an arbitrary address by overwriting physical registers with virtual registers, but only the personality function knows if we should jump to a landing pad. Thus, the personality function needs a way to inform `libunwind` to jump to a new address.

So yes, an interface is defined between `libunwind` and personality functions. `libunwind` calls personality functions to examine the LSDA of a stack frame. The personality function informs `libunwind` its conclusion by the return value. Very often, the personality function must set up additional context for the landing pad before instructing `libunwind` to jump to it. `libunwind` provides several callback functions at the personality function's discretion, for example `unw_get_reg()` to read a virtual register value and `unw_get_reg()` to overwrite it.

The interaction between `libunwind` and personality functions happen in two phases. The first phase is called search phase. `libunwind` walks through the call stack. For each stack frame, it calls into the corresponding personality routine to examine whether the throwing exception can be caught by the current function. `libunwind` continues until the personality routine returns "catch-able" or it reaches the bottom of the call stack. The program is usually forced to terminated with a core dump if no function in the call stack can catch the throwing exception. In C++, it is implemented as a call to `std::terminate()`. Since `libunwind` only walks through the stack with the virtual registers, every state upon exception is kept in the dumped core.

If a function capable of catching the throwing exception is identified during the first phase, the second phase, cleanup phase, starts. `libunwind` now walks through the call stack again, but this time it instructs the personality routines to prepare additional context and jumps to the landing pads during the walk to clean up resources, i.e., destruct objects. If the landing pad is a cleanup routine, at its end it will usually call `_Unwind_Resume()` that passes the control back to `libunwind`. If the landing pad is the catch block, the control flow will not return back to `libunwind`, but resume back to the user program, that is the code after the catch block.

### About ARM

The stack unwinding and exception catching on ARM share the same gist as on x86_64, but the implementation details are different. The executable ELF file stores the unwinding table and LSDA table in `.ARM.extab` section, whereas `.ARM.exidx` section is a binary search index for the functions. The unwinding table has a different format because it is architecture specific. The LSDA table has the same format as on x86_64, because it is language specific but architecture agnostic.

We will describe the implementation of a baremetal unwinder for Rust in detail in the upcoming posts.

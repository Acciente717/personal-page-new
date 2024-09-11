---
# Documentation: https://wowchemy.com/docs/managing-content/

title: "Call Rust Closure From Assembly Code"
subtitle: "Using type erasing and reconstruction to allow calling Rust closure from assembly code."
summary: "Calling a Rust closure from assembly code involves erasing the type and converting everything into raw pointers before passing to the assembly code, and then reconstructing Rust objects from the raw pointers once control returns to Rust."
authors: []
tags: []
categories: []
date: 2023-03-31T20:00:00-05:00
lastmod: 2023-04-02T20:00:00-05:00
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
- Rust

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

Although it may not be immediately apparent, there are practical reasons for calling a closure from inline assembly, particularly when it comes to implementing task spawning and context switching at the lowest level. For instance, one use case involves passing a Rust closure as the entry point of a new thread of execution. However, doing so requires calling the closure from assembly code.

Given that Rust closures lack a stably defined ABI, it is not possible to directly call them from assembly code. Additionally, passing the enclosed environment to the closure is not straightforward, as there is no stable method for doing so. As a result, we must find an indirect approach to call the Rust closure from assembly, and ensure that all of the constructs used in the process are both stable and well-defined.

To keep the code illustration clean, we will assume that we have access to types that require dynamic memory, which is provided by the `alloc` crate on `no_std` environments. However, by the end of this post, we will cover how to generalize this technique if dynamic memory is not available.

We will make the following code snippet work.

```rust
fn main() {
    // This is enclosed by the following closure.
    let enclosed = String::from("Hello");

    // This is explicitly passed as the argument to the closure.
    let passed = String::from("world");

    // Build a closure which encloses a string and also requires an argument.
    let closure = |arg| {
        println!("{} {}!", enclosed, arg);
        42
    };

    // Call the closure indirectly through some assembly code.
    let ret = call_closure_through_asm(closure, passed);

    // Examine the return value.
    println!("{}", ret);
}
```

The key to calling a Rust closure from assembly is to handle everything in the form of raw pointers. To achieve this, we must take two steps. First, before transferring control to the assembly code, we need to erase the type of every object being passed and convert them into raw pointers. Second, after control is further transferred from the assembly to a Rust function, we must reconstruct Rust objects from the raw pointers.

Let's now focus on reconstructing objects from raw pointers. The following function reconstructs the closure and its argument from a raw pointer and then calls the closure with the argument. Once the closure returns, it converts the return value of the closure into a raw pointer and returns the pointer. The `extern "C"` keyword is necessary because the trampoline function will be called from assembly code, thus it must have a stable ABI.

```rust
extern "C" fn trampoline<F, A, R>(ptr_closure: *mut F, ptr_arg: *mut A) -> *mut R
where
    F: FnOnce(A) -> R,
{
    // Reconstruct the closure, the argument, and return value type from the raw pointer.
    let closure = unsafe { Box::from_raw(ptr_closure) };
    let arg = unsafe { Box::from_raw(ptr_arg) };

    // Call the closure, get the return value.
    let ret = (*closure)(*arg);

    // Convert the return value to a raw pointer.
    Box::into_raw(Box::new(ret))
}
```

The above `trampoline` function will be called by an assembly function. Here we use Rust inline assembly for illustration purpose, but it can also be written in raw assembly. The example assembly code is provided for MacOS/AArch64 and Linux/x86_64. While this code example does not do anything particularly interesting other than calling the closure, in real-world scenarios, the assembly code may be used to perform other important tasks, for example, bootstraping the environment of a newly spawned thread.

```rust
#[naked]
extern "C" fn asm_call(ptr_closure: usize, ptr_arg: usize, trampoline: usize) -> usize {
    unsafe {
        #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
        asm!(
            // Preserve return address by pushing into the stack
            // and align stack pointer.
            "str x30, [sp, #-16]!",
            // Call the trampoline function.
            // By the calling convention, `trampoline` is in x2 register,
            // while the argument to `trampoline`, i.e.,  `ptr_closure`
            // and `ptr_arg`, are already in x0 and x1 register.
            "blr x2",
            // Restore return address by popping from the stack.
            "ldr x30, [sp], #16",
            // Return.
            "ret",
            // No automatically generated return instruction.
            // Required by #[naked] attribute.
            options(noreturn)
        );
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        asm!(
            // Align stack pointer.
            "sub  rsp, $8",
            // Call the trampoline function.
            // By the calling convention, `trampoline` is in rdx register,
            // while the argument to `trampoline`, i.e.,  `ptr_closure`
            // and `ptr_arg`, are already in rdi and rsi register.
            "call rdx",
            // Align stack pointer.
            "add  rsp, $8",
            // Return.
            "ret",
            // No automatically generated return instruction.
            // Required by #[naked] attribute.
            options(noreturn)
        );
    }
}
```

Before calling into the assembly function above, we need to reduce the objects into a raw pointers. The following code demonstrates how we can accomplish this.

```rust
fn call_closure_through_asm<F, A, R>(closure: F, arg: A) -> R
where
    F: FnOnce(A) -> R,
{
    // Convert the closure and argument into raw pointers.
    let ptr_closure = Box::into_raw(Box::new(closure));
    let ptr_arg = Box::into_raw(Box::new(arg));

    // Monomorphize the trampoline function, so that the monomorphized version will
    // know the object type after it receives the raw pointer.
    let trampo = trampoline::<F, A, R>;

    // Call the assembly function. Get back the return value as another pointer.
    let ptr_ret = asm_call(ptr_closure as usize, ptr_arg as usize, trampo as usize) as *mut R;

    // Reconstruct the return value from the returned pointer.
    let boxed_ret = unsafe { Box::from_raw(ptr_ret) };
    *boxed_ret
}
```

Finally, even when dynamic memory is not available, the technique of erasing the type and reconstructing later can still be used. For instance, if we are spawning a new thread, we can still move the objects to the bottom of the stack of the new thread. We can then acquire the pointers to the moved object and pass them to the trampoline function.

Full code listing [here](./main.rs).

#### Afterwords

Actually we can apply tail call optimization to the example assembly code above, making it even more compact:
```rust
#[naked]
extern "C" fn asm_call(ptr_closure: usize, ptr_arg: usize, trampoline: usize) -> usize {
    unsafe {
        #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
        asm!(
            "br x2",
            options(noreturn)
        );
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        asm!(
            "jmp rdx",
            options(noreturn)
        );
    }
}
```

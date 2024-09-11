#![feature(naked_functions)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use core::arch::asm;

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

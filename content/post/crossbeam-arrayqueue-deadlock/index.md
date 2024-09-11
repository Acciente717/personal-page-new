---
# Documentation: https://wowchemy.com/docs/managing-content/

title: "Rust Crate Crossbeam's ArrayQueue Can Deadlock"
subtitle: "Showing two scenarios where `ArrayQueue` can deadlock."
summary: "Although `ArrayQueue` contains no explicit spin lock or mutex, its code structure forms a big spin lock and thus deadlock is possible."
authors: []
tags: []
categories: []
date: 2023-08-06T22:00:00-05:00
lastmod: 2023-08-06T22:00:00-05:00
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
- Deadlock
- Crossbeam

categories:
- Algorithm

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

`Crossbeam` is a popular Rust synchronization library. The provided `ArrayQueue` type is

> A bounded multi-producer multi-consumer queue.

Despite numerous webpages suggesting that `ArrayQueue` is lock-free, it is important to clarify that it is NOT. In the following, we will describe two scenarios that can result in deadlocks when using the ArrayQueue. Essentially, while `ArrayQueue` does not explicitly contain any mutex or spin lock, its loop structure and atomic instructions in its methods effectively form a spin lock.

<br/>

## Definitions of Obstruction/Lock/Wait Freedom

The informal concept of lock-freedom can be formally classified into three levels: obstruction-free, lock-free, and wait-free, with comprehensive definitions available on the [Wikipedia page](https://en.wikipedia.org/wiki/Non-blocking_algorithm). Among these, obstruction freedom is the weakest. It essentially asserts that if we suspend all threads operating on a data structure at any given time, but leaving one thread running, that thread will eventually return from the method, instead of blocking or endlessly looping inside it.

To demonstrate that deadlock can indeed occur with `ArrayQueue`, we will present two scenarios that violate obstruction freedom, causing the one running thread to loop indefinitely when other threads are suspended.

It is worth noting that Dmitry Vyukov, the inventor of the algorithm, [explicitly stated](http://www.1024cores.net/home/lock-free-algorithms/queues/bounded-mpmc-queue):

> The algorithm is pretty simple and fast. *It's not lockfree in the official meaning*, just implemented by means of atomic RMW operations w/o mutexes.

Thus, rumors have distorted the true properties of the algorithm.

<br/>

## [`crossbeam::queue::ArrayQueue`](https://docs.rs/crossbeam-queue/0.3.8/src/crossbeam_queue/array_queue.rs.html) Implementation

The MPMC queue is supported by an array, a contiguous chunk of memory. The array element has the `Slot<T>` type.

```rust
struct Slot<T> {
    /// The current stamp.
    ///
    /// If the stamp equals the tail, this node will be next written to. If it equals head + 1,
    /// this node will be next read from.
    stamp: AtomicUsize,

    /// The value in this slot.
    value: UnsafeCell<MaybeUninit<T>>,
}
```

Notably, each slot is associated with a `stamp` value to solve the [ABA problem](https://en.wikipedia.org/wiki/ABA_problem).

The queue constains indices to the head and tail element, with the lap count encoded into `AtomicUsize` type. Storing `cap` and `one_lap` is just for performance optimization, both of which can be calculated from `buffer.len()`.

```rust
pub struct ArrayQueue<T> {
    /// The head of the queue.
    ///
    /// This value is a "stamp" consisting of an index into the buffer and a lap, but packed into a
    /// single `usize`. The lower bits represent the index, while the upper bits represent the lap.
    ///
    /// Elements are popped from the head of the queue.
    head: CachePadded<AtomicUsize>,

    /// The tail of the queue.
    ///
    /// This value is a "stamp" consisting of an index into the buffer and a lap, but packed into a
    /// single `usize`. The lower bits represent the index, while the upper bits represent the lap.
    ///
    /// Elements are pushed into the tail of the queue.
    tail: CachePadded<AtomicUsize>,

    /// The buffer holding slots.
    buffer: Box<[Slot<T>]>,

    /// The queue capacity.
    cap: usize,

    /// A stamp with the value of `{ lap: 1, index: 0 }`.
    one_lap: usize,
}
```

<br/>

## Deadlock Case #1

The first scenario where `ArrayQueue` can deadlock is when one thread is running `.push()` while another thread is concurrently running `.pop()`. Specifically, deadlock may arise in the following sequence of events: The queue is initially empty, a thread calls the `.push()` method and is suspended midway, then another thread calls the `.pop()` method.

The implementation appears somewhat obscure as the library wants to reuse the code for both the `.push()` and `.force_push()` methods by factoring out the common part into `.push_or_else()`. Essentially, `.push_or_else()` first attempts to push an element into the queue, but if the queue happens to be full, it will invoke the provided closure to perform additional actions.

The following code shows how `.push()` is implemented. We comment on the lines which will lead to deadlock if the thread is suspended there when the queue is initially empty.

```rust
pub fn push(&self, value: T) -> Result<(), T> {
    self.push_or_else(value, |v, tail, _, _| {
        let head = self.head.load(Ordering::Relaxed);

        // If the head lags one lap behind the tail as well...
        if head.wrapping_add(self.one_lap) == tail {
            // ...then the queue is full.
            Err(v)
        } else {
            Ok(v)
        }
    })
}

fn push_or_else<F>(&self, mut value: T, f: F) -> Result<(), T>
where
    F: Fn(T, usize, usize, &Slot<T>) -> Result<T, T>,
{
    let backoff = Backoff::new();
    let mut tail = self.tail.load(Ordering::Relaxed);

    loop {
        // Deconstruct the tail.
        let index = tail & (self.one_lap - 1);
        let lap = tail & !(self.one_lap - 1);

        let new_tail = if index + 1 < self.cap {
            // Same lap, incremented index.
            // Set to `{ lap: lap, index: index + 1 }`.
            tail + 1
        } else {
            // One lap forward, index wraps around to zero.
            // Set to `{ lap: lap.wrapping_add(1), index: 0 }`.
            lap.wrapping_add(self.one_lap)
        };

        // Inspect the corresponding slot.
        debug_assert!(index < self.buffer.len());
        let slot = unsafe { self.buffer.get_unchecked(index) };
        let stamp = slot.stamp.load(Ordering::Acquire);

        // If the tail and the stamp match, we may attempt to push.
        if tail == stamp {
            // Try moving the tail.
            match self.tail.compare_exchange_weak(
                tail,
                new_tail,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    // ****** begin DEADLOCK IF SUSPENDED *****

                    // Write the value into the slot and update the stamp.
                    unsafe {
                        slot.value.get().write(MaybeUninit::new(value));
                    }

                    // The `tail` has been updated, but the `stamp` in the
                    // `slot` has not been updated.

                    // ****** end DEADLOCK IF SUSPENDED *****

                    slot.stamp.store(tail + 1, Ordering::Release);
                    return Ok(());
                }
                Err(t) => {
                    tail = t;
                    backoff.spin();
                }
            }
        } else if stamp.wrapping_add(self.one_lap) == tail + 1 {
            atomic::fence(Ordering::SeqCst);
            value = f(value, tail, new_tail, slot)?;
            backoff.spin();
            tail = self.tail.load(Ordering::Relaxed);
        } else {
            // Snooze because we need to wait for the stamp to get updated.
            backoff.snooze();
            tail = self.tail.load(Ordering::Relaxed);
        }
    }
}
```

Suppose that the thread running `.push()` is suspended during the marked region above, another thread running `.pop()` will spin in the method. In the following code, the first `if` condition `head + 1 == stamp` will yield `false`, because the `stamp` has not been updated. The subsequent `else if` on `stamp == head` will yield `true`. However, since `tail` has been updated, it no longer equals to `head`, thus `return None` will not be executed, but rather the thread will backoff for a while and loop back to try again. The thread can never return, thus deadlock. Essentially, the `.pop()` thread is spinning to wait for the `.push()` thread to finish updating the `stamp`.

```rust
pub fn pop(&self) -> Option<T> {
    let backoff = Backoff::new();
    let mut head = self.head.load(Ordering::Relaxed);

    loop {
        // Deconstruct the head.
        let index = head & (self.one_lap - 1);
        let lap = head & !(self.one_lap - 1);

        // Inspect the corresponding slot.
        debug_assert!(index < self.buffer.len());
        let slot = unsafe { self.buffer.get_unchecked(index) };
        let stamp = slot.stamp.load(Ordering::Acquire);

        // If the the stamp is ahead of the head by 1, we may attempt to pop.
        if head + 1 == stamp {
            let new = if index + 1 < self.cap {
                // Same lap, incremented index.
                // Set to `{ lap: lap, index: index + 1 }`.
                head + 1
            } else {
                // One lap forward, index wraps around to zero.
                // Set to `{ lap: lap.wrapping_add(1), index: 0 }`.
                lap.wrapping_add(self.one_lap)
            };

            // Try moving the head.
            match self.head.compare_exchange_weak(
                head,
                new,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    // Read the value from the slot and update the stamp.
                    let msg = unsafe { slot.value.get().read().assume_init() };
                    slot.stamp
                        .store(head.wrapping_add(self.one_lap), Ordering::Release);
                    return Some(msg);
                }
                Err(h) => {
                    head = h;
                    backoff.spin();
                }
            }
        } else if stamp == head {
            atomic::fence(Ordering::SeqCst);
            let tail = self.tail.load(Ordering::Relaxed);

            // If the tail equals the head, that means the channel is empty.
            if tail == head {
                return None;
            }

            backoff.spin();
            head = self.head.load(Ordering::Relaxed);
        } else {
            // Snooze because we need to wait for the stamp to get updated.
            backoff.snooze();
            head = self.head.load(Ordering::Relaxed);
        }
    }
}
```

<br/>

## Deadlock Case #2

The other scenario where `ArrayQueue` can deadlock is when one thread is running `.pop()` while another thread is concurrently running `.force_push()`. Specifically, deadlock may arise in the following sequence of events: the queue is initially full, a thread calls the `.pop()` method and is suspended midway, then another thread calls the `.force_push()` method.

The cause of deadlock is similar to the previous one. Suppose that a thread running `.pop()` successfully executes the compare and exchange operation, but is suspended before updating the `stamp` of the popped slot. Subsequently, the other thread running `.force_push()`, as shown below, will spin forever in the method. This is because the `if tail == stamp` condition in `.push_or_else()` fails, since the `stamp` has not been updated, but the following `else if` condition will succeed. The provided closure is then invoked, seeing that the `head` has been updated, thus will return with `Ok(v)`, so inside `.push_or_else()` it will loop over again and retry. Deadlock.

```rust
pub fn force_push(&self, value: T) -> Option<T> {
    self.push_or_else(value, |v, tail, new_tail, slot| {
        let head = tail.wrapping_sub(self.one_lap);
        let new_head = new_tail.wrapping_sub(self.one_lap);

        // Try moving the head.
        if self
            .head
            .compare_exchange_weak(head, new_head, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
        {
            // Move the tail.
            self.tail.store(new_tail, Ordering::SeqCst);

            // Swap the previous value.
            let old = unsafe { slot.value.get().replace(MaybeUninit::new(v)).assume_init() };

            // Update the stamp.
            slot.stamp.store(tail + 1, Ordering::Release);

            Err(old)
        } else {
            Ok(v)
        }
    })
    .err()
}
```

<br/>

## Deadlock with Real Application


The two scenarios discussed above sound contrived. Is there any real world code that deadlocks? YES.

As I initially believed that `ArrayQueue` was lock-free, and thus, I used it for synchronization between a task and an interrupt handler in my embedded system. The interrupt handler provides data into the queue, while the task consumes it. The handler calls the `.force_push()` method, discarding the oldest element if the queue is already full. However, since the microcontroller I am using is single-core, a deadlock occurs in the following situation: When the queue is full and the task is popping an element, an interrupt is triggered, invoking the handler, which subsequently calls `.force_push()`. This is exactly the deadlock case #2.

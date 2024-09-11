---
# Documentation: https://wowchemy.com/docs/managing-content/

title: "Changing Rust Enum Variant with Mutable Reference"
subtitle: "Discussing two approaches to modify a Rust enum through a mutable reference."
summary: "Changing a Rust enum variant through a mutable reference can be achieved either by wrapping the variant attached variable inside `Option`, or better, by introducing an `Undef` variant in the enum."
authors: []
tags: []
categories: []
date: 2024-02-14T00:00:00-05:00
lastmod: 2024-02-14T00:00:00-05:00
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

Consider the following `Coin` type that has two variants `Front` and `Back`, each attached with a variable of type `T`.

```rust
enum Coin<T> {
    Front(T),
    Back(T),
}
```

Given a `mut` reference to `Coin`, how can we change it from `Front` to `Back` or vice versa? Let us begin with a naive attempt to implement `turn_to_front`, which changes `Back` to `Front` if it is not already `Front`.

```rust
impl<T> Coin<T> {
    fn turn_to_front(&mut self) {
        match self {
            Self::Front(_) => return,
            Self::Back(val) => {
                *self = Self::Front(*val);
            }
        }
    }
}
```

The compiler, however, produces an error when we try to compile the code above:

```
error[E0507]: cannot move out of `*val` which is behind a mutable reference
```

More specifically, the naive code above attempts to create an invalid state of `Coin` through the `*val` expression. Let us zoom in the non-trivial code branch

```rust
Self::Back(val) => {
    *self = Self::Front(*val);
}
```

The single-line statement is equivalent to the following two-line statements. If the code compiles, then the `Coin` variable referenced by `self` would be left in an invalid state for some time. Specifically, the invalidity comes from the fact that `self` continues to be the `Back` variant, yet the variable attached to `Back` is no longer valid because of the move.

```rust
let tmp = *val;
// If the above statement compiles, then `self` will be invalid until the
// next statement is executed.
*self = Self::Front(tmp);
```

A [reddit post](https://www.reddit.com/r/rust/comments/238599/changing_the_variant_of_an_enum_accessible_via_mut/) attempts to circumvent the compiler's check with `unsafe` code, however, as other replies have pointed out, it is impossible to do it safely.

A straightforward correct solution is to wrap `T` with `Option`.

```rust
enum Coin<T> {
    Front(Option<T>),
    Back(Option<T>),
}

impl<T> Coin<T> {
    fn turn_to_front(&mut self) {
        match self {
            Self::Front(_) => return,
            Self::Back(opt) => {
                let val = opt.take();
                *self = Self::Front(val);
            }
        }
    }
}
```

However, wrapping `T` with `Option` might be suboptimal because every access to `T` must go through an `.unwrap()` of the `Option`, degrading the runtime performance. Also, the size of the resulting `Coin` type can be larger if niche optimization is not applicable to `Option<T>`.

An alternative better approach is based on the following insight: We should convey an explicit variant to the compiler, which represents the state when `enum Coin` is not owning the `T` during the transition from `Back` to `Front`.

```rust
enum Coin<T> {
    Front(T),
    Back(T),
    Undef,
}

impl<T> Coin<T> {
    fn turn_to_front(&mut self) {
        match self {
            Self::Front(_) => return,
            Self::Back(_) => {
                let mut tmp = Self::Undef;

                // Make `self` to be `Undef`.
                // `tmp` holds the previous value.
                core::mem::swap(self, &mut tmp);

                // `tmp` must be the `Back` variant.
                if let Self::Back(val) = tmp {
                    *self = Self::Front(val);
                }
            }
            Self::Undef => panic!(),
        }
    }
}
```

This approach avoids wrapping `T`, nor does it increase the size of `Coin`, since the `Front` or `Back` variant has larger size than `Undef`.

The idea has also been discovered on the [Rust forum](https://users.rust-lang.org/t/replace-enum-variant-inside-match/95937).

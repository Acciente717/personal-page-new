---
# Documentation: https://wowchemy.com/docs/managing-content/

title: "Symbols in Compiled Object File and Their Attributes"
subtitle: "The correspondence between C constructs and the symbols they generated."
summary: "We show by example the generated symbols of C constructs and their attributes in each case. We also provide a Rust program for examining the symbols in an object file."
authors: []
tags: []
categories: []
date: 2022-12-06T22:00:00-05:00
lastmod: 2022-12-06T22:00:00-05:00
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
- Compiler
- Object File

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

## Symbols in Object Files

When high level language code is compiled to a binary relocatable file, or object file for short, the file will also include a symbol table so that functions and variables with global visibility can be identified.

Each symbol has some attributes, e.g., whether it is global, which section it belongs to, its size, etc.

The following annotated C code shows the correspondence between C constructs and the emitted symbols. The C source file is named as `test.c` and compiled with `clang -O2 -c test.c`. The Rust code used for extracting the symbols is shown in the next section.

```c
/**
 * File name is saved as a symbol.
 * 
 * symbol: test.c, is_global: false, is_local: true, is_common: false
 * is_definition: false, is_undefined: false, is_weak: false
 * kind: File, section: "None"
 * size: 0
 */

/**
 * Uninitialized global variable goes to Common section.
 * Note that is_definition() is still true.
 * 
 * symbol: global_int_common, is_global: true, is_local: false, is_common: true
 * is_definition: true, is_undefined: false, is_weak: false
 * kind: Data, section: "Common"
 * size: 4
 */
int global_int_common;

/**
 * Zero initialized global variable goes to .bss section.
 * 
 * symbol: global_int_bss, is_global: true, is_local: false, is_common: false
 * is_definition: true, is_undefined: false, is_weak: false
 * kind: Data, section: ".bss"
 * size: 4
 */
int global_int_bss = 0;

/**
 * Non-zero initialized global variable goes to .data section.
 * 
 * symbol: global_int_data, is_global: true, is_local: false, is_common: false
 * is_definition: true, is_undefined: false, is_weak: false
 * kind: Data, section: ".data"
 * size: 4
 */
int global_int_data = 1;

/**
 * The attribute makes the symbol weak.
 * 
 * symbol: global_int_weak_bss, is_global: true, is_local: false, is_common: false
 * is_definition: true, is_undefined: false, is_weak: true
 * kind: Data, section: ".bss"
 * size: 4
 */
int __attribute__((weak)) global_int_weak_bss = 0;

/**
 * The attribute makes the symbol weak.
 * 
 * symbol: global_int_weak_data, is_global: true, is_local: false, is_common: false
 * is_definition: true, is_undefined: false, is_weak: true
 * kind: Data, section: ".data"
 * size: 4
 */
int __attribute__((weak)) global_int_weak_data = 1;

/**
 * Extern variable goes to Undefined section.
 * Note that their size is 0 despite having int type.
 * 
 * symbol: extern_int, is_global: true, is_local: false, is_common: false
 * is_definition: false, is_undefined: true, is_weak: false
 * kind: Unknown, section: "Undefined"
 * size: 0
 */
extern int extern_int;

/**
 * Static variable with local visibility (file or function level) never goes to Common.
 * Note that is_definition() is still true.
 * 
 * symbol: static_int_not_common, is_global: false, is_local: true, is_common: false
 * is_definition: true, is_undefined: false, is_weak: false
 * kind: Data, section: ".bss"
 * size: 4
 */
static int static_int_not_common;

/**
 * Zero-initialized static variable goes to .bss section.
 * 
 * symbol: static_int_bss, is_global: false, is_local: true, is_common: false
 * is_definition: true, is_undefined: false, is_weak: false
 * kind: Data, section: ".bss"
 * size: 4
 */
static int static_int_bss = 0;

/**
 * Non-zero initialized static variable goes to .data section.
 * 
 * symbol: static_int_data, is_global: false, is_local: true, is_common: false
 * is_definition: true, is_undefined: false, is_weak: false
 * kind: Data, section: ".data"
 * size: 4
 */
static int static_int_data = 1;


/**
 * Referenced but undefined function goes to Undefined section.
 * 
 * symbol: global_func_ref, is_global: true, is_local: false, is_common: false
 * is_definition: false, is_undefined: true, is_weak: false
 * kind: Unknown, section: "Undefined"
 * size: 0
 */
void global_func_ref(int *);

/**
 * The extern keyword does not make any difference when function is not defined.
 * 
 * symbol: extern_func_ref, is_global: true, is_local: false, is_common: false
 * is_definition: false, is_undefined: true, is_weak: false
 * kind: Unknown, section: "Undefined"
 * size: 0
 */
extern void extern_func_ref(int *);

/**
 * Static function is local instead of global.
 * 
 * symbol: static_func_def, is_global: false, is_local: true, is_common: false
 * is_definition: true, is_undefined: false, is_weak: false
 * kind: Text, section: ".text"
 * size: 7
 */
static void __attribute__((noinline)) static_func_def(void) {
    global_func_ref(0);
}

/**
 * Normal function definition is global.
 * 
 * symbol: global_func_def, is_global: true, is_local: false, is_common: false
 * is_definition: true, is_undefined: false, is_weak: false
 * kind: Text, section: ".text"
 * size: 37
 */
void global_func_def(int x) {
    // Use static variables and functions to prevent them from being optimized out.
    global_func_ref(&static_int_not_common);
    global_func_ref(&static_int_bss);
    extern_func_ref(&static_int_data);
    global_func_ref(&extern_int);
    static_func_def();
}

/**
 * Function definition can also be made weak.
 * 
 * symbol: global_func_weak_def, is_global: true, is_local: false, is_common: false
 * is_definition: true, is_undefined: false, is_weak: true
 * kind: Text, section: ".text"
 * size: 1
 */
void __attribute__((weak)) global_func_weak_def(void) {}

```


## Rust Code for Extracting Symbols and Attributes

The following code depends on the `object` crate. The example code here works with version `0.30.0`. Provide the path to an ELF file as the argument to the program.

```rust
use object::{self, Object, ObjectSection, ObjectSymbol};
use std::{env, fs};

fn main() {
    let path = env::args().skip(1).next().unwrap();
    let raw_bytes = fs::read(path).unwrap();
    let obj = object::File::parse(&*raw_bytes).unwrap();

    for sym in obj.symbols() {
        let name = sym.name().unwrap();

        let sec_name = match sym.section() {
            object::SymbolSection::Unknown => "Unknown".to_string(),
            object::SymbolSection::None => "None".to_string(),
            object::SymbolSection::Undefined => "Undefined".to_string(),
            object::SymbolSection::Absolute => "Absolute".to_string(),
            object::SymbolSection::Common => "Common".to_string(),
            object::SymbolSection::Section(idx) => obj
                .section_by_index(idx)
                .unwrap()
                .name()
                .unwrap()
                .to_string(),
            _ => panic!("Unknown symbol section type"),
        };

        println!(
            "symbol: {}, is_global: {}, is_local: {}, is_common: {}",
            name,
            sym.is_global(),
            sym.is_local(),
            sym.is_common()
        );
        println!(
            "is_definition: {}, is_undefined: {}, is_weak: {}",
            sym.is_definition(),
            sym.is_undefined(),
            sym.is_weak()
        );
        println!("kind: {:?}, section: {:?}", sym.kind(), sec_name);
        println!("size: {}", sym.size());
        println!();
    }
}
```

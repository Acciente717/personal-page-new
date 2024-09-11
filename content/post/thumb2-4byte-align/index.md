---
# Documentation: https://wowchemy.com/docs/managing-content/

title: "Thumb2 Instructions Require 4-Byte Alignment Rather Than 2-Byte"
subtitle: "Why 2-byte alignment is not sufficient for thumb2 instructions."
summary: "We discuss why 2-byte alignment is not sufficient for thumb2 instructions even though the ISA supports an arbitrary mixture of 2- and 4-byte instructions."
authors: []
tags: []
categories: []
date: 2022-08-15T15:00:00
lastmod: 2022-08-15T15:00:00
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
- Embedded
- ARM

categories:
- Embedded

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

[Thumb2](https://en.wikipedia.org/wiki/ARM_architecture_family#Thumb-2) instruction set is comprised of half-word (2-byte) instructions and one-word (4-byte) instructions. Half-word and one-word instructions can be arbitrarily mixed. There is *not* a separate execution mode for half-word or one-word instructions. The CPU just runs in the thumb mode and can handle the two instruction widths in the instruction decoding frontend.

Importantly, one-word instructions may be placed at an address of a multiple of 2-byte, rather than requiring it to be 4-byte aligned. As an example, see the `str.w` instruction below. Although it is a 4-byte instruction, it can be placed at address range $[2, 6)$.

```
00000000 <foo>:
   0:	6010      	str	r0, [r2, #0]
   2:	f8c2 8004 	str.w	r8, [r2, #4]
   6:	6091      	str	r1, [r2, #8]
```

Thus, it is natual to extrapolate that when we dynamically load a code section into the address space, we need only guarantee 2-byte alignment. Unfortunately, it turns out to be incorrect.

The problem occurs with PC relative loading instructions. According to [ARMv7-M Architecture Reference Manual](https://developer.arm.com/documentation/ddi0403/latest) section A5.1.2, the interpretation of reading the PC register, a.k.a. `R15`, takes into account the instruction's address:

> The use of 0b1111 as a register specifier [...] When a value of 0b1111 is permitted, a variety of meanings is possible. For register reads, these meanings are: [...] Read the word-aligned PC value, that is, the address of the current instruction + 4, with bits [1:0] forced to zero. The base register of `LDC`, `LDR`, `LDRB`, `LDRD`, `LDRH`, `LDRSB`, and `LDRSH` instructions can be the word-aligned PC.

In other words, when the PC relative loading instruction is placed at a 4-byte aligned address, the loading address is calculated as
```
load_addr = instruction_addr + 4 + relative_offset
```

However, if it is placed at a 2-byte aligned address but not 4-byte aligned, the calculation will be
```
load_addr = instruction_addr - 2 + 4 + relative_offset
```

In conclusion, we must compile and load the code section with 4-byte alignment to avoid breaking PC relative loading instructions.

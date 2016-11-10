---
layout: post
title: About fix-rs
permalink: /about/
date: 2016-11-10 16:43:40
---

This is the start of a series of blog posts following the development of fix-rs. fix-rs is a FIX engine library written in Rust. FIX, which stands for *Finanical Information Exchange*, is a messaging protocol used by the financial industry primarily for automating communication of security transactions. Rust is a fast and safe systems programming language used to fill many of the same roles as C++.

As a FIX engine, fix-rs aims to make it simple and efficient to work with FIX messages. It will handle all message management and network communication so you can focus on business logic. At the same time, users of the library should be able to integrate with an existing infrastructure by picking and choosing only the parts that are needed.

With the strict safety and performance sensitive requirements in the financial industry, Rust seems like a natural fit. It's a very fast programming language. It provides powerful compile time checking to protect against things like segfaults. It's built on the LLVM backend so it can share many of the same optimizations enjoyed by C++.

But Rust is still a relatively new language and it isn't always obvious what is the best way to approach a problem. A big part of this blog will be investigating different approaches to solving a problem and then reviewing the trade offs being made. In the end, writing these posts should improve fix-rs. If it helps other people with their own projects, that's even better.

About the Author
----------------

James Bendig is a programmer who is experienced in working on performance demanding products with C++ ranging from Voice Over IP (VOIP) to image processing. He most recently worked on [Phoduit](https://phoduit.com), a professional node-based photo editor.

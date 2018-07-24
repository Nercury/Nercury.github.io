---
layout: post
title:  "Rust and OpenGL from scratch - Procedural Macros"
date:   2018-07-11
categories: rust opengl tutorial
---

Welcome back!

[Previously](/rust/opengl/tutorial/2018/06/27/opengl-in-rust-from-scratch-09-vertex-attribute-format.html), 
we have written this repetitive impl for our `Vertex` type:

(some code omitted)

```rust
#[repr(C, packed)]
struct Vertex {
    pos: data::f32_f32_f32,
    clr: data::f32_f32_f32,
}

impl Vertex {
    fn vertex_attrib_pointers(gl: &gl::Gl) {
        let stride = std::mem::size_of::<Self>(); // byte offset between consecutive attributes

        // pos

        let location = 0; // layout (location = 0)
        let offset = 0; // offset of the first component

        unsafe {
            data::f32_f32_f32::vertex_attrib_pointer(gl, stride, location, offset);
        }
        
        // clr

        let location = 1; // layout (location = 1)
        let offset = offset + std::mem::size_of::<data::f32_f32_f32>(); // offset of the first component

        unsafe {
            data::f32_f32_f32::vertex_attrib_pointer(gl, stride, location, offset);
        }
    }
}
```

This time, we will auto-generate this `impl Vertex` code using 
[procedural macros](https://doc.rust-lang.org/book/second-edition/appendix-04-macros.html#procedural-macros-for-custom-derive).

## What are the Procedural Macros?

Procedural macros allow us to generate code at compile time. You've used them before:
for example, the `failure` crate we've tried recently uses procedural macros
to automatically implement `Fail` trait when we write `#[derive(Fail)]` above the struct.

A procedural macro applied to a struct can receive the tokens for struct from the compiler,
and output any amount of code for this struct (in form of tokens) that will be then inlined bellow
the struct and type-checked as usual.

Procedural macro can not remove or add fields on a struct, it can only append a new code.
Which is perfectly fine for our use case.

## The Goal

The goal is to replace the code above with some attributes on our type:

(src/main.rs, example)

```rust
#[derive(VertexAttribPointers)]
#[repr(C, packed)]
struct Vertex {
    #[location = "0"]
    pos: data::f32_f32_f32,
    #[location = "1"]
    clr: data::f32_f32_f32,
}
```

## The simplest possible Implementation

We can start by auto-generating `impl Vertex` code as-is, to test that 
this set up can work at all. We will simply hardcode the procedural macro output for now.

There is one caveat: procedural macro will need to live in it's own crate. However, as
strange as it may seem, this crate would not need to depend on `render_gl` crate
to generate code for `render_gl::data` structs: all it is going to care of are tokenized 
code input and output.

The longer-term plan in our examples would be to move `render_gl` and `resources` modules
into their own crates. However, I want to keep the code for each part separately
downloadable, and I am reluctant to move it into a separate crate shared by all lessons
until it is clear that we won't need to change it. However, for procedural macros, 
we _have_ to have a separate crate, and it is clear that it _will_ need to change, so I
have to keep the code inside the lesson's directory.

However, in your own project, you may do as you wish.

Let's create `render_gl_derive` crate. The `_derive` suffix is a convention for macro
crates that implement `derive`:

![open gl derive](/images/opengl-rust/10/lesson-10-derive.gif)

It would sit nicely there alongside `render_gl` and `resources` crates, but for
now we will leave `render_gl` and `resources` inside the `src`. Again, I do this to 
simplify the example code.

Now, we can set up a minimal procedural macro crate.

Inside `Cargo.toml`, we add the usual stuff, as well as dependencies on
`quote` and `syn` crates. We are following here [the official 
tutorial for procedural macros](https://doc.rust-lang.org/book/second-edition/appendix-04-macros.html#procedural-macros-for-custom-derive).

(render_gl_derive/Cargo.toml, full file contents)

```toml
[package]
name = "lesson_10_render_gl_derive"
version = "0.1.0"
authors = []

[dependencies]
quote = "0.3.15"
syn = "0.11.11"

[lib]
proc-macro = true
```

Two notes here:

- We need to mark crate with `proc-macro = true` flag at the end.
- I've prefixed the crate name with `lesson_10_`, because I am going to copy this
code into the next lesson, and because the lessons are living in the same parent
[cargo workspace](https://doc.rust-lang.org/book/second-edition/ch14-03-cargo-workspaces.html),
I will be using these prefixes to give different names to the same crates in different lessons.

(render_gl_derive/src/lib.rs, full file contents)

```rust
#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;

#[proc_macro_derive(VertexAttribPointers, attributes())]
pub fn vertex_attrib_pointers_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = generate_impl(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}

fn generate_impl(ast: &syn::DeriveInput) -> quote::Tokens {
    quote! {
        impl Vertex {
            fn vertex_attrib_pointers(gl: &gl::Gl) {
                let stride = std::mem::size_of::<Self>(); // byte offset between consecutive attributes

                let location = 0; // layout (location = 0)
                let offset = 0; // offset of the first component

                unsafe {
                    data::f32_f32_f32::vertex_attrib_pointer(gl, stride, location, offset);
                }

                let location = 1; // layout (location = 1)
                let offset = offset + std::mem::size_of::<data::f32_f32_f32>(); // offset of the first component

                unsafe {
                    data::f32_f32_f32::vertex_attrib_pointer(gl, stride, location, offset);
                }
            }
        }
    }
}
```

Let's go over this code bit by bit.

The `#![recursion_limit="128"]` is needed because we will be using some amazing macros from
`quote` crate.

```rust
extern crate proc_macro;
```

The `proc_macro` crate is a compiler hack: it does not exist, except inside
a crate marked with `proc-macro`, and we suddenly get access to compiler tokens.

```rust
extern crate syn;
#[macro_use] extern crate quote;
```

These crates will be used to work with macro input and output in a convenient way.

```rust
#[proc_macro_derive(VertexAttribPointers, attributes())]
pub fn vertex_attrib_pointers_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ...
}
```

This line "registers" our derive implementation. We may choose any name we want,
I've picked `VertexAttribPointers`. Our function will be called `vertex_attrib_pointers_derive`.

```rust
pub fn vertex_attrib_pointers_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = generate_impl(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}
```

Inside this helper function, we take tokens from the compiler (they are a string),
then parse them with `syn` crate into AST (abstract syntax tree, which will allow
us to inspect the code in a convenient way). Then, we run our function to generate 
the implementation code, and then return the string back to the compiler.

Let's got back to the next part, the `generate_impl` function:

```rust
fn generate_impl(ast: &syn::DeriveInput) -> quote::Tokens {
    quote! {
        impl Vertex {
            fn vertex_attrib_pointers(gl: &gl::Gl) {
                let stride = std::mem::size_of::<Self>(); // byte offset between consecutive attributes

                let location = 0; // layout (location = 0)
                let offset = 0; // offset of the first component

                unsafe {
                    data::f32_f32_f32::vertex_attrib_pointer(gl, stride, location, offset);
                }

                let location = 1; // layout (location = 1)
                let offset = offset + std::mem::size_of::<data::f32_f32_f32>(); // offset of the first component

                unsafe {
                    data::f32_f32_f32::vertex_attrib_pointer(gl, stride, location, offset);
                }
            }
        }
    }
}
```

Now, this is amazing. The input for this function is a `ast: &syn::DeriveInput` type,
which can now be inspected to find all the fields on the struct.

The return result of this function is generated with `quote` macro (more on it soon). So,
everything you see between `quote! {` and `}` _is a macro_, which is converted to
`quote::Tokens` type by `quote!`.

Inside the `quote!`, we've copy-pasted our `impl Vertex` code as-is.

Now, let's make sure this code is used from our main function.

### Using our Procedural Macro

We can reference our new crate inside our main `Cargo.toml`:

(Cargo.toml, add the dependency to the `[dependencies]` section)

```toml
lesson_10_render_gl_derive = { path = "render_gl_derive" }
```

And use it inside the `main.rs`:

(src/main.rs, add `extern crate` line)

```rust
#[macro_use] extern crate lesson_10_render_gl_derive as render_gl_derive;
```

(again, I have to do the `lesson_10_` prefix dance that you may skip)

Now, we can delete `impl Vertex` code, and the crate should fail to compile:

```plain
84 |         Vertex::vertex_attrib_pointers(&gl);
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ function or associated item not found in `Vertex`
```

But then, if we add `#[derive(VertexAttribPointers)]` attribute to `Vertex` struct, it should:

(src/main.rs, modify `struct Vertex`)

```rust
#[derive(VertexAttribPointers)]
#[repr(C, packed)]
struct Vertex {
    pos: data::f32_f32_f32,
    clr: data::f32_f32_f32,
}
```

Now, we just need to replace the hardcoded implementation with another one that
generates different code dependant on the number of fields, their types and attributes.
Let's continue!

## Generating the impl based on the Struct Definition

We can "debug" the procedural macro code by panicking. Panic happens at compile-time
and is included in error as a help message. For example, we can panic and 
check the contents of
`ast: &syn::DeriveInput`:

(render_gl_derive/src/lib.rs, inside fn generate_impl, example)

```rust
panic!("ast = {:#?}", ast);
```

When we try to compile it, we get this error message:

```rust
error: proc-macro derive panicked
  --> lesson-10\src\main.rs:13:10
   |
13 | #[derive(VertexAttribPointers)]
   |          ^^^^^^^^^^^^^^^^^^^^
   |
   = help: message: ast = DeriveInput {
               ident: Ident(
                   "Vertex"
               ),
               vis: Inherited,
               attrs: [
                   Attribute {
                       style: Outer,
                       value: List(
                           Ident(
                               "repr"
                           ),
                           [
                               MetaItem(
                                   Word(
                                       Ident(
                                           "C"
                                       )
                                   )
                               ),
                               MetaItem(
                                   Word(
                                       Ident(
                                           "packed"
                                       )
                                   )
                               )
                           ]
                       ),
                       is_sugared_doc: false
                   }
               ],
               generics: Generics {
                   lifetimes: [],
                   ty_params: [],
                   where_clause: WhereClause {
                       predicates: []
                   }
               },
               body: Struct(
                   Struct(
                       [
                           Field {
                               ident: Some(
                                   Ident(
                                       "pos"
                                   )
                               ),
                               vis: Inherited,
                               attrs: [],
                               ty: Path(
                                   None,
                                   Path {
                                       global: false,
                                       segments: [
                                           PathSegment {
                                               ident: Ident(
                                                   "data"
                                               ),
                                               parameters: AngleBracketed(
                                                   AngleBracketedParameterData {
                                                       lifetimes: [],
                                                       types: [],
                                                       bindings: []
                                                   }
                                               )
                                           },
                                           PathSegment {
                                               ident: Ident(
                                                   "f32_f32_f32"
                                               ),
                                               parameters: AngleBracketed(
                                                   AngleBracketedParameterData {
                                                       lifetimes: [],
                                                       types: [],
                                                       bindings: []
                                                   }
                                               )
                                           }
                                       ]
                                   }
                               )
                           },
                           Field {
                               ident: Some(
                                   Ident(
                                       "clr"
                                   )
                               ),
                               vis: Inherited,
                               attrs: [],
                               ty: Path(
                                   None,
                                   Path {
                                       global: false,
                                       segments: [
                                           PathSegment {
                                               ident: Ident(
                                                   "data"
                                               ),
                                               parameters: AngleBracketed(
                                                   AngleBracketedParameterData {
                                                       lifetimes: [],
                                                       types: [],
                                                       bindings: []
                                                   }
                                               )
                                           },
                                           PathSegment {
                                               ident: Ident(
                                                   "f32_f32_f32"
                                               ),
                                               parameters: AngleBracketed(
                                                   AngleBracketedParameterData {
                                                       lifetimes: [],
                                                       types: [],
                                                       bindings: []
                                                   }
                                               )
                                           }
                                       ]
                                   }
                               )
                           }
                       ]
                   )
               )
           }
```

As you can see, in the `ast` we have all the information we need about the struct, like its name,
its fields, the field types, generic parameters, attributes, lifetimes, and so on.

We can now simply loop over this struct's fields and build our impl code. For that, we will invoke
`quote` separately for each field and its type, and then combine the results in a single
piece of code blob. This documentation might come in handy:

- [The main doc page of the `quote` crate](https://docs.rs/quote/0.6.3/quote/);
- [The documentation of the `quote` macro](https://docs.rs/quote/0.6.3/quote/macro.quote.html);
- [The announcement video of quote macro](https://air.mozilla.org/procedural-macros-in-rust/) (video note: _syntax extensions_ no longer exist).
    <iframe src="https://air.mozilla.org/procedural-macros-in-rust/video/" width="640" height="380" frameborder="0" allowfullscreen></iframe>

To check if we are generating the right thing, we will panic inside `generate_impl` function.
We start by generating `impl Vertex {}` code:

(render_gl_derive/src/lib.rs, inside `generate_impl` function)

```rust
let ident = &ast.ident;
let generics = &ast.generics;
let where_clause = &ast.generics.where_clause;

panic!("code = {:#?}", quote!{
    impl #ident #generics #where_clause {
        // continue here
    }
});
```

This produces "help" error message:

```rust
13 | #[derive(VertexAttribPointers)]
   |          ^^^^^^^^^^^^^^^^^^^^
   |
   = help: message: code = Tokens(
               "impl Vertex { }"
           )
```

To be thorough, we have also included the `generics` tokens, to generate the correct code
if our vertex had any generics (not that we plan any at this point). To see it in effect,
we can test it out with these examples:

(src/main.rs, temporary example change to `Vertex`)

```rust
struct Vertex<A, B> {
    pos: A,
    clr: B,
}
```

produces:

```rust
= help: message: code = Tokens(
           "impl Vertex < A , B > { }"
       )
```

As well as

```rust
struct Vertex<A, B: Display> where A: Clone {
    pos: A,
    clr: B,
}
```

produces:

```rust
= help: message: code = Tokens(
           "impl Vertex < A , B : Display > where A : Clone { }"
       )
```

Of course, the generics and `where` clauses might be completely unnecessary for our use case, but
I have added them for the completeness sake.

Inside the `impl` block, we can now add the function implementation. The stride calculation
will always be the same, so we can add it too. It uses `Self` keyword which refers to
containing struct, so we don't need to use the `#ident` here (but we could):

(render_gl_derive/src/lib.rs, inside `generate_impl` function, continued)

```rust
    #[allow(unused_variables)]
    pub fn vertex_attrib_pointers(gl: &gl::Gl) {
        let stride = std::mem::size_of::<Self>(); // byte offset between consecutive attributes
        let offset = 0;
        
        // continue here
    }
```

Next, we will need some repeated macro code for each field. We will generate
tokens for each field in a new `generate_vertex_attrib_pointer_calls` function 
(which will return a Vec of them), 
and then assign the return value to `fields_vertex_attrib_pointer`:

(render_gl_derive/src/lib.rs, above `panic!("code = {:#?}", quote!{`)

```rust
let fields_vertex_attrib_pointer 
    = generate_vertex_attrib_pointer_calls(&ast.body);
```

By consulting the `quote` macro docs, we can find how to include anything that
implements `IntoIterator<Item=ToTokens>` trait (`Vec<quote::Tokens>` would) as
repeatable code contents:

(render_gl_derive/src/lib.rs, continued)

```rust
#(#fields_vertex_attrib_pointer)*
```

Now, we just need to write `generate_vertex_attrib_pointer_calls` function
following the same technique (by the "same technique" I mean inspecting types with `panic` calls
until we arrive at something reasonable):

(render_gl_derive/src/lib.rs, new function)

```rust
fn generate_vertex_attrib_pointer_calls(body: &syn::Body) -> Vec<quote::Tokens> {
    match body {
        &syn::Body::Enum(_) 
            => panic!("VertexAttribPointers can not be implemented for enums"),
        &syn::Body::Struct(syn::VariantData::Unit) 
            =>  panic!("VertexAttribPointers can not be implemented for Unit structs"),
        &syn::Body::Struct(syn::VariantData::Tuple(_)) 
            =>  panic!("VertexAttribPointers can not be implemented for Tuple structs"),
        &syn::Body::Struct(syn::VariantData::Struct(ref s)) => {
            s.iter()
                .map(generate_struct_field_vertex_attrib_pointer_call)
                .collect()
        },
    }
}
```

Here, we decide _not_ to handle enums, unit structs (with no size), 
tuple structs (with unpredictable order), just plain structs please.

For that, we will write the actual token generation in `generate_struct_field_vertex_attrib_pointer_call`
call:

(render_gl_derive/src/lib.rs, new function)

```rust
fn generate_struct_field_vertex_attrib_pointer_call(field: &syn::Field) -> quote::Tokens {
    panic!("field = {:#?}", field)
}
```

Here we can continue our "debugging technique" by panicking to check how our field
data looks like:

```rust
   = help: message: field = Field {
               ident: Some(
                   Ident(
                       "pos"
                   )
               ),
               vis: Inherited,
               attrs: [],
               ty: Path(
                   None,
                   Path {
                       global: false,
                       segments: [
                           PathSegment {
                               ident: Ident(
                                   "data"
                               ),
                               parameters: AngleBracketed(
                                   AngleBracketedParameterData {
                                       lifetimes: [],
                                       types: [],
                                       bindings: []
                                   }
                               )
                           },
                           PathSegment {
                               ident: Ident(
                                   "f32_f32_f32"
                               ),
                               parameters: AngleBracketed(
                                   AngleBracketedParameterData {
                                       lifetimes: [],
                                       types: [],
                                       bindings: []
                                   }
                               )
                           }
                       ]
                   }
               )
           }
```

The code we plan to generate is this (reordered a bit so that each field is generated
in exactly the same way):

(example)

```rust
let stride = std::mem::size_of::<Self>(); // byte offset between consecutive attributes
let offset = 0; // offset of the first component

let location = 0; // layout (location = 0)
unsafe {
    data::f32_f32_f32::vertex_attrib_pointer(gl, stride, location, offset);
}
let offset = offset + std::mem::size_of::<data::f32_f32_f32>(); // offset of the second component

let location = 1; // layout (location = 1)
unsafe {
    data::f32_f32_f32::vertex_attrib_pointer(gl, stride, location, offset);
}
let offset = offset + std::mem::size_of::<data::f32_f32_f32>(); // offset of the third component
```

The last line is unnecessary, however we can rely on the compiler to optimize this dead 
code away. 

But we have one bit of information missing: location.
Of course, we could simply assume that the `location` always starts at `0` and is
incremented by `1` for each field, however this would not be a very flexible design
(much worse it can be prone to bugs when adding fields in the middle).

Instead, it would be best if we could specify the location for each field over
a custom attribute, like `#[location = 0]`, `#[location = 1]` and so on.

It is actually really simple to do: in our `proc_macro_derive`, we simply list `location`
as a possible attribute for our procedural macro:

(render_gl_derive/src/lib.rs, change `#[proc_macro_derive(VertexAttribPointers, attributes())]`)

```rust
#[proc_macro_derive(VertexAttribPointers, attributes(location))]
```

And now this attribute is legal to add to our `Vertex` struct:

(src/main.rs, modify `struct Vertex`)

```rust
#[derive(VertexAttribPointers)]
#[repr(C, packed)]
struct Vertex {
    #[location = 0]
    pos: data::f32_f32_f32,
    #[location = 1]
    clr: data::f32_f32_f32,
}
```

Now when we run our panicking implementation again, we can find the
attr value inside `attrs`:

```rust
   = help: message: field = Field {
               ...
               attrs: [
                   Attribute {
                       style: Outer,
                       value: NameValue(
                           Ident(
                               "location"
                           ),
                           Int(
                               0,
                               Unsuffixed
                           )
                       ),
                       is_sugared_doc: false
                   }
               ],
               ...
           }
```

In `generate_struct_field_vertex_attrib_pointer_call`, we can start by requiring `location`
attribute in `field.attrs`, and printing a reasonable message if it is missing:

(render_gl_derive/src/lib.rs, `generate_struct_field_vertex_attrib_pointer_call`, full)

```rust
fn generate_struct_field_vertex_attrib_pointer_call(field: &syn::Field) -> quote::Tokens {
    let field_name = match field.ident {
        Some(ref i) => format!("{}", i),
        None => String::from(""),
    };
    let location_attr = field.attrs
        .iter()
        .filter(|a| a.value.name() == "location")
        .next()
        .unwrap_or_else(|| panic!(
            "Field {:?} is missing #[location = ?] attribute", field_name
        ));
        
    // continue here
}
```

Then, we can extract the integer value literal from it:

(render_gl_derive/src/lib.rs, continued)

```rust
let location_value_literal = match location_attr.value {
    syn::MetaItem::NameValue(_, ref literal @ syn::Lit::Int(_, _)) => literal,
    _ => panic!("Field {} location attribute value must be an integer literal", field_name)
};

// continue here
```

Here we match `syn::Lit::Int(_, _)`, but take `Lit` token itself out with a `literal @`subpattern, 
because it will be directly usable from the `quote` macro.

Speaking of which:

(render_gl_derive/src/lib.rs, continued)

```rust
let field_ty = &field.ty;
quote! {
    let location = #location_value_literal;
    unsafe {
        #field_ty::vertex_attrib_pointer(gl, stride, location, offset);
    }
    let offset = offset + std::mem::size_of::<#field_ty>();
}
```

All that's left is removing panic code `panic!("code = {:#?}" ...)` that surrounds the `quote` macro
from `generate_impl` function, and we should be good to go!

However, if we try to compile it, we receive this surprising error:

```rust
error[E0658]: non-string literals in attributes, or string literals in top-level positions, are experimental (see issue #34981)
  --> lesson-10\src\main.rs:16:5
   |
16 |     #[location = 0]
   |     ^^^^^^^^^^^^^^^
```

Yep, it's a bit sad, but we will have to use `#[location = "0"]` [until this feature is stabilized](https://github.com/rust-lang/rust/issues/34981).
It is easy to rewrite the code a bit to require string literal on our attribute:

(render_gl_derive/src/lib.rs, full `generate_struct_field_vertex_attrib_pointer_call` function)

```rust
fn generate_struct_field_vertex_attrib_pointer_call(field: &syn::Field) -> quote::Tokens {
    let field_name = match field.ident {
        Some(ref i) => format!("{}", i),
        None => String::from(""),
    };
    let location_attr = field.attrs
        .iter()
        .filter(|a| a.value.name() == "location")
        .next()
        .unwrap_or_else(|| panic!(
            "Field {} is missing #[location = ?] attribute", field_name
        ));

    let location_value: usize = match location_attr.value {
        syn::MetaItem::NameValue(_, syn::Lit::Str(ref s, _)) => s.parse()
            .unwrap_or_else(
                |_| panic!("Field {} location attribute value must contain an integer", field_name)
            ),
        _ => panic!("Field {} location attribute value must be a string literal", field_name)
    };

    let field_ty = &field.ty;
    quote! {
        let location = #location_value;
        unsafe {
            #field_ty::vertex_attrib_pointer(gl, stride, location, offset);
        }
        let offset = offset + std::mem::size_of::<#field_ty>();
    }
}
```

And modify our `Vertex` type:

```rust
#[derive(VertexAttribPointers)]
#[repr(C, packed)]
struct Vertex {
    #[location = "0"]
    pos: data::f32_f32_f32,
    #[location = "1"]
    clr: data::f32_f32_f32,
}
```

With that, our code should finally compile.

## Referencing the crates by Full Path

If we tried to move our `Vertex` type from the crate root (main.rs file)
to some submodule, we will find that the generated implementation no longer sees module paths
such as `std` or `gl`. We need to change them to be absolute.

We nee to change `std::mem` to `::std::mem` and `gl::Gl` to `::gl::Gl`.

## Discussion

The final procedural macro implementation is much shorter than this blog post:

(render_gl_derive/src/lib.rs, full)

```rust
#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;

#[proc_macro_derive(VertexAttribPointers, attributes(location))]
pub fn vertex_attrib_pointers_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = generate_impl(&ast);
    gen.parse().unwrap()
}

fn generate_impl(ast: &syn::DeriveInput) -> quote::Tokens {
    let ident = &ast.ident;
    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;
    let fields_vertex_attrib_pointer = generate_vertex_attrib_pointer_calls(&ast.body);

    quote!{
        impl #ident #generics #where_clause {
            #[allow(unused_variables)]
            pub fn vertex_attrib_pointers(gl: &::gl::Gl) {
                let stride = ::std::mem::size_of::<Self>();
                let offset = 0;

                #(#fields_vertex_attrib_pointer)*
            }
        }
    }
}

fn generate_vertex_attrib_pointer_calls(body: &syn::Body) -> Vec<quote::Tokens> {
    match body {
        &syn::Body::Enum(_) => panic!("VertexAttribPointers can not be implemented for enums"),
        &syn::Body::Struct(syn::VariantData::Unit) =>  panic!("VertexAttribPointers can not be implemented for Unit structs"),
        &syn::Body::Struct(syn::VariantData::Tuple(_)) =>  panic!("VertexAttribPointers can not be implemented for Tuple structs"),
        &syn::Body::Struct(syn::VariantData::Struct(ref s)) => {
            s.iter().map(generate_struct_field_vertex_attrib_pointer_call).collect()
        },
    }
}

fn generate_struct_field_vertex_attrib_pointer_call(field: &syn::Field) -> quote::Tokens {
    let field_name = match field.ident {
        Some(ref i) => format!("{}", i),
        None => String::from(""),
    };
    let location_attr = field.attrs
        .iter()
        .filter(|a| a.value.name() == "location")
        .next()
        .unwrap_or_else(|| panic!(
            "Field {} is missing #[location = ?] attribute", field_name
        ));

    let location_value: usize = match location_attr.value {
        syn::MetaItem::NameValue(_, syn::Lit::Str(ref s, _)) => s.parse()
            .unwrap_or_else(
                |_| panic!("Field {} location attribute value must contain an integer", field_name)
            ),
        _ => panic!("Field {} location attribute value must be a string literal", field_name)
    };

    let field_ty = &field.ty;
    quote! {
        let location = #location_value;
        unsafe {
            #field_ty::vertex_attrib_pointer(gl, stride, location, offset);
        }
        let offset = offset + ::std::mem::size_of::<#field_ty>();
    }
}
```

However it requires a bit learning to get there the first time.

But having done this once, we can see how procedural macro can be
quite powerful for certain use cases. From now on, extending the code to support
additional features such as padding between the fields is quite trivial (hint: make a 
data type for padding).

We also saw an interesting property of procedural macro: it has no idea if
function `vertex_attrib_pointer` exists on a type, it simply generates the code.
The code, of course, would fail to compile if there was no such `vertex_attrib_pointer`
function implemented.

Should we continue using procedural macros? Well, it was possible to avoid them, with a bit more
boilerplate. But it is good to know they exist! Yet another tool in our toolbox with its own
trade-offs.

Next time, [we will implement the remaining 65 OpenGL data types](/rust/opengl/tutorial/2018/07/12/opengl-in-rust-from-scratch-11-vertex-data-types.html).

[Full source code is available on github](https://github.com/Nercury/rust-and-opengl-lessons/tree/master/lesson-10).
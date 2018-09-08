---
layout: post
title:  "Mutable Shared God Object and a little Flyweight"
date:   2018-09-01 11:00:00 UTC
categories: rust flyweight pattern
---

Let's talk about a Rust pattern that puts a flyweight facade on a 
reference-counted shared mutable object.

Some designs we want to make do not map well into idiomatic Rust code.
For example, the design of a channel that transmits data between
the sending and receiving ends requires a hidden state that ties
them together. Or we may want to create many different mesh objects 
for our game, and update mesh positions all around the codebase,
without touching the list used by the loop that renders them sequentially 
and efficiently.

Enter the Mutable Shared God Object and the little Flyweight pattern.

## The God Object

Let's say we have this Lines Renderer that renders lines fast.
We can submit lines in groups, and each group can have a translation
transformation applied to it:

```rust
let mut r = LinesRenderer::new();

// group data
let line_coordinates = vec![line1, line2, ...];
let group_transformation = (0.0, 0.0, 0.0);

// create this group
r.create_group(line_coordinates, group_transformation);
```

The `LinesRenderer` transforms this group data into a more efficient
representation, places it inside a buffer - does everything to make 
the render call to be as efficient as possible:

```rust
r.render();
```

This is our God Object. It does the bulk of work, and does it efficiently.

## The Mutability Problem

We use `LinesRenderer` to render lines for debugging. That means our groups
change often, and may appear/disappear. We render lots of lines, so we want 
the performance to be reasonable.

Say, we want to update a group we have added previously to `LinesRenderer`. 
We can't just re-create the whole lines list and add our updated group, 
because we don't know who else has added other lines.

What we can do however, is to assign a handle for each group, and use it
to issue update call:

```rust
let mut r = LinesRenderer::new();

// group data
let line_coordinates = vec![line1, line2, ...];
let group_transformation = (0.0, 0.0, 0.0);

// create this group
let handle: usize = r.create_group(line_coordinates, group_transformation);

// later...

r.update_group_transformation(handle, (1.0, 1.0, 1.0));
```

We can let the `update_group_transformation` to do the most efficient
thing possible and reshuffle the internal data to reflect this update.

The similar approach can be used to remove the line group from the
renderer:

```rust
r.remove_group(handle);
```

This neatly solves the mutability issue, and the handle is a very lightweight
thing to store or move around.

## The Sharing Problem

To actually use the handle, we need a mutable reference to the
`LinesRenderer`. One approach would be to pass this reference to every function that
needs it. This would be a preferred method if we did'nt have many functions.

There are other issues.

The `usize` we use to identify line group is not tied in any way to original
`LinesRenderer`. If we had multiple different renderers 
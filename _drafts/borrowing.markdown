---
layout: post
title:  "Explore the borrowing system in Rust"
date:   2015-01-20
categories: rust guide
---

### References DRAFT AND IDEAS

When we create a reference, the value can't be moved.

Explain why.

References poison struct stacks. Investigate how deeply.
(Workarounds? Example?)

Reference must not reach out of stack of any "poisoned" object related to it.
(Example?)

Imagine you drive a car. This is a thing you own. Then someone asks
to borrow the wheels from you. You are not moving anywhere until you get
your wheels back! Getting your wheels stolen is a compile-time error.
So it is always safe to lend your wheels in Rust!

We can move values into better positions on stack to make them live longer
than references. (Example? Relations)

References are temporary. From the point of view of value they
reference.

Pitfalls of putting references in structs. Then using them from the
member functions.

It may initially feel like reference is your usual object that you can
pass around. Not really - it must always get back to its originating
owner - otherwise you are screwed. (Screwiness Examples?)

Maybe pattern: move all values in position, do the work (by borrowing
as neccesary), produce new values as artefacts of work, repeat.

Rust makes you think more about the data.

About object oriented programming maybe? How to make it work in Rust.

The idea that you can model "real world" in the code is fundamentally
flawed. You are not modelling "car", "engine", "wheels", because it
is not the car you are looking at - it is a piece of friggin code!
Model in code will always be different from the abstraction that is being
modeled. (Needs convincing examples?).

If you model a car and add method "update()", what does that mean? Well,
precisely nothing. What is it going to drive on? Well, you need road.
Except that road is also not real, and now you have to shoehorn your
perfect car model onto the "road" which is not real one at all.

What happens when you call "update()". Behind the scenes it would
change something. Probably the car would move. So, there is a mutable thing
behind the scenes. So the "update()" would have what we call "side effect".

Where do we keep this "behind the scenes" thing? What is going to move?
Well, it real world, a lot of thing would change position in complicated ways,
but since this is just piece of code, we would probably have a single position
on a road for whole car. __So the car now knows about roads it drives
on__.

But there is this weird thing that was taught to us: we would treat the
car as "black box" of some sorts, and calling "update()" would change
the internal representation ONLY. But when we are writing it FOR REAL,
our next task is to DRAW THE CAR ON SCREEN.

For that to happen we probably would add "draw()" call on it. But wait,
now the car must __know about our graphics pipeline__.

But maybe we also need to save / load our game. Would that mean our
car have a __save__ and __load__? So it would know about format used for
serialising it to the file?

Now about inheritance: suppose our car inherits __Object__ entity, that
has all the "update", "draw", "save", "load" calls. We would just need to
implement them correctly for our car. So the parent object would have
everything that a child could ever have.

I am gonna skip discussion how horribly inefficient this is. I might now
mention how bad it is that when saving our game we have to go over all
the objects just to know if they can be saved. Same with drawing, or updating.

It IS going to be messy, because we are trying to go against the flow of
the application. Let's see what is going on:

- For each updatable thing, update
- For each drawable stuff draw
- For each saveable thing, save

Car itself should come from pieces of these:

    graphics <- drawable piece -> car <- updatable piece -> physics

But really, does "drawable piece" need to even know about the car? The same
about "updatable piece".

Let's instead think of situation when they do. __For example, when physics
updates position, we have to draw car in different place__.

Um. So. What if we do stupid thing and follow that sentence precisely:

    graphics <- drawable piece <- when updatable position changes update drawable -> updatable piece -> physics

But wait! So inefficient! So many pieces! Well. Think about it: each
piece does precisely what it is concerned about.

Graphics does graphics. Drawable piece is concerned how to correctly draw
the car in graphics pipeline. Physics probably uses some kind of
physics engine that has it's own quirks, so updatable piece
is properly separated from the rest of our code - we have concrete place for these
quirks in our codebase.

The drawable piece can be added to the draw list, while updatable
piece can be in update list. You know waht they say, that the fastest code
is the one you DONT run?

Well, imagine if we do not want to update something if it is too far away:
too far away FROM THE PLAYER. This adds another dimension to our
objects, which would have made class hierarchy even more impossible to maintain.
Instead, we could add a feature for that:

    updatable player <- update only if player is in radius -> updatable piece

How this would look like?

... no longer about borrowing i guess..

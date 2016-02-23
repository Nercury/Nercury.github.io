---
layout: post
title:  "Automatic event cleanup in C++"
date:   2016-02-22
categories: C++ interesting
---

Event subscriptions have a downside: someone has to unsubscribe. Usual approach
is to make sure it happens when the subscibtion is no longer needed.

But, can we do it automatically, in a way that is easy to use and extend?

Let's see how it can be done in C++.

## The Problem

The event dispatching is nothing too complicated. It requires some kind of
list to store the event listeners, and two methods: to subscribe to event and
trigger the event.

Here, we want to make sure this event list is cleaned up when a subscription
is gone.

But that presents a problem: the subscriber would need to keep
reference to the subscription list, so it can remove itself, this way creating
a circular dependency.

Luckily, we can use `weak_ptr` to break the cycle, but how exactly?

## The Mechanism

The idea is to keep only the `weak_ptr` in event dispatcher,
and do the cleanup on event invocation.

This way the subscriber does not need to know about the event list beyond the
initial subscription, and event list does not need to know anything
about a subscriber except if it is not null.

It would also be great to separate this mechanism from business code, so it
can be easily reused. We can do that using C++ template for dispatcher.

## The Optimistic Dispatcher Implementation

Let's go over the code.

```cpp
template <typename C> class OptimisticDispatcher final {
private:
    std::vector<std::weak_ptr<C>> callbacks;
```

Here, we defined a wrapper for callback list, where `C` is anything callable
(although it will usually be a `std::function`).

```cpp
public:
    std::shared_ptr<C> add(C && callback) {
        auto shared = std::make_shared<C>(callback);
        this->callbacks.push_back(shared);
        return shared;
    }
```

Here, we define a public method to add a callback to the list.
I have chosen to require a plain callable as an argument, and then give back
the `std::shared_ptr<C>` for caller to keep alive while it needs to receive
the events.

The `shared_ptr` is converted to `weak_ptr` automatically on `push_back` call.

```cpp
public:
    template <typename ...A>
    void invoke(A && ... args) {
        // Go over all callbacks and dispatch on those that are still available.
        // Remove all callbacks that are gone.
        typename std::vector<std::weak_ptr<C>>::iterator iter;
        for (iter = this->callbacks.begin(); iter != this->callbacks.end(); ) {
            auto callback = iter->lock();
            if (callback) {
                (*callback)(std::forward<A>(args)...);
                ++iter;
            } else {
                iter = this->callbacks.erase(iter);
            }
        }
    }
```

Here, the most interesting part: invoking the event.

In this method we either invoke the callback, or clean it up.
Also we can forward arguments directly to callable using variable argument
template.

## Usage

Can be used like this:

```cpp
{
    OptimisticDispatcher<function<void (uint32_t, string)>> event;

    auto subscriber = event.add([] (auto number, auto text) {
        cout << "Number was " << number << " and text was " << text << endl;
    });

    event.invoke(42, "Hello Universe!");

    // the subscriber is removed from event here automatically
}
```

## Update

Shortly after writing this blog post, I received this note from Jardic:

> Dispatching events usually leads to more events being generated/queued in many program.
> And as such, it could lead to modifications of the vector storing the callbacks and invalidating its iterators.

At first I thought that I took care of iterator invalidation by resuming
iteration using the iterator returned from `erase`:

```cpp
iter = this->callbacks.erase(iter);
```

However, I completely missed the case where an event dispatch could lead
into the same dispatcher being invoked while it is executing, thus messing up
with callback list.

We need somehow avoid clash of aliasing and mutation by separating the event
dispatch from the cleanup. Well, more than that: we have to
ensure cleanup does not happen while iteration is in progress.

We want to avoid creating a new copy of callbacks just to iterate over them,
because that would require an allocation for every event dispatch.

I chose to iterate twice: first for dispatch, and then for cleanup. And
do the clean up only if no one else is dispatching. That means keeping track
of the number of concurent dispatches on the same callback list:

```cpp
private:
    ...
    int32_t concurrent_dispatcher_count = 0;
```

Then, increment and decrement this count for every dispatch iteration:

```cpp
this->concurrent_dispatcher_count++;

// Go over all callbacks and dispatch on those that are still available.
typename std::vector<std::weak_ptr<C>>::iterator iter;

for (iter = this->callbacks.begin(); iter != this->callbacks.end(); ++iter) {
    if (auto callback = iter->lock()) {
        (*callback)(std::forward<A>(args)...);
    }
}

this->concurrent_dispatcher_count--;
```

And then cleanup only if this count is zero, in one go:

```cpp
// Remove all callbacks that are gone, only if we are not dispatching.

if (0 == this->concurrent_dispatcher_count) {
    this->callbacks.erase(
        std::remove_if(
            this->callbacks.begin(),
            this->callbacks.end(),
            [] (std::weak_ptr<C> callback) { return callback.expired(); }
        ),
        this->callbacks.end()
    );
}
```

Actually, this way of doing things might even be more efficient, because
we are not removing elements one by one.

Of course, this dispatcher is still only safe in single-threaded contexts.

## Full Code of Optimistic Dispatcher (updated)

```cpp
#include <memory>
#include <vector>

/**
 * Dispatches function on a number of callbacks and cleans up callbacks when
 * they are dead.
 */
template <typename C> class OptimisticDispatcher final {
private:
    std::vector<std::weak_ptr<C>> callbacks;
    int32_t concurrent_dispatcher_count = 0;
public:
    std::shared_ptr<C> add(C && callback) {
        auto shared = std::make_shared<C>(callback);
        this->callbacks.push_back(shared);
        return shared;
    }

    template <typename ...A>
    void invoke(A && ... args) {
        this->concurrent_dispatcher_count++;

        // Go over all callbacks and dispatch on those that are still available.
        typename std::vector<std::weak_ptr<C>>::iterator iter;

        for (iter = this->callbacks.begin(); iter != this->callbacks.end(); ++iter) {
            if (auto callback = iter->lock()) {
                (*callback)(std::forward<A>(args)...);
            }
        }

        this->concurrent_dispatcher_count--;

        // Remove all callbacks that are gone, only if we are not dispatching.

        if (0 == this->concurrent_dispatcher_count) {
            std::cout << "callback count before erase " << callbacks.size() << std::endl;
            this->callbacks.erase(
                std::remove_if(
                    this->callbacks.begin(),
                    this->callbacks.end(),
                    [] (std::weak_ptr<C> callback) { return callback.expired(); }
                ),
                this->callbacks.end()
            );

            std::cout << "callback count after erase " << callbacks.size() << std::endl;
        }
    }

};
```

## Some Thoughts

C++ let's us do some things here that would not be possible with generics in
other languages like C#, Java or Rust.

Mainly, we did not need to specify anywhere what kind of callable type is
required by this template.

Also, we were able to easily forward to this unknown type the arbitrary argument
list.

Of course, we have hit some iterator invalidation issues, but only Rust
would have prevented that.

### Updates

- Remove forwarding for C in template.
- Make sure dispatcher is re-entrant.

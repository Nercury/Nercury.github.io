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
        auto shared = std::make_shared<C>(std::forward<C>(callback));
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

## Full Code of Optimistic Dispatcher

```cpp
// OptimisticDispatcher.hpp
#pragma once

#include <memory>
#include <vector>

/**
 * Dispatches function on a number of callbacks and cleans up callbacks when
 * they are dead.
 */
template <typename C> class OptimisticDispatcher final {
private:
    std::vector<std::weak_ptr<C>> callbacks;

public:
    std::shared_ptr<C> add(C && callback) {
        auto shared = std::make_shared<C>(std::forward<C>(callback));
        this->callbacks.push_back(shared);
        return shared;
    }

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

};
```

## Some Thoughts

C++ let's us do some things here that would not be possible with generics in
other languages like C#, Java or Rust.

Mainly, we did not need to specify anywhere what kind of callable type is
required by this template.

Also, we were able to easily forward to this unknown type the arbitrary argument
list.

Quite nice.

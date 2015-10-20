---
layout: post
title:  "Porting Twig template engine to Rust"
date:   2015-08-15
categories: rust experience
---
Since the release of Symfony2, Twig has become almost de facto
template engine in PHP world. Developers like its extensibility,
speed and safety; designers are quite familiar with the syntax;
all major IDEs have a plugin for it.

So naturally, with my foray into Rust, I wanted to bring bits of that
world too.

Goal was quite simple: implement __Twig__ in Rust, and make it completely
compatible with existing Twig templates.

## The Challenge

When compared to PHP, Rust is very different. So, even before I started,
I had obvious doubts.

Rust has no full-blown inheritance, and Twig uses inheritance heavily for
AST and extensions. Is there another way in Rust?

Rust has no garbage collector. Instead it uses ownership-based resource
management. Will I need to resort to reference-counting wrappers to implement
same features, or is there a better way?

Twig compiles its templates to PHP and caches them to disk for good runtime
performance. How should it work in Rust? I obviously don't want to invoke
`rustc` from the application. Will I need a capability to run AST and cache
it to disk?

What about Extensions? They are the main selling point for Twig. However,
they go as far as adding custom logic to Twig's parser and even additional
nodes in AST. Clearly they require some kind of abstraction and Twig's
parser requires some kind of API to make this possible.

The remaining portions of this blog post were written in the course of a year,
and tell a story of how it went.

## Picking my Poison

I knew I had to start from Lexer. That was easy to figure out. What was not
easy, is figuring out how to become compatible with original Twig.

The most obvious solution was to open Twig source code and start converting
it to Rust. Very boring.

Or, more Rusty way: write something like Twig, and then change it until
it becomes "good enough".

The first approach would force me into a style that is foreign to Rust.
The second would risk deviating too far away from Twig, effectively creating
another dialect.

However, Twig itself has many tests. So, there was a third way: write basics
of it "Rust way", until a very basic test passes. And then convert each Twig
test to Rust and do the rest "boring way".

## The Tale of Lexer

The `Twig_Lexer` is quite simple, just a single file:

```php
class Twig_Lexer
{
    // 15 protected fields
    // 5 lexer state constants
    // 6 regular expression constants

    public function __construct(Twig_Environment $env, array $options = array())
    {
        $this->env = $env;

        // merge default options with provided options

        $this->regexes = array(
            // build regexes for environment
        );
    }

    public function tokenize($code, $filename = null)
    {
        // process pieces of code based on the state

        return new Twig_TokenStream($this->tokens, $this->filename);
    }

    // helper functions for each state
}
```

Now I had to pick Rust way that would still let me implement
all the features here. However, in Rust, there is no luxury of a GC, so I
had to decide in what sequence objects will be created, used and destroyed.

Let's start from the `Environment`. It is an object that lives the longest.
It basically defines how the Twig is going to behave in _your_ project.

Given this, the `Twig_Lexer` envy to grab it in constructor might seem
reasonable:

{% highlight php5 startinline=true %}
$this->env = $env;
{% endhighlight %}

However, if we look in `Environment`, it has this:

{% highlight php5 startinline=true %}
class Twig_Environment
{
    // many getters

    public function getLexer()
    {
        if (null === $this->lexer) {
            $this->lexer = new Twig_Lexer($this);
        }

        return $this->lexer;
    }
}
{% endhighlight %}

In Twig, the `Environment` seems to be a [God object][god-object] which is an
entry point for both Twig configuration and the runtime state.

Clearly, we have to break this cycle somewhere.

First, there was no reason to implement `getLexer` caching in `Environment`.
If someone using our Twig needs caching, it is trivial to keep same
Lexer around.

Here, we have broken the cycle. But, if we follow the data, we can do even
better. Let's see how much of the actual data the Lexer will need:

- The project Environment configuration is created at the beginning;
- Lexer initializes with correct configuration by reading it from Environment;
- Same Lexer can be used to parse any amount of files, by simply keeping it
around.

Looks like we _should not_ need to keep `env` reference in Lexer. If anything,
we can simply copy necessary invariants to Lexer at the Lexer initialization.

Next, let's check `tokenize`:

{% highlight php5 startinline=true %}
public function tokenize($code, $filename = null)
{
    // process pieces of code based on the state

    return new Twig_TokenStream($this->tokens, $this->filename);
}
{% endhighlight %}

Why does it need `filename` when the `code` is full template string already
in memory?

Turns out, the `filename` is for error messages. Well, I can't argue that
Twig error messages were always great, but I am removing filename from here.
I will try to add it as additional info to error somewhere "higher up", where
it is actually known what file was read.

Next, the `Twig_TokenStream`. Is not really a stream. Not in a sense that
the tokens are streamed from the file. When it is created, all the tokens
are already in memory.

After strugging a bit with few attempts, I settled for the following compromise:
I am going to try and __not__ keep the tokens in memory (by returning them from
iterator), however, for the first version, I am going to leave `code` as
`String`.

Much later I found out that it is actually a desirable solution. I will need to have
full AST in memory anyway, so it is better that it is a tree of nodes of small
fixed-sized structs with pointers to original string blob, rather than many
small allocated strings of varying size.

So, I started from this:

{% highlight rust %}
pub struct Lexer {
    // configuration
}

impl Lexer {
    pub fn with_env(env: &Environment) -> Lexer {
        Lexer {
            // initialize configuration from Environment
        }
    }

    pub fn tokens<'r>(&'r self, code: &'r str) -> Iter
    {
        Iter::new(self, code)
    }
}
{% endhighlight %}

The `'r` lifetime uses the reference to both `Lexer` and `code` for the
duration of `tokens` call.

We will have to create another `Iter` struct that contains iteration state.
However, it simply means that it won't be mixed in the same Lexer impl.
Also the Rust borrow will ensure that we only ever _read_ the configuration when
iterating.

[god-object]: https://en.wikipedia.org/wiki/God_object

## The Drama of UTF-8

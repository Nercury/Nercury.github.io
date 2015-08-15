---
layout: post
title:  "Building Twig template engine for Rust"
date:   2015-08-15
categories: rust experience
---
Since the release in Symfony2, Twig has quickly became the most popular
template engine in PHP world. Developers like its extensibility,
speed and safety; designers by now are quite familiar with its syntax;
all major IDEs have either a built-in support for it, or a plugin.

So naturally, with my foray into Rust, I wanted to bring bits of that
world too.

Goal was quite simple: implement the template engine, that could be
called __Twig__, because it would be completely compatible with existing
templates. In principle, it should be possible to re-use twig templates,
and switch from PHP to Rust.

This blog post tells a story of how it went.

## Picking my Poison

I knew I had to start from Lexer. That was easy to figure out. What was not
easy, is figuring out how to become compatible with original Twig.

The most obvious solution was to open Twig source code and start converting
it to Rust. Very boring.

Another was more Rusty way: write something like Twig, and then change it until
it becomes "good enough".

The first approach was going to force me into a style that is foreign to Rust.
The second had a risk to deviate too far away from Twig, effectively creating
another dialect.

However, Twig itself has many tests. So, there was a third way: write basics
of it "Rust way", until a very basic test passes. And then convert each Twig
test to Rust and do the rest "boring way".

## The Lexer

The `Twig_Lexer` is quite simple, just a single file:

{% highlight php5 startinline=true %}
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
{% endhighlight %}

Starting with that, I had to pick Rust way that would still let me implement
all the features there. However, in Rust, we don't have a luxury of GC, so I
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
`String`. This is what we are going to work with:

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

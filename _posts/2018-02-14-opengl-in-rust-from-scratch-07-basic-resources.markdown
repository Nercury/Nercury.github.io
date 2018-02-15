---
layout: post
title:  "Rust and OpenGL from scratch - Basic Resources"
date:   2018-02-14
categories: rust opengl tutorial
---

Welcome back!

[Previously](/rust/opengl/tutorial/2018/02/12/opengl-in-rust-from-scratch-06-gl-generator.html), 
we have created our own `gl` crate, that allows us to control OpenGL API coverage and extensions, as
well as see OpenGL function invocations and errors in the debug mode.

This time, we will load shaders from files at start-up. The way we've done it previously (embedding them
in executable) may be fine for certain cases, but requires program recompilation
every time the shader is modified, which slows down the experimentation.

## Resources

Shaders won't be the only thing we will need to load. Things like images, fonts, models, audio, configuration
will all need to be loaded. This time we will start working on basic `CString` loader, which can
later be extended to load all kinds of things.

The only need we have so far is to load our shader program from the `.vert` and `.frag` files.

Let's go ahead and create a new empty module file named `resources.rs` in `src` directory,
and then define a reference to it inside of `main.rs`:

(main.rs, at the top of the file)

```rust
pub mod resources;
```

## Usage in main

Inside the `resources`, we will have `Resources` struct, which will be our resource loader.
At the top of our `main.rs`, we will initialize it to point to the path of our executable:

(main.rs)

```rust
use resources::Resources;

fn main() {
    let res = Resources::from_exe_path().unwrap();
    
    ...
}
```

And we should be able to load our shader program from resources instead of source:

```rust
let shader_program = render_gl::Program::from_res(
    &gl, &res, "shaders/triangle"
).unwrap();
```

We will leave it for `Program` to take care and find the necessary `triangle.vert` and `triangle.frag`
variants.

We use `.unwrap()` liberally in main, because panicking there and terminating the program
is reasonable way to handle errors. However, in the deeper module, we should always return
`Result` and leave the decision of how to handle an error to the module user.

## Resources

Our `Resources` struct will have function to initialize resources, which might fail.
In that case, it may return an error.

Previously with SDL, we used `String` for error value everywhere. If the only error 
we are interested in is SDL error, `String` is a sensible choice, because there is not much more
SDL could return.

Using `String` has few problems though:

- If an error is expected to happen often and in hot path, `String` creation would
slow the program down, because it requires allocation;
- It is inconvenient to handle `String` error - if, say, we want to terminate the program
on some errors but not the others, `String` is absolutely not ideal;
- The program logic is cluttered with string formatting everywhere!

For these reasons, it's much better to create a module-local enum and list add possible error
cases there as needed. Then, take care of nice error formatting in another place.

## Resources Error enum

Inside `resources.rs`, besides `Resources`
struct with `root_path`, let's also add `Error` enum, which will contain any error this module 
could return:

(resources.rs, full file)

```rust
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    FailedToGetExePath,
}

pub struct Resources {
    root_path: PathBuf,
}

impl Resources {
    pub fn from_exe_path() -> Result<Resources, Error> {
        // continue here
    }
}
```

Adding the `#[derive(Debug)]` attribute auto-implements the `Debug` trait for `Error`, for it
to be printed with `{:?}` formatter.
When we call `.unwrap()` and the program panics on such an error, `Debug` implementation is
used to print it to
the output. 

Then, when we use a function like `std::env::current_exe` that returns `Result`, we can
change the error returned from there to our `Error::FailedToGetExePath` with [`map_err` function
defined on all `Result` types](https://doc.rust-lang.org/beta/std/result/enum.Result.html#method.map_err):

(resources.rs, continued)

```rust
impl Resources {
    pub fn from_exe_path() -> Result<Resources, Error> {
        let exe_file_name = ::std::env::current_exe()
            .map_err(|_| Error::FailedToGetExePath)?;
            
        // continue here
    }
}
```

The `?` at the end unwraps the `Ok` result (so that `exe_file_name` contains `PathBuf`), or exits
from function with `Err(Error::FailedToGetExePath)`. Note that `?` can exit from 
function only if the error types match!

We got executable file name in `exe_file_name` - but we need a path to that.
[We can use `.parent()` function on `Path`](https://doc.rust-lang.org/beta/std/path/struct.Path.html#method.parent) (`PathBuf` implements all `Path` functions):

(resources.rs, continued)

```rust
let exe_path = exe_file_name.parent()
    .ok_or(Error::FailedToGetExePath)?;
    
// continue here
```

However, it is possible (though highly unlikely) that `exe_file_name` has no parent, in that case
`.parent` may return `Option`, and we need to handle that too. We can change `Option` to `Result` with
[handy `ok_or` function](https://doc.rust-lang.org/beta/std/option/enum.Option.html). In case ok `Some(T)` value, it will return `Ok(T)`, and we provide 
`Error::FailedToGetExePath` to be used if the result was `None`.

The value returned from `ok_or` is now changed to `Result`, and we can again use `?` to unwrap the `Ok`
value, or return from the function with the error.

If everything went well up to this point, we can finally return the `Resources` struct:

(resources.rs, continued)

```rust
Ok(Resources {
    root_path: exe_path.into()
})
```

That `.into()` function on `exe_path` might be confusing. Turns out, `.parent` does not return a
`PathBuf`, instead it returns a reference to the portion of `PathBuf` in the form of `Path`, the
same way a `str` slice reference can be returned from a `String`.

But in the standard library, there is a trait `From<Path>` implemented for `PathBuf`, 
that can create a new allocated `PathBuf`
from a `Path` reference. We could have used it this way:

(example)

```rust
Ok(Resources {
    root_path: PathBuf::from(exe_path) // de-sugared exe_path.into()
})
```

So, what that `into` was all about? Turns out that in standard library there is a sibling `Into` trait,
that is implemented for [ANY type A that has `B::from(A)`](https://doc.rust-lang.org/beta/std/convert/trait.Into.html).
Yep, Rust type system can do this kind of trickery. It means that, as long as
the `from` conversion exists, and the target type can be deduced (it can be, because
we defined `root_path` as `PathBuf`), the `.into()` can be used to convert between the two.

## Loading CString from file

First, let's add few more `use` statements:

(resources.rs)

```rust
use std::fs;
use std::io::{self, Read};
use std::ffi;
```

The `use io::{self, Read}` is a comonly used shorthand for `use io` and `use io::Read`, same as:

(example)

```rust
use std::io;
use std::io::Read;
```

Then, we will have few more kinds of errors! I know this, because I've already written the code.
Usually though, we add new error types as they are needed:

(resources.rs, modified)

```rust
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    FileContainsNil,
    FailedToGetExePath,
}
```

The `Error` enum `Io` variant can contain an inner `io::Error`. File and networking
functions usually return `io::Result`, which has `io::Error` for error type.

To easily convert from `io::Error` to our `Error`, we can implement `From` trait we were talking
about earlier:

(resources.rs, bellow enum Error)

```rust
impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Error::Io(other)
    }
}
```

Finally, we can add `load_cstring` function to `Resources` impl:

(resources.rs, inside impl Resources)

```rust
impl Resources {
    ...

    pub fn load_cstring(&self, resource_name: &str) -> Result<ffi::CString, Error> {
        let mut file = fs::File::open(
            self.root_path.join(resource_name)
        )?;
    }
}
```

The `self.root_path.join(resource_name)` extends the path with a file name.
However, we will also need to use relative paths, in which we should probably hide
platform differences: "shaders/traingle.vert" should work on both Windows and Unix.

For that, we can write a helper function that transforms resource name to full path:

(resources.rs, at the end)

```rust
use std::path::Path;

fn resource_name_to_path(root_dir: &Path, location: &str) -> PathBuf {
    let mut path: PathBuf = root_dir.into();

    for part in location.split("/") {
        path = path.join(part);
    }

    path
}
```

Inside, we iterate over the chunks of resource name separated by "/", and then add
them to path, which will internally use platform-correct separator.

Let's fix-up file-open statement to use this function:

(resources.rs, inside impl Resources)

```rust
impl Resources {
    ...

    pub fn load_cstring(&self, resource_name: &str) -> Result<ffi::CString, Error> {
        let mut file = fs::File::open(
            resource_name_to_path(&self.root_path,resource_name)
        )?;
        
        // continue here
    }
}
```

Rememeber how I've said that when using `?`, the error type in Result returned from function should match
the the error we are handling with `?`. Turns out, the `?` calls `.into()` on the error behind the
scenes, which uses our `From<io::Error>` implementation to convert `io::Error` to our error.

Next up, we allocate `Vec` of bytes of the file size + 1 byte (for `0` byte at the end), 
read file contents into it, check that file contents do not contain `0`, create `CString` from
it and return it:

```rust
// allocate buffer of the same size as file
let mut buffer: Vec<u8> = Vec::with_capacity(
    file.metadata()?.len() as usize + 1
);
file.read_to_end(&mut buffer)?;

// check for nul byte
if buffer.iter().find(|i| **i == 0).is_some() {
    return Err(Error::FileContainsNil);
}

Ok(unsafe { ffi::CString::from_vec_unchecked(buffer) })
```

We need to dereference `**i` two times in `find`, because the `i` there is double reference.
How did we get the double reference there? Well, the `iter()` function returns iterator over the
references, and the closure in `find` function receives a reference of whatever iterator iterates over.

## Using our load_cstring function

Now we can use `load_cstring` function in our `Shader` to initialize it from file:

(render_gl.rs, impl Shader)

```rust
use resources::Resources;

impl Shader {
    pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Shader, String> {
        const POSSIBLE_EXT: [(&str, gl::types::GLenum); 2] = [
            (".vert", gl::VERTEX_SHADER),
            (".frag", gl::FRAGMENT_SHADER),
        ];

        let shader_kind = POSSIBLE_EXT.iter()
            .find(|&&(file_extension, _)| {
                name.ends_with(file_extension)
            })
            .map(|&(_, kind)| kind)
            .ok_or_else(|| format!("Can not determine shader type for resource {:?}", name))?;

        let source = res.load_cstring(name)
            .map_err(|e| format!("Error loading resource {:?}: {}", name, e))?;

        Shader::from_source(gl, &source, shader_kind)
    }

    ...

}
```

Here we check file extension to determine the `shader_kind`.

Similarly, we add `from_res` to shader `Program`:

(render_gl.rs, impl Program)

```rust
impl Program {
    pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Program, String> {
        const POSSIBLE_EXT: [&str; 2] = [
            ".vert",
            ".frag",
        ];

        let shaders = POSSIBLE_EXT.iter()
            .map(|file_extension| {
                Shader::from_res(gl, res, &format!("{}{}", name, file_extension))
            })
            .collect::<Result<Vec<Shader>, String>>()?;

        Program::from_shaders(gl, &shaders[..])
    }
    
    ...
    
}
```

Here, we require both `.vert` and `.frag` files to exist.
One interesting note on the `collect` function: when we have a bunch of `Result<T, E>` items,
such as returned from `Shader::from_res`, we can collect them into a `Result<Vec<T>, E>`,
which will contain a first encountered error or a list of unwrapped values.

With that done, we can modify our main function to use resource loader:

(main.rs, modified)

```rust
// set up shader program

let shader_program = render_gl::Program::from_res(&gl, &res, "shaders/triangle").unwrap();
```

If we run our program now, we get the error:

```txt
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: 
"Error loading resource \"shaders/triangle.vert\": 
Io(Error { repr: Os { code: 3, message: \"The system cannot find the path specified.\" } })"'
```

Our executable is placed into `target/debug`, while our shader files are in `src`.

## Build script

We will create a build script that copies our shaders from project asset directory to 
the build directory.

First, let's move our shaders from `src` to `shaders` - from now on, only the Rust code
should be in `src`.

Then, add a `build.rs` file with the following code:

(build.rs, complete)

```rust
extern crate walkdir;

use std::env;
use std::fs::{self, DirBuilder};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // locate executable path even if the project is in workspace

    let executable_path = locate_target_dir_from_output_dir(&out_dir)
        .expect("failed to find target dir")
        .join(env::var("PROFILE").unwrap());

    copy(
        &manifest_dir.join("shaders"),
        &executable_path.join("shaders"),
    );
}

fn locate_target_dir_from_output_dir(mut target_dir_search: &Path) -> Option<&Path> {
    loop {
        // if path ends with "target", we assume this is correct dir
        if target_dir_search.ends_with("target") {
            return Some(target_dir_search);
        }

        // otherwise, keep going up in tree until we find "target" dir
        target_dir_search = match target_dir_search.parent() {
            Some(path) => path,
            None => break,
        }
    }

    None
}

fn copy(from: &Path, to: &Path) {
    let from_path: PathBuf = from.into();
    let to_path: PathBuf = to.into();
    for entry in WalkDir::new(from_path.clone()) {
        let entry = entry.unwrap();

        if let Ok(rel_path) = entry.path().strip_prefix(&from_path) {
            let target_path = to_path.join(rel_path);

            if entry.file_type().is_dir() {
                DirBuilder::new()
                    .recursive(true)
                    .create(target_path).expect("failed to create target dir");
            } else {
                fs::copy(entry.path(), &target_path).expect("failed to copy");
            }
        }
    }
}
```

The important bit is `copy` function, that copies our shader assets:

(example)

```rust
copy(
    &manifest_dir.join("shaders"),
    &executable_path.join("shaders"),
);
```

Our `copy` function copies files recursively using `walkdir` crate, so let's add
`walkdir` to our crate dependencies:

(Cargo.toml, incomplete)

```toml
[build-dependencies]
walkdir = "2.1"
```

This does the trick, and we see our triangle again!

This, of course, still requires re-compilation for build script to copy files,
but we can also run the program directly from `/target` directory and
modify the shader files there.

## Shader Errors

For initial simplicity, we have not created another `Error` type for shader, and used
simple `String` instead. Let's change `String` to module-local enum too, in the same
way we've done this for `Resources`. Then we can change result returned from `from_res`
functions to `Result<Program, Error>`.

Before copy-pasting the final source, see if you can do
it yourself.

In my case, final `Error` structure ended up looking like this:

```rust
#[derive(Debug)]
pub enum Error {
    ResourceLoad { name: String, inner: resources::Error },
    CanNotDetermineShaderTypeForResource { name: String },
    CompileError { name: String, message: String },
    LinkError { name: String, message: String },
}
```

I decided to include additional `name` information in the `ResourceLoad` error, therefore
I have't used the automatic error conversion with `From<resource::Error>` here. 

[The full code is available on github](https://github.com/Nercury/rust-and-opengl-lessons/tree/master/lesson-07).

[Next time, we will make our error output look much nicer](/rust/opengl/tutorial/2018/02/15/opengl-in-rust-from-scratch-08-failure.html).
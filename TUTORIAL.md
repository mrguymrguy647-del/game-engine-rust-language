# Learn Rust by Building a Game Engine

This guide teaches you Rust using the code in this repository: a small 2D
game engine called **tiny_engine** and three games built on it. Every Rust
concept is shown in real code that you can run, change and break.

- **Part 1** teaches the Rust language, one idea at a time.
- **Part 2** explains how the engine works and why it's built that way.
- **Part 3** has exercises, from "change a number" to "write a new game".

Keep the source files open next to this guide. When you see a file like
`src/canvas.rs`, open it and look at the real code.

---

## Contents

- [0. Setup](#0-setup)
- **Part 1: Learning Rust**
  - [1. Cargo: Rust's build tool](#1-cargo-rusts-build-tool)
  - [2. Variables, mutability and types](#2-variables-mutability-and-types)
  - [3. Functions and expressions](#3-functions-and-expressions)
  - [4. Structs and methods](#4-structs-and-methods)
  - [5. Enums and pattern matching](#5-enums-and-pattern-matching)
  - [6. Ownership and borrowing](#6-ownership-and-borrowing)
  - [7. Collections, loops, closures and iterators](#7-collections-loops-closures-and-iterators)
  - [8. Traits](#8-traits)
  - [9. Modules and visibility](#9-modules-and-visibility)
  - [10. Error handling](#10-error-handling)
  - [11. Tests](#11-tests)
- **Part 2: How the engine works**
  - [12. The big picture](#12-the-big-picture)
  - [13. The game loop](#13-the-game-loop)
  - [14. Pixels and the framebuffer](#14-pixels-and-the-framebuffer)
  - [15. Drawing shapes and text](#15-drawing-shapes-and-text)
  - [16. Keyboard input](#16-keyboard-input)
  - [17. Random numbers](#17-random-numbers)
  - [18. Breakout, piece by piece](#18-breakout-piece-by-piece)
- **Part 3: Your turn**
  - [19. Exercises](#19-exercises)
  - [20. Common compiler errors](#20-common-compiler-errors)
  - [21. Where to go next](#21-where-to-go-next)

---

## 0. Setup

**Install Rust** from <https://rustup.rs>. It installs three tools:

| Tool     | What it does                                         |
|----------|------------------------------------------------------|
| `rustc`  | The compiler. You'll rarely call it directly.        |
| `cargo`  | The build tool and package manager. You'll use this all the time. |
| `rustup` | Installs and updates Rust itself (`rustup update`).  |

On Windows the installer will ask you to install the Visual Studio C++ Build
Tools. Say yes: Rust needs their linker. This project needs Rust 1.85 or
newer; `rustc --version` tells you what you have.

**Run the games** from the project folder:

```sh
cargo run --example hello              # a square you move with the arrow keys
cargo run --example shapes             # every drawing function, animated
cargo run --release --example breakout # the full game
```

The first build takes a minute because Cargo downloads and compiles minifb
and the crates it depends on. After that builds are fast. `--release` turns on optimizations;
the game runs fine without it, but release builds are much faster.

**Cargo commands you'll use:**

| Command            | What it does                                              |
|--------------------|-----------------------------------------------------------|
| `cargo check`      | Checks your code for errors without building. The fastest feedback. |
| `cargo build`      | Compiles everything.                                      |
| `cargo run --example NAME` | Builds and runs `examples/NAME.rs`.               |
| `cargo test`       | Runs all the tests.                                       |
| `cargo clippy`     | A linter with hundreds of helpful suggestions.            |
| `cargo fmt`        | Formats your code in the standard style.                  |
| `cargo doc --open` | Builds documentation for the engine and opens it in your browser. |

**The project layout:**

```text
Cargo.toml          project name, Rust edition, dependencies
src/
  lib.rs            the front door: lists the modules, defines the prelude
  engine.rs         the Game trait and the game loop (the only file that uses minifb)
  canvas.rs         the pixel buffer and all drawing functions
  font.rs           a tiny 3x5 pixel font
  input.rs          keyboard state
  math.rs           Vec2 and Rect
  color.rs          Color
  rng.rs            random numbers
examples/
  hello.rs          the smallest game
  shapes.rs         a tour of the drawing functions
  breakout.rs       a complete game
```

---

# Part 1: Learning Rust

## 1. Cargo: Rust's build tool

Open `Cargo.toml`:

```toml
[package]
name = "tiny_engine"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "A small 2D game engine, written for learning Rust"

[dependencies]
minifb = "0.28"
```

- A Rust project is called a **crate**. This one is a *library crate*
  because it has a `src/lib.rs`. (A program with a `main` function has a
  `src/main.rs` and is a *binary crate*.)
- `edition = "2024"` picks the version of the language rules. Editions let
  Rust improve without breaking old code. `rust-version` is the oldest
  compiler that can build the project.
- `[dependencies]` lists other crates to download from
  [crates.io](https://crates.io). We use exactly one: **minifb**, which opens
  a window, shows pixels and reports key presses. Everything else we write
  ourselves, because that's the point of the exercise.

Files in `examples/` are small programs that use the library. Each one
starts with:

```rust
use tiny_engine::prelude::*;
```

That line brings the engine's most-used names (`Game`, `Canvas`, `Color`,
`Vec2`, ...) into scope. We'll see how in [chapter 9](#9-modules-and-visibility).

## 2. Variables, mutability and types

### `let` and `mut`

Variables are created with `let`, and **they can't be changed unless you
say `mut`**:

```rust
let lives = 3;
lives = 2; // error!
```

The compiler refuses:

```text
error[E0384]: cannot assign twice to immutable variable `lives`
help: consider making this binding mutable
```

With `mut` it's fine:

```rust
let mut lives = 3;
lives = 2; // ok
```

This might feel strict, but it means that when you read `let x = ...` you
*know* `x` never changes, which makes code much easier to follow.

### Shadowing

You can declare a new variable with the same name as an old one. The new
one "shadows" the old. `update_ball` in `examples/breakout.rs` does this:

```rust
let step = self.ball_velocity.x * dt;
// ... use step for the x axis ...
let step = self.ball_velocity.y * dt; // a brand new `step`
// ... use step for the y axis ...
```

### Types

Every value has a type. Rust usually works it out for you (`let x = 3;` is
an `i32`), but you can write it out: `let x: f32 = 3.0;`.

The number types in the engine, and why each was chosen:

| Type    | What it holds                   | Where the engine uses it                          |
|---------|---------------------------------|---------------------------------------------------|
| `f32`   | 32-bit decimal number           | Positions, speeds, time: they need fractions      |
| `i32`   | 32-bit signed integer           | Pixel coordinates: can be negative (off-screen)   |
| `u32`   | 32-bit unsigned (never negative)| Score, lives, one packed pixel color              |
| `u8`    | 0 to 255                        | One color channel (red, green or blue)            |
| `usize` | Unsigned, the size of a memory address | Lengths and indexes: `vec[i]` needs a `usize` |
| `u64`   | 64-bit unsigned                 | The frame counter                                 |

Plus `bool` (`true`/`false`), `char` (one Unicode character) and `&str`
(text, see [chapter 6](#strings-string-vs-str)).

### Converting with `as`

Rust **never converts number types for you**. This doesn't compile:

```rust
let width: usize = 320;
let half: f32 = width / 2;
```

```text
error[E0308]: mismatched types
   |     let half: f32 = width / 2;
   |               ---   ^^^^^^^^^ expected `f32`, found `usize`
help: you can cast a `usize` to an `f32`
   |     let half: f32 = (width / 2) as f32;
```

You'll see `as` all over the engine, like in `Canvas::index`:

```rust
if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
```

It's a little noisy, but it forces you to think about every conversion,
and conversions are where many bugs come from. (What does `-1` mean as a
`usize`? Rust makes you decide.)

### Constants

`const` values are fixed forever and must have a written type. By
convention they're in `SCREAMING_SNAKE_CASE`. The top of
`examples/breakout.rs` is full of them:

```rust
const PADDLE_WIDTH: f32 = 48.0;
const PADDLE_SPEED: f32 = 240.0;
const BALL_START_SPEED: f32 = 150.0;
```

Collecting the "tuning knobs" at the top of a file like this makes a game
easy to tweak. Try changing them!

## 3. Functions and expressions

A function lists its parameter types and, after `->`, its return type:

```rust
/// Shorthand for [`Vec2::new`]: `vec2(1.0, 2.0)`.
pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}
```

Two things to notice:

1. **There's no `return`.** In Rust, a block's value is its last
   *expression*, a line **without a semicolon**. `Vec2 { x, y }` is the
   last expression, so it's the return value. Adding a `;` would turn it
   into a statement that returns nothing, and the compiler would complain.
2. **`Vec2 { x, y }`** is shorthand for `Vec2 { x: x, y: y }`, used when
   the variables have the same names as the fields.

Comments starting with `///` are **doc comments**. They show up in
`cargo doc` and when you hover in your editor.

### `if` is an expression too

Since `if` produces a value, there's no need for a `? :` operator. From
`Canvas::draw_line`:

```rust
let step_x = if x < x_end { 1 } else { -1 };
```

And from `Vec2::normalized`:

```rust
pub fn normalized(self) -> Self {
    let len = self.length();
    if len == 0.0 {
        Self::ZERO
    } else {
        self * (1.0 / len)
    }
}
```

The whole `if`/`else` is the last expression, so its value is returned.

You can still leave a function early with `return`, as in `Canvas::index`:

```rust
fn index(&self, x: i32, y: i32) -> Option<usize> {
    if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
        return None;
    }
    Some(y as usize * self.width + x as usize)
}
```

## 4. Structs and methods

A **struct** groups related data. Here's `Vec2` from `src/math.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
```

(The `#[derive(...)]` line is explained in [chapter 8](#derive-free-trait-implementations).)

A game's whole state is usually one struct. Here's Breakout's:

```rust
struct Breakout {
    state: State,
    paddle: Rect,
    ball: Rect,
    ball_velocity: Vec2,
    ball_speed: f32,
    bricks: Vec<Brick>,
    particles: Vec<Particle>,
    score: u32,
    lives: u32,
}
```

### Methods live in `impl` blocks

```rust
impl Vec2 {
    pub const ZERO: Vec2 = vec2(0.0, 0.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
```

- `Self` means "the type this `impl` is for", here `Vec2`.
- `new` has no `self` parameter, so it's an **associated function**, called
  with `::` like `Vec2::new(1.0, 2.0)`. Rust has no constructors; a function
  called `new` is just a convention.
- `length` takes `self`, so it's a **method**, called with a dot:
  `velocity.length()`.

### `self`, `&self` and `&mut self`

The first parameter of a method says how it uses the value. This is one of
the most important things to understand in Rust:

| Parameter   | Meaning                                          | Example                            |
|-------------|--------------------------------------------------|------------------------------------|
| `&self`     | Borrows the value to **read** it                 | `Rect::overlaps(&self, other: &Rect)` |
| `&mut self` | Borrows the value to **change** it               | `Canvas::clear(&mut self, color: Color)` |
| `self`      | Takes the value itself (ownership moves in)      | `Vec2::length(self)`               |

`Vec2::length` can take `self` cheaply because `Vec2` is `Copy` (it's just
two numbers), so calling it copies the vector. [Chapter 6](#6-ownership-and-borrowing)
explains what all of this really means.

### Private fields and getters

`Vec2`'s fields are `pub`, so anyone can read or write `v.x`. `Canvas`'s
fields are *not* `pub`:

```rust
pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
}
```

Outside `canvas.rs` you can only read the size through `canvas.width()`.
Why? If a game could set `canvas.width = 1000` without resizing `pixels`,
every drawing function would index out of bounds. Keeping the fields
private makes that mistake impossible.

### Struct update syntax and destructuring

Every example creates its `Config` like this:

```rust
let config = Config {
    title: String::from("Hello, tiny_engine"),
    ..Config::default()
};
```

`..Config::default()` means "take every field I didn't list from
`Config::default()`".

You can also pull a struct apart into variables. From `Canvas::draw_rect`:

```rust
let Rect { x, y, w, h } = rect;
```

## 5. Enums and pattern matching

An **enum** is a type whose value is exactly *one* of several variants.
Breakout's game state:

```rust
enum State {
    Serving,
    Playing,
    Paused,
    GameOver,
    Won,
}
```

The game can never be "sort of paused and sort of game over". With separate
`bool` flags like `is_paused` and `is_game_over` that bug would be possible.
With an enum it can't happen.

### `match`

`match` runs the arm that fits the value. Breakout's `update` is built
around one:

```rust
match self.state {
    State::Serving => {
        self.move_paddle(ctx);
        self.put_ball_on_paddle();
        if ctx.input.was_pressed(Key::Space) {
            self.launch_ball(&mut ctx.rng);
        }
    }
    State::Playing => { /* ... */ }
    State::Paused => { /* ... */ }
    State::GameOver | State::Won => { /* ... */ } // `|` matches either
}
```

**`match` must cover every variant.** If you add a new state and forget to
handle it, the program won't compile:

```text
error[E0004]: non-exhaustive patterns: `State::Paused` not covered
```

This is great when a program grows: add a variant, then let the compiler
show you every place that needs updating.

To deliberately ignore the rest, use the `_` wildcard. `convert_key` in
`src/engine.rs` handles 14 keys and ignores the other ~100:

```rust
let key = match key {
    minifb::Key::Left => Key::Left,
    minifb::Key::Right => Key::Right,
    // ...
    _ => return None,
};
```

`match` is also an expression that returns a value. From `draw_hud`:

```rust
let (title, hint) = match self.state {
    State::Playing => return, // nothing to show while playing
    State::Serving => ("", "PRESS SPACE TO LAUNCH"),
    State::Paused => ("PAUSED", "PRESS P TO CONTINUE"),
    State::GameOver => ("GAME OVER", "PRESS SPACE TO TRY AGAIN"),
    State::Won => ("YOU WIN!", "PRESS SPACE TO PLAY AGAIN"),
};
```

Each arm gives back a *tuple* of two strings, which `let (title, hint)`
unpacks.

### Enums can carry data: `Option`

Variants can hold values. The most important enum in Rust is in the
standard library:

```rust
enum Option<T> {
    Some(T), // there is a value, of type T
    None,    // there is no value
}
```

**Rust has no `null`.** When something might be missing, its type says so.
`Canvas::get_pixel` returns `Option<Color>` because the coordinates might be
off the canvas:

```rust
pub fn get_pixel(&self, x: i32, y: i32) -> Option<Color>
```

You *can't* use an `Option<Color>` as a `Color` by accident; you have to
deal with `None` first. The engine shows several ways to do that:

```rust
// if let: run code only when there is a value
if let Some(i) = self.index(x, y) {
    self.pixels[i] = color.to_u32();
}

// map: transform the value inside, if there is one
self.index(x, y).map(|i| Color::from_hex(self.pixels[i]))

// unwrap_or: use a fallback when there's no value
let longest = text.lines().map(|line| line.chars().count()).max().unwrap_or(0);

// let-else: take the value, or leave the function
let Some(index) = self.bricks.iter().position(|brick| brick.rect.overlaps(&self.ball)) else {
    return false;
};
```

There's also `.unwrap()`, which takes the value and **crashes the program
if it's `None`**. It's fine in tests and quick experiments. In real code,
prefer the forms above.

## 6. Ownership and borrowing

This is the chapter that makes Rust *Rust*. Take your time with it.

Most languages manage memory in one of two ways: you free it by hand (C,
C++), which is fast but causes crashes and security holes when you get it
wrong; or a *garbage collector* frees it for you (Java, Python, C#), which
is safe but costs performance and causes pauses. Rust does neither. Instead
the compiler follows a few rules that decide, at compile time, exactly when
each value is freed.

### The rules of ownership

1. Every value has exactly one **owner** (a variable, a struct field, ...).
2. When the owner goes away, the value is freed.
3. Ownership can be **moved** to a new owner. The old owner can't be used
   any more.

```rust
let name = String::from("Breakout");
let title = name;      // ownership moves from `name` to `title`
println!("{}", name);  // error!
```

```text
error[E0382]: borrow of moved value: `name`
   |     let name = String::from("Breakout");
   |         ---- move occurs because `name` has type `String`, which does not implement the `Copy` trait
   |     let title = name;
   |                 ---- value moved here
   |     println!("{}", name);
   |                    ^^^^ value borrowed here after move
help: consider cloning the value if the performance cost is acceptable
```

Why? If both variables owned the same text, it would be freed twice when
they both went away. Rust's answer: only one owner at a time.

You can see a move in every example's `main`:

```rust
tiny_engine::run(config, Breakout::new())
```

`run` takes the game *by value*, so the engine now owns your game for as long
as it runs. The game is freed when `run` finishes.

Another move is in Breakout, where a brick is destroyed:

```rust
let brick = self.bricks.remove(index);
self.score += brick.points;
```

`Vec::remove` moves the brick out of the vector and gives it to us. It's
freed at the end of the function, after we've used its points, color and
position.

### `Copy` types

Small, simple values like numbers are **copied** instead of moved, so the
old variable stays usable:

```rust
let a = 5;
let b = a; // copies
println!("{a}"); // fine
```

Our own types can opt in with `#[derive(Clone, Copy)]`. `Vec2`, `Rect` and
`Color` do, because they're just a few numbers and copying them costs
nothing. That's why you can write `let pos = self.position;` freely.

`Canvas` is **not** `Copy`: it owns a `Vec` of 76,800 pixels. Copying it
by accident on every function call would be slow, so Rust makes copying
explicit (you'd have to call `.clone()`, and `Canvas` doesn't even offer
that).

### Borrowing: `&` and `&mut`

Moving everything around would be exhausting, so usually you **borrow**
instead. A borrow is a *reference*: a pointer the compiler checks for
you.

- `&value` is a **shared borrow**: you can read, not change.
- `&mut value` is a **mutable borrow**: you can change it.

**The borrowing rule:** at any moment you can have *either* any number of
`&` borrows, *or* exactly one `&mut` borrow. Never both.

Look at the heart of the game loop in `src/engine.rs`:

```rust
game.update(&mut ctx);
game.draw(&mut canvas);
```

And the `Game` trait:

```rust
fn update(&mut self, ctx: &mut Context);
fn draw(&self, canvas: &mut Canvas);
```

- `update` gets `&mut self` (it can change the game) and `&mut Context`
  (it can use the random number generator or ask to quit).
- `draw` gets `&self` (it can only *look* at the game) and `&mut Canvas`
  (it can draw). If you try to change the score in `draw`, it won't
  compile. The type signature is a promise that the compiler enforces.

### Why the rule exists

Here's a bug that's easy to write in many languages: changing a list while
you're looping over it. Rust catches it at compile time. Try this:

```rust
impl Game {
    fn add_points(&mut self, n: u32) {
        self.score += n;
    }

    fn update(&mut self) {
        for brick in &self.bricks {   // shared borrow of self.bricks...
            if brick.alive {
                self.add_points(10);  // ...but this needs `&mut self`!
            }
        }
    }
}
```

```text
error[E0502]: cannot borrow `*self` as mutable because it is also borrowed as immutable
  |         for brick in &self.bricks {
  |                      ------------
  |                      |
  |                      immutable borrow occurs here
  |                      immutable borrow later used here
  |             if brick.alive {
  |                 self.add_points(10);
  |                 ^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
```

`add_points` takes `&mut self`, so it *could* change `self.bricks` while
the loop is walking over it. The compiler can't know it won't, so it says
no. Two ways to fix it:

1. **Touch the field directly.** `self.score += 10;` compiles fine inside
   the loop, because the compiler can see that `self.score` and
   `self.bricks` are *different fields*.
2. **Split the work into two steps**: first find what you need with a
   shared borrow, then change things once that borrow has ended.

Breakout uses the second approach in `break_brick_under_ball`:

```rust
// Step 1: a shared borrow, just to find the index. It ends on this line.
let Some(index) = self
    .bricks
    .iter()
    .position(|brick| brick.rect.overlaps(&self.ball))
else {
    return false;
};

// Step 2: now we're free to change things.
let brick = self.bricks.remove(index);
self.score += brick.points;
// ...
self.spawn_particles(brick.rect.center(), brick.color, rng);
```

When you fight the borrow checker, this "find first, change second"
pattern is usually the answer.

### Ask for only what you need

`break_brick_under_ball` needs random numbers for the particles. It could
take the whole `&mut Context`, but it takes just `rng: &mut Rng`:

```rust
fn break_brick_under_ball(&mut self, rng: &mut Rng) -> bool
```

and is called with `self.break_brick_under_ball(&mut ctx.rng)`. You can
borrow one field of a struct on its own. Asking for less makes a function
easier to understand and easier to call.

### `std::mem::take`

Sometimes you want to move a value *out* of something you only have a
`&mut` to. `Input::begin_frame` does this:

```rust
self.down_last_frame = std::mem::take(&mut self.down);
self.down.extend(keys_down);
```

`mem::take` moves the set out of `self.down` and leaves an empty set
behind. Nothing is copied, and `self.down` is never left invalid.

### Strings: `String` vs `&str`

- `String` is **owned** text. It lives on the heap and can grow.
  `String::from("hi")` and `format!("SCORE {}", score)` create one.
- `&str` is a **borrowed** view of some text. String literals like
  `"GAME OVER"` are `&str`.

Functions that only read text should take `&str`, like `draw_text`:

```rust
pub fn draw_text(&mut self, text: &str, pos: Vec2, scale: u32, color: Color)
```

Then it accepts both kinds:

```rust
canvas.draw_text("HELLO, RUST!", vec2(8.0, 8.0), 2, Color::WHITE); // a &str literal
let lives = format!("LIVES {}", self.lives);                        // a String
canvas.draw_text(&lives, vec2(x, 4.0), 1, Color::WHITE);            // borrowed as &str
```

The same idea applies to arrays: `Vec<u32>` owns its items, while `&[u32]` (a
*slice*) borrows some of them. `Canvas::pixels` lends out the pixels
without giving away ownership:

```rust
pub fn pixels(&self) -> &[u32] {
    &self.pixels
}
```

### A word on lifetimes

The compiler tracks how long every borrow lives, and never lets a
reference outlive the value it points to. In `pixels()` above, it knows the
returned slice borrows from `self`, so the canvas can't be freed or
changed while someone is still holding that slice. Sometimes you have to
spell these relationships out with *lifetime annotations* like `<'a>`.
This engine never needed one, and small programs rarely do.

## 7. Collections, loops, closures and iterators

### `Vec<T>`: a growable list

```rust
let mut bricks = Vec::with_capacity(rows * BRICK_COLUMNS); // empty, with room reserved
bricks.push(Brick { /* ... */ });                          // add to the end
let brick = self.bricks.remove(index);                     // take one out
if self.bricks.is_empty() { /* ... */ }                    // check if empty
let pixels = vec![0; width * height];                      // `width * height` zeros
self.pixels[i] = color.to_u32();                           // index (panics if out of range)
```

`Input` uses a `HashSet<Key>`, a collection with no duplicates and a fast
`contains` check, which is exactly what "which keys are held?" needs.

### Loops

```rust
for column in 0..BRICK_COLUMNS { }   // 0, 1, ..., BRICK_COLUMNS - 1
for c in 'A'..='Z' { }               // ..= includes the end
for brick in &self.bricks { }        // borrow each item to read it
for particle in &mut self.particles { } // borrow each item to change it
for (row, &color) in ROW_COLORS.iter().enumerate() { } // with an index
while window.is_open() { }           // the game loop
loop { /* ... */ break; }            // forever, until `break` (see draw_line)
```

### Closures

A **closure** is a small function without a name, written `|args| body`.
It can use variables from around it. From `Color::lerp`:

```rust
pub fn lerp(self, other: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
    Color::rgb(
        mix(self.r, other.r),
        mix(self.g, other.g),
        mix(self.b, other.b),
    )
}
```

`mix` uses `t` from the surrounding function without it being passed in.

### Iterators

Iterators let you describe *what* you want from a collection instead of
writing loops by hand. `Canvas::text_width` finds the length of the longest
line of text:

```rust
let longest = text
    .lines()                          // each line of the text
    .map(|line| line.chars().count()) // ...turned into its length
    .max()                            // the biggest one (an Option: text might be empty)
    .unwrap_or(0);                    // 0 if there were no lines
```

More iterator methods in the engine:

| Method                | What it does                                  | Where                         |
|-----------------------|-----------------------------------------------|-------------------------------|
| `.position(\|x\| ...)` | Index of the first match, as an `Option`      | `break_brick_under_ball`      |
| `.filter_map(f)`      | Transform each item, dropping the `None`s     | reading keys in `run`         |
| `.enumerate()`        | Pairs each item with its index                | `build_bricks`                |
| `.all(\|x\| ...)`      | Are all items true?                           | a canvas test                 |
| `.chain(other)`       | One iterator, then another                    | a font test                   |

Iterators are *lazy*: nothing happens until something like `max`, a `for`
loop or `collect` asks for items. They compile down to code as fast as a
hand-written loop.

`Vec::retain` is a handy relative. It keeps only the items that pass a
test. That's how dead particles are removed:

```rust
self.particles.retain(|particle| particle.life > 0.0);
```

## 8. Traits

A **trait** describes behavior that different types can share. It's like
an *interface* in other languages. The engine's most important trait is
`Game`, in `src/engine.rs`:

```rust
pub trait Game {
    fn update(&mut self, ctx: &mut Context);
    fn draw(&self, canvas: &mut Canvas);
}
```

Any type that implements these two methods is a game. From `examples/hello.rs`:

```rust
struct Hello {
    position: Vec2,
}

impl Game for Hello {
    fn update(&mut self, ctx: &mut Context) { /* ... */ }
    fn draw(&self, canvas: &mut Canvas) { /* ... */ }
}
```

### Generics: code that works for any `Game`

```rust
pub fn run<G: Game>(config: Config, mut game: G) -> Result<(), Box<dyn Error>>
```

`<G: Game>` reads as "for any type `G` that implements `Game`". The engine
doesn't know about `Hello` or `Breakout`, but it can run both. The compiler
creates a separate copy of `run` for each game type ("monomorphization"),
so calling `game.update(...)` is as fast as a normal function call.

A similar shortcut is `impl Trait` in a parameter:

```rust
pub(crate) fn begin_frame(&mut self, keys_down: impl IntoIterator<Item = Key>)
```

"Give me anything that can be turned into an iterator of `Key`s." The game
loop passes an iterator; the tests pass a plain array like `[Key::Space]`.
Both work.

`Canvas::write_bmp` does the same with `out: &mut impl Write`. When saving
a screenshot, it writes to a file. In the test, it writes to a `Vec<u8>` in
memory, so the test can check the bytes without touching the disk.

### Operator overloading

Implementing standard library traits lets your types work with operators.
Because `src/math.rs` implements `Add` for `Vec2`:

```rust
impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        vec2(self.x + rhs.x, self.y + rhs.y)
    }
}
```

you can write `a + b` with vectors. `Sub`, `Mul<f32>`, `Neg`, `AddAssign` and
`SubAssign` give us `-`, `* 2.0`, unary `-` and `+=`/`-=`. That's what makes
this line in `hello.rs` read like the math it is:

```rust
self.position += direction * SPEED * ctx.dt();
```

`type Output = Vec2;` is an *associated type*: it says what `a + b`
produces.

### `Default`

`Config` implements `Default` by hand, which lets examples write
`..Config::default()`:

```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            title: String::from("tiny_engine"),
            width: 320,
            height: 240,
            scale: 3,
            target_fps: 60,
        }
    }
}
```

### `derive`: free trait implementations

For common traits, the compiler can write the implementation for you:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key { /* ... */ }
```

| Trait              | Gives you                                              |
|--------------------|--------------------------------------------------------|
| `Debug`            | Printing with `{:?}`, e.g. `println!("{:?}", key)`     |
| `Clone`            | An explicit `.clone()` method                          |
| `Copy`             | Implicit copying instead of moving ([chapter 6](#copy-types)) |
| `PartialEq`, `Eq`  | Comparing with `==`                                    |
| `Hash`             | Can be stored in a `HashSet` or used as a `HashMap` key |
| `Default`          | A `::default()` value with every field zeroed/empty    |

`Key` needs `Hash` and `Eq` because `Input` keeps keys in a `HashSet`.
`Vec2` derives `PartialEq` but not `Eq`, because floats can be `NaN`,
which isn't equal to itself.

### Dynamic dispatch (for later)

`run<G: Game>` picks the game type at compile time. To pick at runtime, say
to hold a list of different scenes, you'd use a *trait object* like
`Box<dyn Game>`. The standard library does this for errors: we return
`Box<dyn Error>`, which can hold any type of error. You won't need your own
trait objects for a while.

## 9. Modules and visibility

`src/lib.rs` declares the modules. Each `mod name;` line loads `src/name.rs`:

```rust
pub mod canvas;
pub mod color;
pub mod engine;
pub mod input;
pub mod math;
pub mod rng;

// Not `pub`: the font is an internal detail of `Canvas::draw_text`.
mod font;
```

**Everything is private by default.** You choose what to share:

| Written as     | Visible to                                           | Example                        |
|----------------|------------------------------------------------------|--------------------------------|
| (nothing)      | The current module (and modules inside it)           | `Canvas::index`, `mod font`    |
| `pub(crate)`   | Anywhere inside this crate, but not to games using it | `Context::begin_frame`         |
| `pub`          | Everyone                                             | `Canvas::fill_rect`            |

`Context::begin_frame` is `pub(crate)` because only the engine's loop should
advance the clock. A game calling it would mess up its own timing, so we
make that impossible.

Inside the crate, modules refer to each other with `crate::`:

```rust
use crate::color::Color;
use crate::math::{Rect, Vec2, vec2};
```

### The prelude pattern

Games would need a long list of `use` lines without this, at the bottom of
`lib.rs`:

```rust
pub mod prelude {
    pub use crate::canvas::Canvas;
    pub use crate::color::Color;
    pub use crate::engine::{Config, Context, Game};
    pub use crate::input::{Input, Key};
    pub use crate::math::{Rect, Vec2, vec2};
    pub use crate::rng::Rng;
}
```

`pub use` re-exports a name, so `use tiny_engine::prelude::*;` brings all of
them in at once. Many Rust libraries do this.

## 10. Error handling

Rust has two kinds of errors.

**Unrecoverable errors: `panic!`.** A bug that should never happen, like an
out-of-range index, stops the program with a message. Integer overflow
panics too in debug builds. So does `.unwrap()` on a `None`.

**Recoverable errors: `Result`.** When something can fail for reasons
outside your control (no display, disk full), a function returns:

```rust
enum Result<T, E> {
    Ok(T),  // it worked, here's the value
    Err(E), // it failed, here's why
}
```

### The `?` operator

Opening a window can fail. In `run`:

```rust
let mut window = Window::new(
    &config.title,
    config.width * config.scale,
    config.height * config.scale,
    options,
)?;
```

That `?` at the end means: *if this is an `Err`, return the error from
this function right now; otherwise give me the value inside the `Ok`.* It
replaces a whole `match`. That only works because `run` returns a `Result`
too:

```rust
pub fn run<G: Game>(config: Config, mut game: G) -> Result<(), Box<dyn Error>>
```

`Result<(), ...>` means "nothing useful on success, just that it worked".
`()` is the "empty" type. And `main` can return a `Result` too, so every
example ends like this:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ...
    tiny_engine::run(config, game)
}
```

If `run` fails, the program exits and prints the error.

`Canvas::save_bmp` is a chain of `?`s, one per write:

```rust
pub fn save_bmp(&self, path: impl AsRef<Path>) -> io::Result<()> {
    let file = File::create(path)?;
    let mut out = BufWriter::new(file);
    self.write_bmp(&mut out)?;
    out.flush()
}
```

### Handling an error instead of passing it on

A failed screenshot shouldn't crash the game, so `save_screenshot` deals
with the error itself:

```rust
match canvas.save_bmp(&path) {
    Ok(()) => println!("Saved {path}"),
    Err(err) => eprintln!("Could not save {path}: {err}"),
}
```

### Design choice: drawing never fails

Drawing off the edge of the canvas is *normal* in games (a particle flies
off-screen, text is too wide). So the canvas quietly clips instead of
panicking or returning a `Result`. There's a test that proves it:

```rust
#[test]
fn drawing_off_canvas_does_not_panic() {
    let mut canvas = Canvas::new(10, 10);
    canvas.set_pixel(-5, 100, Color::WHITE);
    canvas.fill_rect(Rect::new(-50.0, -50.0, 20.0, 20.0), Color::WHITE);
    // ...
}
```

Deciding what's a bug (panic), what's expected failure (`Result`) and
what's not an error at all (clip it) is a big part of designing Rust APIs.

## 11. Tests

Tests live next to the code they test, at the bottom of each file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressed_is_true_for_one_frame_only() {
        let mut input = Input::new();

        input.begin_frame([Key::Space]);
        assert!(input.is_down(Key::Space));
        assert!(input.was_pressed(Key::Space));

        input.begin_frame([Key::Space]);
        assert!(input.is_down(Key::Space));
        assert!(!input.was_pressed(Key::Space));
        // ...
    }
}
```

- `#[cfg(test)]` means "only compile this when testing".
- `use super::*;` imports everything from the parent module. Tests can use
  private items like `begin_frame`, since they're inside the module.
- `#[test]` marks a test function. `assert!(condition)` and
  `assert_eq!(left, right)` fail the test if they're not satisfied.

Run them all with `cargo test`. The code example in the doc comment at the
top of `src/lib.rs` is compiled as a test too (a *doc test*), so the
documentation can't go out of date.

Game logic is hard to test by playing, and easy to test with code. Whenever
you fix a bug, try writing a test that would have caught it.

---

# Part 2: How the engine works

## 12. The big picture

```text
   examples/breakout.rs      (your game: implements `Game`, calls `run`)
            |
            v
+----------------------- tiny_engine ------------------------+
|  engine.rs   run(): the game loop, Context, Config         |
|              <- the ONLY file that knows minifb exists     |
|                                                            |
|  canvas.rs   pixels + drawing  ---uses--->  font.rs        |
|  input.rs    which keys are down / pressed / released      |
|  math.rs     Vec2, Rect         color.rs    rng.rs         |
+------------------------------------------------------------+
            |
            v
        minifb     (opens the window, shows pixels, reports keys)
            |
            v
     Windows / macOS / Linux
```

Three design decisions shape everything:

1. **A trait is the contract between engine and game.** The engine calls
   `update` and `draw`; the game never has to know how windows work.
2. **Only one file touches the window library.** Games use our own `Key`
   type, not minifb's. To switch to another library (like SDL2 or winit),
   you'd rewrite `engine.rs` and nothing else.
3. **The engine draws every pixel itself.** No GPU and no graphics API,
   just a `Vec<u32>`. That's slower than a GPU, but a 320x240 screen is only
   76,800 pixels and modern CPUs don't break a sweat. And you can
   understand every line.

## 13. The game loop

Every game engine, from this one to Unreal, is built around a loop that
runs once per frame. Here's ours, from `run` in `src/engine.rs`:

```rust
while window.is_open() && !ctx.quit_requested {
    // 1. Time.
    let now = Instant::now();
    let dt = now.duration_since(last_frame).as_secs_f32().min(MAX_DT);
    last_frame = now;

    // 2. Input: translate the window library's keys into our own `Key`s.
    let keys = window.get_keys().into_iter().filter_map(convert_key);
    ctx.begin_frame(dt, keys);

    // 3 + 4. The game's turn.
    game.update(&mut ctx);
    game.draw(&mut canvas);

    if ctx.input.was_pressed(Key::F12) {
        save_screenshot(&canvas);
    }

    // 5. Show the frame. This also collects new keyboard events from the OS.
    window.update_with_buffer(canvas.pixels(), canvas.width(), canvas.height())?;
}
```

At 60 frames per second, this whole thing runs every 16.7 milliseconds.

### Delta time (`dt`): frame-rate independence

`dt` is how many seconds passed since the last frame. Games multiply every
speed by it:

```rust
self.position += direction * SPEED * ctx.dt();
```

Why? Say `SPEED` is 120 pixels per second:

| Frame rate | `dt`   | Moved per frame | Moved per second |
|------------|--------|-----------------|------------------|
| 60 fps     | 1/60 s | 2 pixels        | 120 pixels       |
| 30 fps     | 1/30 s | 4 pixels        | 120 pixels       |

Without `dt`, the game would run twice as fast on a computer that draws
twice as many frames. (Some very old games really did this!)

### Capping `dt`

```rust
const MAX_DT: f32 = 1.0 / 30.0;
```

If the game freezes for half a second (you're dragging the window, say),
the next `dt` would be 0.5, and a ball moving 280 px/s would jump 140
pixels in one step, straight through the paddle. This is called
**tunneling**. Capping `dt` means the game briefly runs in slow motion
instead, which players barely notice.

### Frame pacing

`window.set_target_fps(60)` makes `update_with_buffer` wait a little each
frame, so the loop doesn't run thousands of times per second and burn a
whole CPU core for nothing.

> **Going further:** many engines use a *fixed timestep*: they always update
> in exact steps of, say, 1/60 s, running 0, 1 or 2 updates per drawn
> frame. That makes physics perfectly repeatable. It's a nice exercise;
> search for "Fix Your Timestep" by Glenn Fiedler.

## 14. Pixels and the framebuffer

The canvas is a single flat `Vec<u32>` with one number per pixel, stored row
after row:

```text
          x=0  x=1  x=2  x=3
   y=0  [  0 ][  1 ][  2 ][  3 ]
   y=1  [  4 ][  5 ][  6 ][  7 ]     index = y * width + x
   y=2  [  8 ][  9 ][ 10 ][ 11 ]     so (2, 1) is at 1 * 4 + 2 = 6

   in memory: [0 1 2 3 | 4 5 6 7 | 8 9 10 11]
```

Each `u32` packs a color as `0x00RRGGBB`. `Color::to_u32` builds it with bit
shifts:

```rust
pub const fn to_u32(self) -> u32 {
    ((self.r as u32) << 16) | ((self.g as u32) << 8) | self.b as u32
}
```

```text
 bits:  00000000 RRRRRRRR GGGGGGGG BBBBBBBB
                 ^ r << 16 ^ g << 8 ^ b
```

`<<` shifts bits left, and `|` combines them.

**Scaling:** the canvas is 320x240 but the window opens at 960x720 (`scale:
3`). minifb stretches the small image to fill the window, which gives the
chunky pixel-art look for free. The window keeps the right shape if you
resize it (`ScaleMode::AspectRatioStretch`).

## 15. Drawing shapes and text

All drawing is in `src/canvas.rs`. Every function follows one rule: **never
write outside the pixel buffer**. Indexing out of range would panic.

### Rectangles

```rust
pub fn fill_rect(&mut self, rect: Rect, color: Color) {
    // Round to whole pixels, then clamp to the canvas so we never index out of bounds.
    let clamp_x = |v: f32| (v.round() as i32).clamp(0, self.width as i32) as usize;
    let clamp_y = |v: f32| (v.round() as i32).clamp(0, self.height as i32) as usize;
    let (x0, x1) = (clamp_x(rect.left()), clamp_x(rect.right()));
    let (y0, y1) = (clamp_y(rect.top()), clamp_y(rect.bottom()));
    if x0 >= x1 || y0 >= y1 {
        return; // nothing visible
    }

    let c = color.to_u32();
    for y in y0..y1 {
        let row = y * self.width;
        self.pixels[row + x0..row + x1].fill(c);
    }
}
```

Clamping the corners to the canvas is called **clipping**. Then each row of
the rectangle is one contiguous stretch of memory, so we fill it as a
*slice* in one go. That's much faster than setting pixels one at a time.
This is the most-used function in the engine: the font is drawn with it,
too.

### Circles

For every pixel in the circle's bounding box, check if the pixel's center
is within `radius` of the circle's center (Pythagoras: `dx² + dy² <= r²`):

```rust
let dx = x as f32 + 0.5 - center.x;
let dy = y as f32 + 0.5 - center.y;
if dx * dx + dy * dy <= r2 {
    self.set_pixel(x, y, color);
}
```

Comparing squared distances avoids a slow square root.

### Lines

`draw_line` uses **Bresenham's algorithm** (1962!). It walks from one end
to the other, one pixel at a time, keeping an `error` counter of how far
the drawn pixels have drifted from the true line. When the error gets too
big it takes a step sideways. It uses only integer addition, which mattered
a lot on 1960s hardware.

### Text

`src/font.rs` stores each character as five rows of three bits:

```rust
'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
```

```text
0b010   . # .
0b101   # . #
0b111   # # #
0b101   # . #
0b101   # . #
```

`is_lit` checks one bit with a shift and a mask:

```rust
pub fn is_lit(glyph: &Glyph, col: usize, row: usize) -> bool {
    let bit = GLYPH_WIDTH - 1 - col;
    (glyph[row] >> bit) & 1 == 1
}
```

`draw_text` loops over the characters, and for each lit bit draws a
`scale` x `scale` square with `fill_rect`. That's all a bitmap font is.

### Screenshots

Press **F12** in any game to save a `.bmp` file. BMP is one of the simplest
image formats: a 54-byte header describing the size, then the raw pixels.
`write_bmp` writes each number with `to_le_bytes()` ("little-endian": least
significant byte first), which is what the format requires. Our
`0x00RRGGBB` pixels, written little-endian, come out as the bytes B, G, R,
0, which is exactly BMP's order.

## 16. Keyboard input

Games ask two different questions about keys:

- **"Is it held down?"** For continuous things like moving. → `is_down`
- **"Was it *just* pressed?"** For one-shot things like launching the ball.
  If you used `is_down` for that, holding Space for half a second would
  launch 30 times. → `was_pressed`

The OS only tells us which keys are down *right now*. To detect presses,
`Input` also remembers last frame's keys and compares:

```rust
pub fn was_pressed(&self, key: Key) -> bool {
    self.down.contains(&key) && !self.down_last_frame.contains(&key)
}
```

Here's what happens when you tap Space:

| Frame | Your finger         | `down`    | `down_last_frame` | `is_down` | `was_pressed` | `was_released` |
|-------|---------------------|-----------|-------------------|-----------|---------------|----------------|
| 1     | not touching        | {}        | {}                | false     | false         | false          |
| 2     | presses Space       | {Space}   | {}                | true      | **true**      | false          |
| 3     | still holding       | {Space}   | {Space}           | true      | false         | false          |
| 4     | lets go             | {}        | {Space}           | false     | false         | **true**       |

`axis(negative, positive)` is a convenience that turns two keys into a
number between -1.0 and 1.0, which makes movement code short:

```rust
let direction = ctx.input.axis(Key::Left, Key::Right) + ctx.input.axis(Key::A, Key::D);
```

## 17. Random numbers

Rust's standard library has no random number generator (the popular `rand`
crate fills that gap), so `src/rng.rs` implements **xorshift32**, a
famously tiny algorithm:

```rust
pub fn next_u32(&mut self) -> u32 {
    let mut x = self.state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    self.state = x;
    x
}
```

Three shift-and-XOR steps scramble the bits into a sequence that *looks*
random. It isn't really random: the same **seed** always gives the same
sequence. That's useful for tests (`Rng::new(42)`), for replays and for
debugging. The engine seeds it from the clock so each game is different.

Other methods build on `next_u32`: `next_f32` gives `0.0..1.0`, `range`
scales that to any range, and `chance(0.25)` is true 25% of the time.

## 18. Breakout, piece by piece

Now let's see how a real game uses all of this. Open `examples/breakout.rs`.

### A state machine

The `State` enum and the `match` in `update` form a **state machine**, the
most useful pattern in game programming:

```text
                    ball lost (lives left)
               +--------------------------+
               v                          |
  start --> Serving ----- Space -----> Playing <--- P ---> Paused
               ^                         |   |
               |             ball lost   |   |  last brick
               |         (no lives left) v   v  broken
               |                  GameOver   Won
               |                         |   |
               +---- Space or Enter -----+---+
```

Each state decides what input means and what moves. In `Serving` the
arrows move the paddle and the ball sits on it. In `Paused` nothing moves
except P. Menus, cutscenes, enemy AI: they're all state machines.

### Collisions: rectangle overlap

Everything in Breakout is a `Rect`, and `Rect::overlaps` is the whole
collision system:

```rust
pub fn overlaps(&self, other: &Rect) -> bool {
    self.left() < other.right()
        && self.right() > other.left()
        && self.top() < other.bottom()
        && self.bottom() > other.top()
}
```

Two boxes overlap unless one is entirely to the left, right, above or below
the other. This is called **AABB** (axis-aligned bounding box) collision.

### Which way to bounce?

Knowing the ball hit a brick isn't enough; we need to know *which side* it
hit, to know whether to flip `x` or `y` velocity. The trick is to **move one
axis at a time**:

```rust
let step = self.ball_velocity.x * dt;
self.ball.x += step;
let hit_wall = self.ball.left() < 0.0 || self.ball.right() > WIDTH;
if hit_wall || self.break_brick_under_ball(&mut ctx.rng) {
    self.ball.x -= step; // undo the move...
    self.ball_velocity.x = -self.ball_velocity.x; // ...and bounce
}

let step = self.ball_velocity.y * dt;
self.ball.y += step;
// ... the same for y ...
```

If moving *sideways* caused a hit, it was a side hit: undo the move and
flip `x`. Then do the same for `y`. Notice `hit_wall || ...`: `||` stops
early, so if the ball hit a wall we don't even check the bricks.

### Aiming with the paddle

If the ball always bounced straight back, you couldn't aim. So the bounce
angle depends on where the ball lands on the paddle:

```rust
// -1.0 = left edge, 0.0 = middle, 1.0 = right edge.
let offset = (self.ball.center().x - self.paddle.center().x) / (PADDLE_WIDTH / 2.0);
let angle = offset.clamp(-1.0, 1.0) * MAX_BOUNCE_ANGLE;
self.ball_velocity = direction_from_angle(angle) * self.ball_speed;
```

```text
   offset: -1.0            0.0             +1.0
              \             |             /
               \            |            /      bounce angle grows with
                \           |           /       distance from the middle
          [=============== paddle ===============]
```

`direction_from_angle` uses a little trigonometry: `sin(angle)` gives the
sideways part and `-cos(angle)` the upward part (negative because screen `y`
grows *downward*). The result is a vector of length 1, which we multiply by
the speed.

The paddle only bounces the ball while `ball_velocity.y > 0.0` (moving
down). Without that check, a ball that clipped the paddle's side could stay
overlapping it and bounce back and forth every frame, stuck inside.

### Particles

When a brick breaks, 12 particles fly out in random directions:

```rust
let angle = rng.range(0.0, TAU); // TAU = 2π = a full circle
let speed = rng.range(30.0, 120.0);
self.particles.push(Particle {
    position: at,
    velocity: vec2(angle.cos(), angle.sin()) * speed,
    life: rng.range(0.3, PARTICLE_LIFE),
    color,
});
```

Each frame, gravity pulls them down, they move, and they age. Dead ones are
dropped with `retain`. When drawn, `Color::lerp` fades each one into the
background color as its life runs out. Particles don't change gameplay at
all, but they make it feel much better. Game developers call this
"juice".

### Drawing order is layering

`draw` paints back to front: background, bricks, particles, paddle, ball,
then text on top. Later drawing covers earlier drawing, like painting on a
canvas (it's called the *painter's algorithm*).

### Restarting in one line

```rust
*self = Breakout::new();
```

`self` is a `&mut Breakout`, a reference. `*self` means "the thing it
points to", so this replaces the entire game with a fresh one. The old
bricks, particles and score are freed automatically.

---

# Part 3: Your turn

## 19. Exercises

Do them in order. After each change, run the game and see what happened.
When the compiler complains, **read the whole error message**: Rust's error
messages are unusually helpful, and often include the fix.

### Warm-up: turn the knobs (`examples/breakout.rs`)

1. Make the paddle twice as wide. Then make the ball twice as fast.
2. Set `GRAVITY` to `-300.0`. What happens to the particles?
3. Give the player 5 lives.
4. Add a seventh row of bricks by adding a color to `ROW_COLORS`. The
   compiler will stop you:

   ```text
   error[E0308]: mismatched types
     expected an array with a size of 6, found one with a size of 7
   ```

   An array's length is part of its type. Fix the `6`.

### Small features

5. **High score.** Keep the best score across restarts and show it in the
   HUD. Careful: `*self = Breakout::new()` wipes everything. You'll need
   to save the high score in a variable, restart, then put it back.
6. **Tough bricks.** Give `Brick` a `hits_left: u32` field. Top-row bricks
   need two hits. Draw damaged bricks darker (`color.lerp(Color::BLACK, 0.4)`).
7. **Hello, in color.** In `examples/hello.rs`, make the square change color
   while Space is held. Then make it leave a trail: store the last 20
   positions in a `Vec<Vec2>` and draw them fading out.
8. **A new key.** Add `Key::M` to the engine: add it to the `Key` enum in
   `src/input.rs` and to `convert_key` in `src/engine.rs`. Use it to toggle
   something in a game.

### Engine features

9. **Screen shake.** When a brick breaks, shake the screen for 0.2 seconds.
   Hint: add a `camera_offset: Vec2` field to `Canvas` that every drawing
   function adds to positions, and set it randomly each frame while shaking.
10. **Mouse support.** minifb has `window.get_mouse_pos(MouseMode::Clamp)`.
    Add a `mouse_position: Vec2` to `Input` (remember the canvas is scaled!)
    and let Breakout's paddle follow the mouse.
11. **Sprites.** Draw small pictures stored in the code, like the font:
    ```rust
    const INVADER: [&str; 4] = [
        "..#..#..",
        ".######.",
        "##.##.##",
        "#.#..#.#",
    ];
    ```
    Write `Canvas::draw_sprite(&mut self, sprite: &[&str], pos: Vec2, scale: u32, color: Color)`.
12. **Fixed timestep.** Change `run` to update in fixed steps of 1/60 s.
    Watch out: what should `was_pressed` do if two updates run in one frame?

### New games

Start by copying this skeleton to `examples/mygame.rs`, then run it with
`cargo run --example mygame`:

```rust
use tiny_engine::prelude::*;

struct MyGame {
    // your game's state goes here
}

impl Game for MyGame {
    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.was_pressed(Key::Escape) {
            ctx.quit();
        }
        // move things, check collisions...
    }

    fn draw(&self, canvas: &mut Canvas) {
        canvas.clear(Color::BLACK);
        // draw things...
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tiny_engine::run(Config::default(), MyGame {})
}
```

13. **Pong.** Two paddles (W/S and Up/Down), one ball, a score for each
    side. You can reuse a lot of Breakout.
14. **Snake.** Use a grid: 16x16 pixel cells give a 20x15 board. The snake
    is a `VecDeque<(i32, i32)>` (from `std::collections`): push a new head,
    pop the tail. Move only every 0.1 s by adding up `dt` in a timer. Place
    food with `ctx.rng.range_i32(0, 20)`. Use an enum for the direction.
15. **Space invaders.** A ship, bullets in a `Vec`, rows of aliens that
    march sideways and step down. Uses everything in this guide.

## 20. Common compiler errors

| Error    | Message (short)                                   | Usually means                                                 |
|----------|---------------------------------------------------|---------------------------------------------------------------|
| `E0384`  | cannot assign twice to immutable variable          | You forgot `mut`.                                             |
| `E0308`  | mismatched types                                   | Often `f32` vs `usize` vs `i32`: add an `as` conversion.       |
| `E0382`  | borrow of moved value                              | You used a value after moving it. Borrow with `&`, or `.clone()`. |
| `E0502`  | cannot borrow as mutable because it is also borrowed as immutable | Reading and changing at once. Use "find first, change second". |
| `E0499`  | cannot borrow as mutable more than once at a time  | Two `&mut` to the same thing. Same fix as above.              |
| `E0004`  | non-exhaustive patterns                            | A `match` is missing a case. Handle it, or add `_ =>`.        |
| `E0425`  | cannot find value in this scope                    | A typo, or a missing `use`.                                   |
| `E0599`  | no method named `...` found                        | A typo, a missing `use` for a trait, or the wrong type.       |

For a long explanation of any error, run `rustc --explain E0502`.

## 21. Where to go next

- **The Rust Programming Language** ("the Book"), free at
  <https://doc.rust-lang.org/book/>. The best way to learn Rust properly.
  You've already seen most of chapters 1 to 11!
- **Rustlings**: <https://github.com/rust-lang/rustlings>. Small exercises
  that you fix until they compile.
- **Rust by Example**: <https://doc.rust-lang.org/rust-by-example/>.
- **`cargo doc --open`** in this project shows the engine's documentation,
  with links to the standard library.
- **Real Rust game engines**, now that you know what's under the hood:
  - [macroquad](https://macroquad.rs): simple, and similar in spirit to this engine, but GPU-powered.
  - [Bevy](https://bevyengine.org): a big, modern engine built around an *Entity Component System*.

Have fun, and break things. The compiler has your back.

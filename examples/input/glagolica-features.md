---
title: Glagolica Flavor Features
author: Glagolica
version: 1.0.0
---

# Glagolica Markdown Flavor

This document demonstrates all extended markdown features supported by Glagolica.

## Alert Blockquotes

GitHub-style alert callouts for important information:

> [!NOTE]
> Highlights information that users should take into account, even when skimming.
> This is a multi-line note with **bold** and _italic_ text.

> [!TIP]
> Optional information to help a user be more successful.
> Use tips for shortcuts and best practices.

> [!IMPORTANT]
> Crucial information necessary for users to succeed.
> Don't skip this section!

> [!WARNING]
> Critical content demanding immediate user attention due to potential risks.
> Proceed with caution when modifying configuration files.

> [!CAUTION]
> Negative potential consequences of an action.
> This operation cannot be undone. Make sure to backup your data first.

## Table of Contents

Use `<toc>` to insert an auto-generated table of contents:

<toc />

## Step-by-Step Guides

<steps>
  <step>
    ### Install Dependencies

    First, make sure you have Rust installed:

    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

    Verify the installation:

    ```bash
    rustc --version
    cargo --version
    ```

  </step>
  <step>
    ### Clone the Repository

    Clone the Bukvar repository from GitHub:

    ```bash
    git clone https://github.com/glagolica/bukvar.git
    cd bukvar
    ```

  </step>
  <step>
    ### Build the Project

    Build in release mode for optimal performance:

    ```bash
    cargo build --release
    ```

    The binary will be available at `target/release/bukvar`.

  </step>
  <step>
    ### Run Your First Parse

    Parse a markdown file:

    ```bash
    ./target/release/bukvar parse examples/input/readme.md
    ```

    You should see JSON output of the parsed AST.

  </step>
</steps>

## Tabbed Code Blocks

Show the same concept in multiple languages:

<tabs names="JavaScript, Python, Rust, Go">
  ```js
  // JavaScript
  function greet(name) {
    console.log(`Hello, ${name}!`);
  }

greet("World");

````

```python
# Python
def greet(name):
    print(f"Hello, {name}!")

greet("World")
````

```rust
// Rust
fn greet(name: &str) {
    println!("Hello, {}!", name);
}

fn main() {
    greet("World");
}
```

```go
// Go
package main

import "fmt"

func greet(name string) {
    fmt.Printf("Hello, %s!\n", name)
}

func main() {
    greet("World")
}
```

</tabs>

## Code Block Highlighting

Highlight specific lines to draw attention:

```rust highlight="3, 7-9"
use std::collections::HashMap;

fn main() {
    let mut scores = HashMap::new();

    // These lines are highlighted
    scores.insert("Blue", 10);
    scores.insert("Yellow", 50);
    scores.insert("Red", 25);

    for (key, value) in &scores {
        println!("{}: {}", key, value);
    }
}
```

## Code Diffs

Show changes between versions with diff highlighting:

```python plusdiff="6-8" minusdiff="4-5"
def calculate_total(items):
    total = 0
    for item in items:
        # Old implementation (removed)
        total = total + item.price
        # New implementation (added)
        discount = item.get_discount()
        total += item.price * (1 - discount)
    return total
```

Another example showing a bug fix:

```javascript plusdiff="3" minusdiff="2"
function divide(a, b) {
  return a / b;
  if (b === 0) throw new Error("Division by zero");
  return a / b;
}
```

## Line Numbers

Add line numbers for easier reference:

```kt linenumbers
package com.example

fun main() {
    val message = "Hello, Kotlin!"
    println(message)

    val numbers = listOf(1, 2, 3, 4, 5)
    numbers
        .filter { it % 2 == 0 }
        .map { it * 2 }
        .forEach { println(it) }
}
```

## Combined Attributes

You can combine multiple attributes:

```typescript highlight="5-7" linenumbers
interface User {
  id: number;
  name: string;
  email: string;
  // Highlighted section
  preferences: {
    theme: "light" | "dark";
    notifications: boolean;
  };
}

function createUser(name: string, email: string): User {
  return {
    id: Date.now(),
    name,
    email,
    preferences: {
      theme: "light",
      notifications: true,
    },
  };
}
```

## Standard GFM Features

### Tables

| Feature           | Status | Notes                 |
| ----------------- | :----: | --------------------- |
| Alerts            |   ✅   | All 5 types supported |
| Steps             |   ✅   | Nested markdown works |
| Tabs              |   ✅   | With named tabs       |
| Line Highlighting |   ✅   | Single and ranges     |
| Diffs             |   ✅   | Plus and minus        |
| Line Numbers      |   ✅   | Boolean attribute     |

### Task Lists

- [x] Implement alert blockquotes
- [x] Add steps container
- [x] Create tabs element
- [x] Support code highlighting
- [x] Add diff syntax
- [x] Enable line numbers
- [ ] Add more examples
- [ ] Write documentation

### Math (if supported)

Inline math: $E = mc^2$

Block math:

$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$

### Footnotes

Glagolica supports extended markdown[^1] with many useful features[^2].

[^1]: Extended markdown includes GFM and custom Glagolica extensions.
[^2]: See the full documentation for all supported features.

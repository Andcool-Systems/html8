# HTML8

**HTML8** is a high-level compiled programming language with syntax inspired by HTML. It combines a declarative style with the capabilities of imperative programming.

---

## 📄 Program Structure

Every HTML8 program has the following basic structure:

```xml
<html>
    <head>
        <head-code />
    </head>
    <main>
        <main-program-code />
    </main>
</html>
```

- The `<main>` block is **required** and serves as the **entry point** of the program.
- The `<head>` block is optional and is used for importing modules, declaring constants, and preparatory code.

---

## 📦 Scopes

HTML8 uses block scoping:

- Variables and functions declared in `<main>` are visible to all its nested blocks.
- Objects created in nested blocks are **not visible** to their parent blocks.
- Once a block ends, all objects created within it are destroyed.
- For empty blocks, `<div></div>` can be used.

---

## 🧠 Variables

HTML8 is a **statically typed** language, so variable types must be known at compile time.

### 🔹 Declaration

```xml
<int name="my_var">12</int>
```

Creates a variable `my_var` of type `int` with value `12`. Initialization with another variable is also allowed.

### 🔹 Assignment

```xml
<my_var>16</my_var>
```

Updates the value of the variable. The variable's name is used as the tag name.

### 🔹 Arithmetic Expressions

```xml
<my_var>2 * 2 + 4</my_var>
```

Arithmetic expressions are allowed. The compiler may perform constant folding.

---

## 🛠️ Functions

Functions are declared similarly to variables, with the ability to specify arguments and return values.

```xml
<int name="my_func" arg1="int">
    <function-body />
    <return {result} />
</int>
```

- The function's return type (`int` in this example) is declared like a variable.
- Arguments are specified as attributes (`arg1="int"`).
- Return values are provided via `<return {value} />`.
- For `void` functions, `return` can be omitted.

### 🔹 Variables as Functions

Until a reserved keyword (like `return` or another function call) is used inside the block, the object is treated as a **computed variable**:

```xml
<!-- Variable -->
<int name="sum">1 + 2</int>

<!-- Function -->
<int name="show" arg="int">
    <println {arg} />
    <return {arg} />
</int>
```

---

## 🔁 Function Calls

Functions are called using self-closing tags with argument passing:

```xml
<func_name arg1={value1} arg2={value2} />
```

- Argument order doesn't matter.
- Function results can be stored in a variable:

```xml
<int name="result">
    <my_func arg={input} />
</int>
```

---

## 🔂 `for` Loops

HTML8 supports `for` loops with the following syntax:

```xml
<for i="i" start={0} end={10}>
    <body />
</for>
```

- The `i` attribute specifies the iterator's name.
- The iterator is automatically created within the loop's scope as an `int` with the name from the `i` attribute.
- The `start` and `end` attributes define the loop range (inclusive `start`, exclusive `end`).

Example:

```xml
<for i="x" start={0} end={5}>
    <println {x} />
</for>
```

---

## ✅ Example Program

```xml
<html>
    <head></head>
    <main>
        <int name="a">1</int>
        <int name="b">2</int>

        <int name="sum" arg="int">
            <return {arg + b} />
        </int>

        <int name="result">
            <sum arg={a} />
        </int>

        <println {result} />
    </main>
</html>
```

---

## 💡 Features

- HTML8 preserves the readability and structure of HTML while offering the power of a typed language.
- The HTML8 compiler optimizes code during compilation, including constant folding.
- Encourages pure functions and localized scope.
- **Compiles to the C programming language**, enabling high-performance executables and access to C’s mature ecosystem.

---

## 📌 In Development

Planned features:

- Conditional operators (`if`, `else`)
- `while` loops
- Boolean operations

---

**Created by AndcoolSystems under PEPSI Community, March 14, 2025**


# Riven Programming Language

## How to Use
The `riven` compiler is a simple frontend that compiles `.rvn` source files. It accepts various command-line options for debugging and execution. You can download the compiler executable from this repository in the `bin` folder. You can compile a source file using the commands below:
- `-i`, `--input <FILE>`  
  **(Required)**  
  Specifies the path to the input `.rvn` source file to compile.

- `-l`, `--debug-lex`  
  Enables **lexing debug output**, showing tokenization details.

- `-p`, `--debug-parse`  
  Enables **parsing debug output**, displaying the structure of the parsed code.

- `-v`, `--debug-validate`  
  Enables **validation debug output**, useful for type checking or semantic analysis.

- `-r`, `--run`  
  **Runs the program immediately after compiling**.

- `-h`, `--help`
  **Displays the help menu**

All output files are written inside of a `out` directory, that is created inside of the current working directory. The compiled lua is stored inside of that directory in a `out.lua` file. All debugging output files are stored in the same `out` directory.

## Basic Syntax:
```rs
// functions (implicit `Void` return type)
fn hello()
{
  println("Hello World!");
}

// member functions
fn Int.to_string(self) -> String
{
  return "" + self;
}

fn Bool.to_string(self) -> String
{
  if (self)
  {
    return "True";
  }
  else
  {
    return "False";
  }
}

// functions as variables
fn main()
{
  // variables
  let a = 5;
  let b = 7;
  let c = a + b;

  let bound = c.to_string; // using a member function, and binding it
  println(bound()); // "13"

  // lambda expressions
  let lambda = |a: Int, b: Int| -> String => // variable shadowing `a`, `b`
  {
    return (a + b).to_string();
  }

  println(lambda(1, 2)); // "3"

  let array = [5, 6, 7]; // array of intagers
  array[0] = -1;
  println(array[1].to_string()) // "6"

  // optionals
  let val: ?Int = null; // needs an explicit type name to infer null
  println(val.is_none().to_string()); // "False"
  val = 7 as ?Int;
  println(val.unwrap().to_string()); // "7"
}
```

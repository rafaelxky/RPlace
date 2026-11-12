# Introduction
- rplace is a preprocessing template language that allows you to write templates and place them over any text file
- its made with programming in mind, it uses markers that use comment simbols to be embedable in code files

# markers
- markers are used to call commands
- list:

- //-
- -//
- /*-
- -*/
- *///-

- any marker can be used interchangebly, pick the one that is better for the ocasion

# variables

## regular variables
- variable declaration $#var_name
- variables must be inside templates
- you can do $#var_name+suffix, here + is used to append something to the var name, + will be removed so the value and sufix will have no space in between 
- also support optional with $#var? or $#var?+ if no value is provided, it will be empty

## arrow variables
- right arrow var dec /*- $#var_name -> -*/ default,
creates a variable in place
- ex:
pub struct /*- $#struct_name -> -*/ Vec2 
- here struct_name is a variable that has Vec2 as a default 
- value, given a value, Vec2 will be replaced by it
- you can do /*- $#struct_name -> -*/ default+suffix to append to the variable

## variable options
- variable options allow you to modify variables
- they are marked with "\"
- ex: var=val\regex:
- can also be chained
- :: can be used for args
- ex: $#var\lua::"return args[1]..\" world\""
- supported:
- derive:
    - \regex -> allows regex
    - \def -> places def around match
    - \var -> places arrow variable around match
- template variables
    - \snakecase -> turns to snake case
    - \camelcase -> turns to camel case
    - \screaming -> turns to screaming case
    - \pascalcase -> turns to pascal case
    - \lua -> executes inline code, places the returned string

## array variables
- array variables can be used in for each loops
- ex: [(a,b,c),(d,f,g)]
- array variables can be nested
- ex: [([(a,b,c)],b,c),([(a),(b),(c)],b,c)]
- anything inside [] is an array and anything inside () is a struct
- ex: //- for a,b,c in array:
    $#a 
    $#b
    $#c
//- end 
- this will place the respective values of a, b and c
- you can assign names to the variables in the arrays
- ex: [(a=b, c=d, e=f),(g=h,i=j)]
- you can either have named or unamed variables but not both in the same parentesis
- [(a,b,c=d)] is not allowed
- array elements are accessed sequentially
- struct elements are accessed at the same time and can be named
- ex: //- for elem1,elem2 in array: 
- elem1 and elem2 are struct elements
- ex: //- for val in array:
- here val can be part of an array or first element of struct
- ex: //- place ex where arr = [a,(a,b),(name=valueA, name2=valueB), b, [a,b,c]]
- this is an example mix of structs and arrays


## parent variables
- you can use parent variables inside place
- ex: //- place name where a=$#var:
- in this case "a" will be replaced by the value of "var" from the parent

# def
## define templates
- define template
/*- def def_name:
    template here $#var
*///- end:

## def place / inheritance
inheritance is possible with def place
//- def a place b where c=d:
//- place a:
here "a" will be placed with c=d, however, you can call
//- place a where c=e:
this will place with c=e so it allows overrides
also usefull to provide defaults

## template overload / when
- template overload is possible using the keyword when
- ex: //- def name when var=val:
- if var = val then this will be called
- ex: //- place name where var=val:
- it will trow if var is undefined
but you can define a default def with no parameters wich will be called instead
- ex: //- def name:
- if you call //- place name:
it will call the default def
- variables still work as normal
- you can use the var name inside the body
- ex:
//- def name when var=val:
    $#var
//- end:
- this will place the value in the variable as normal

- inheritance and overloading can be used at the same time like
- //- def a when lang=java place java_class where var=val:
- this will declare a template "a" wich will replace the variable "var" with "val" from "java_class" template if lang = java

## def defaults
- def accept defaults trough def where
- ex: 
//- def name where var=val:
    $#var
//- end:

## def derive
- you can create a reverse template and define it
- ex:
//- def derived derive examples/to_derive.txt where var="[Vv]ar"\regex:
- then you can place the result
- ex: //- place derived:

# file include
- //- include file_path:
- this will get the definitions from this other file
- works with .
- supports http includes, the url must provide raw text only

# place
- place is how you call your definitions with values assigned
- ex: //- place def_name where var_name=var_value, var2=val2:
- it suports single line and multiline double quote variables
- ex: //- place def_name where body = "
    multiline variable
//- ":
- if no value or default is provided the program will throw
- you can define defaults with def place or arrow variables

# place arguments
- you can pass arguments to place like so
- ex: //- place a where var = val:
- you can also use "double quotes" 
- ex: //- place a where var = "hello world":
- multiline variables are also allowed like so 
- //- place c where var = " 
    Hello
    World
    !
//-":

# data source
- rplace allows for file text data and http
- for http you must provide a file whose body is pure text similar to as if it was a file 

# inner macros
- when you call a macro, all inner macros will also execute
- inner def will make so that a macro is only defined after its parent is called, this is essentially runtime definitions
- inner place works as a normal place

# literal blocks
- Use a raw block to output RPlace syntax literally, such as when generating one RPlace template from another.
- Raw content is copied without expanding variables or processing directives. Put outer-template interpolation outside the raw block.
- Raw blocks are not nested. A missing `endraw` is an error.
- Fuzzy syntax can repair raw-block delimiters when enabled, and leaves the raw body unchanged.

```text
//- def generated_template:
//- raw:
//- def greeting:
Hello, $#name!
//- end:
//- endraw:
//- end:
```

Placing `generated_template` outputs the inner `greeting` definition and its `$#name` reference unchanged.

# create
- create allows you to create files and folders and place data inside it
- ex: //- create folder/file.txt place name:
- this will create "folder/file.txt" and place the "name" template inside
- if it starts with . then it will be in relation to the file

# reverse templates / derive
- rplace supports reverse templates
this will take regular text and transform it into a template acording to rules
- ex: //- derive path/file.txt where var="[Vv]ar"\regex:
- this will insert an arrow variable with name "var" before any match 
- also supports specific options
- \regex
- \def
- \var
- the default option will be \var and no regex
- you can also do it on the same file
- ex: //- derive where var="name":



# controll flow
## match
- inside of template bodies you can match variables
- match allows you to conditionally place blocks of code if the value matches the arm
- ex: 
//- match var:
    //- case valA
        stuff
    //- end: 
    //- case valB:
        more stuff
    //- end:
//- end:
- the first matched case will be placed
- can have inner instructions wich will be executed
- if the case value is _ then it will  match any or none

## for loops
- for loops allow you to iterate over an array
- integer ranges are also supported; `0..10` binds the loop variable to `0` through `9` (the end is exclusive)
- ex: `rplace run -e '//- for i in 0..10:$#i,//- end:'` prints `0,1,2,3,4,5,6,7,8,9,`
- either range bound can come from a template variable (`$#start`) or global (`$g#limit`), for example `//- for i in $#start..$g#limit:`
- ex:
//- for varA,varB in val:
    $#varA
    $#varB?
//- end:
- then called as such:
//- place template where val = [(a,b),(c,d),(e)]

# lua
## inline lua options
- you can use inline lua to manipulate variables
- ex: $#var\lua::"return args[1]..\" world \""
- Lua is provided by an optional runtime module rather than being linked into the RPlace executable.
- Build all packages with `cargo build --workspace --release`; `./build.sh` also installs the executable and runtime modules on Linux.
- On Linux, the default directory is `~/.local/share/rplace/addons`. Runtime modules are loaded from this directory.
- Set `allow_lua` to `true` in RPlace's `config.json` to enable Lua execution.

## regex derive option
- Regex matching is also provided by an optional runtime module.
- The workspace build compiles the regex module, and `./build.sh` installs it on Linux.
- The module is loaded when a derive declaration uses the `\regex` option.

# config
- the config.json has some program specifications
- allow_lua -> allows lua code (default: off)
- allow_import -> allows code import from other files(default: true)
- allow_fuzzy_syntax -> enables the optional fuzzy syntax plugin (default: off)
- allow_compile_commands -> allows project compile hooks to execute shell commands (default: off)

Project compile hooks are configured in `rplace.toml` under `[config]`. They run only for a normal project run, not inline evaluation, pipe mode, or parse-only mode:

```toml
[config]
before_compile_comand = "echo Starting compile"
after_compile_comand = "echo Finished compile"
```

Enable `"allow_compile_commands": true` in the global `config.json` to permit these commands. The before hook runs before project compilation, and the after hook runs after generated files are written. A nonzero command exit status fails the run.

# fuzzy syntax
- Fuzzy syntax is an optional, conservative repair pass that runs between lexing and parsing.
- It is disabled by default. To enable it, set `"allow_fuzzy_syntax": true` in the global `config.json`.
- Build the workspace plugin with `cargo build -p fuzzy_sintax` or build the workspace. The runtime searches for the plugin beside the executable and in the addons directory. If it is not installed, no fuzzy repairs are applied and normal parsing continues.
- Fuzzy syntax can recognize complete commands whose marker was omitted at the start of a line, and can supply a missing colon on a recognized command. For example:
```
def greeting:
    Hello, world!
end
```
  With fuzzy syntax enabled and the plugin available, the definition and its matching `end` are repaired so the template can be parsed.
- The pass recognizes command headers such as `def`, `for`, `match`/`case`, `end`, `place`, `include`, `create`, `derive`, `parse`, `mod`, `here`, and file-variable assignments. Repairs are limited to supported, structurally valid forms.
- Fuzzy syntax does not rewrite command-looking words in the middle of prose or inside quoted text. It does not invent missing arguments, fix malformed commands, or guess at an incomplete or unmatched block. Such input remains unchanged and may still produce a parser error.
- A marker on a malformed command does not make the command valid. When a repair is ambiguous, prefer writing the explicit marker and colon rather than relying on inference.

# file config global variables
- you can configure a file to change how the compiler interacts with it by setting variables 
- next up is a list of what you can do:
- //- $#output = "out.txt":
- Sets the output path for the file

# comand line arguments (args, cli)
- replace reload
- reloads the config file with the default values
- rplace run <origin> <target>
- executes the code at origin if target is provided writes at target if not, write direcly on origin
- target can be overriten by the file config variable $#output
- pass global template values from the shell with `-D KEY=VALUE` or `--var KEY=VALUE`
- you can pass the option multiple times, and shell values override values declared in the source
- use `$g#KEY` in the template to read a global value
- ex: `rplace run template.txt -D name=RPlace -D greeting="hello world"`
- in the template, `$g#greeting` will render as `hello world`
- render a short template directly from the command line with `-e` or `--eval`
- inline mode writes the rendered text to stdout by default; provide `<target>` to write it to a file
- use `-D KEY=VALUE` or `--var KEY=VALUE` to pass globals to an inline template
- ex: `rplace run -e 'Hello, $g#name!' -D name=Rafael`
- output: `Hello, Rafael!`
- write inline output to a file with `rplace run -e 'Hello, $g#name!' result.txt -D name=Rafael`
- global values beginning with `[` or `(` are parsed as arrays or structs, so templates can loop over them
- ex: `rplace run -e '/*- def list:/*- for item in items:$#item,/*- end:*///- end:*///- place list:' -D 'items=[red,green,blue]'`
- pass a struct with `-D 'record=(Rafael,admin)'`; iterate its fields with `/*- for name,role in record: ... *///- end:`
- loops over a global array or struct can also be written directly at the top level, outside a `def`, using `in $g#KEY`
- `--eval` cannot be combined with a source `<origin>` or `--parse-only`
- quote the template with single quotes so the shell does not expand `$g` before RPlace sees it
- global values use the first `=` as the key/value separator, so values may contain additional `=` characters
- replace new <project_name> 
- creates a new project config "rplace.toml" file
- rplace pipe <target> <text> 
- takes the target file and replaces the //- here: tag with the <text>

# piping / here command
- the here command provides a spot to place arguments passed by the CLI
- //- here:
- when you run rplace pipe <target> <text>
- <here> will be replaced by <text>
- this is meant to be used like so:
- rplace pipe file.txt "$(echo "hello world")"
- its meant for quick replacement

# parse instruction
- //- parse file.txt:
- runs the template on the designated file
- if it starts with . then the path will be in relation to the file
- paths cannot escape the project root

# paths
- paths cannot escape the project root or output root depending on the instruction
- if it starts with . then the path will be in relation to the file

# project config
- project config config is defined in the rplace.toml file
- here is defined the root file to be executed
- this allows you to simply run "rplace" to run the program

# package manager
- the rpalce package manager is a work in progress
- it allows you to fetch and use dependencies from the web
## cli 
### login
- package login <email> <password>
### create user 
-  package create-user <username> <email> <password>
- creates a new user
### push package
- package push
- pushes files to server
### new package
- package new
- uploads a new package from the rplace.toml version
### add package
- package add <package_name>
- adds a dependency to rplace.toml
### create version
- package create-version
- creates a new version, uses the version in rplace.toml

# list of examples
```
//- place:
//- def name:
//- def name where var=val:
//- def name when var = val:
//- def name where varA=valA when varB=valB:
//- end:
//- place name:
//- place name where varA=valA, varB=valB:
//- def nameA place nameB:
//- def nameA place nameB where var=val:
//- def nameA when varA=valA where varB=valB:
$#varname
$#varname+sufix
/*- $#varname -> -*/ defaultvar
/*- $#varname -> -*/ defaultvar+sufix
//- create file:
//- create folder/file.txt place name:
//- create folder/file.txt place name where var=val:
//- place name where var=$#var_paren:
//- derive path/file.txt where var="[Vv]ar"\regex, def="[Ss]truct"\def\regex:
//- def derived derive path/file.txt where var=val:
//- derive where var="name":
//- include path/file.txt:

//- match word:
    //- case apple
        apple
    //- end:
    //- case banana:
        //- place b:
    //- end:
//- end:

//- place def_name where body = "
    multiline variable
//- ":

$#var\snakecase
$#var\lua::"return args[1]..\" world \""

//- place c where var = " 
    Hello
    World
    !
//-":

//- place b where var = "Hello World!":
//- $#output = "out.txt":
//- parse file.txt:
//- parse ./file.txt:
//- for varA,varB in val:
    $#varA
    $#varB?
//- end:
//- place template where val = [(a,b),(c,d),(e)]:
$#var?\option+
//- def a place b where c=d:
//- place a where arr = [a,b,c,d]:

```

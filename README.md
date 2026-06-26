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

## arrow variables
- right arrow var dec /*- $#var_name -> -*/ default,
creates a variable in place
- ex:
pub struct /*- $#struct_name -> -*/ Vec2 
- here struct_name is a variable that has Vec2 as a default 
- value, given a value, Vec2 will be replaced by it
- you can do /*- $#struct_name -> -*/ default+suffix to append to the variable

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
- ~ in the path is a shortcut for the default ~/.rplace folder
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

# parent variables
- you can use parent variables inside place
- ex: //- place name where a=$#var
- in this case "a" will be replaced by the value of "var" from the parent

# data source
- rplace allows for file text data and http
- for http you must provide a file whose body is pure text similar to as if it was a file 

# inner macros
- when you call a macro, all inner macros will also execute
- inner def will make so that a macro is only defined after its parent is called, this is essentially runtime definitions
- inner place works as a normal place

# create
- create allows you to create files and folders and place data inside it

- ex: //- create folder/file.txt place name:
- this will create "folder/file.txt" and place the "name" template inside

# variable options
- variable options allow you to modify variables
- they are marked with "\"
- ex: var=val\regex:
- can also be chained
- supported:
- derive:
    - \regex -> allows regex
    - \def -> places def around match
    - \var -> places arrow variable around match

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

```

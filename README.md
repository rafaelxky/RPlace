# Introduction
- rplace is a preprocessing template language that allows you to write templates and place them over any text file
- its made with programming in mind, it uses markers that use comment simbols to be embedable in code files

# markers
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
- you can do $#var_name+suffix, here + is used to append something to the var name

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
*///- endef:

## def place / inheritance
inheritance is possible with def place
//- def a place b where c=d:
//- place a:
here "a" will be placed with c=d, however, you can call
//- place a were c=e:
this will place with c=e so it allows overrides
also usefull to provide defaults

## def overload / when
- def overload is possible using the keyword when
- ex: //- def name when var=val:
- if var = val then this will be called
- ex: //- place name where var=val:
- it will trow if var is undefined
but you can define a default def with no parameters wich will be called instead
ex: //- def name:
if you call //- place name:
it will call the default def
- variables still work as normal
- you can use the var name inside the body
ex:
//- def name when var=val:
    $#var
//- endef:
this will place the value in the variable as normal

- inheritance and overloading can be used at the same time like
//- def a when lang=java place java_class were var=val:
this will declare a template "a" wich will replace the variable "var" with "val" from "java_class" template if lang = java

# def defaults
- def accept defaults trough def where
- ex: 
//- def name where var=val:
    $#var
//- endef:

# file include
- //- include file_path:
- this will get the definitions from this other file
- ~ in the path is a shortcut for the default ~/.rplace folder

# place
- ex: //- place def_name were var_name=var_value, var2=val2:
//- place def_name were body = "
    multiline variable
//- ":
- this is how you call your definitions with value assignements
- it suports single line and multiline double quote variables
- if no value or default is provided the program will trow
- you can define defaults with def place or arrow variables

# data source
- rplace allows for file text data and http
- for http you must provide a file whose body is pure text similar to as if it was a file 

# inner macros
- when you call a macro, all inner macros will also execute
- inner def will make so that a macro is only defined after its parent is called, this is essentially runtime definitions
- inner place works as a normal place

# create
- create allows you to create files and folders and place data inside it
- create folder/file.txt place name:
- this will create folder/file.txt and place the "name" def inside

# memo
//- place:
//- def name:
//- def name where var=val:
//- def name when var = val;
//- def name where varA=valA when varB=valB:
//- endef:
//- place name;
//- place name where varA=valA, varB=valB:
//- def nameA place nameB:
//- def nameA place nameB where var=val:
//- def nameA when varA=valA where varB=valB:
$#varname
/*- $#varname -> -*/ defaultvar
//- create file:
//- create folder/file.txt place name:

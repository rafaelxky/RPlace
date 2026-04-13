rplace is a preprocessing template language that allows you to write templates and place them over any text file
its made with programming in mind, it uses markers that use comment simbols to be embedable in code files

markers:
//-
-//
/*-
-*/
*///-

any marker can be used interchangebly, pick the one that is better for the ocasion

variable declaration $#var_name
variables must be inside templates

define template
/*- def def_name:
    template here $#var
*///- endef:

right arrow var dec /*- $#var_name -> -*/ default
creates a variable in place 
ex:
pub struct /*- $#struct_name -> -*/ Vec2 
here struct_name is a variable that has Vec2 as a default value, given a value, Vec2 will be replaced by it

file include
//- include file_path:
this will get the definitions from this other file
 ~ in the path is a shortcut for the default ~/.rplace folder

place
//- place def_name were var_name=var_value, var2=val2:
//- place def_name were body = "
    multiline variable 
//- ":
this is how you call your definitions with value assignements
it suports single line and multiline double quote variables
if no value or default is provided the program will trow

inheritance is possible with def place
//- def a place b where c=d:
//- place a:
here "a" will be placed with c=d, however, you can call
//- place a were c=e:
this will place with c=e so it allows overrides
also usefull to provide defaults

def overload is also possible using when
//- def name when var=val:
if var = val then this will be called
//- place name where var=val:
if it will trow if var is undefined
but you can define a default def with no parameters 
//- def name:
if you call
//- place name:
it will call the default def
variables still work as normal
you can use the var name inside the body
ex:
//- def name when var=val:
    $#var
//- endef:
this will place the value in the variable as normal

inheritance and overloading can be used at the same time like
//- def a when lang=java place java_class were var=val:
this will declare a template "a" wich will replace the variable "var" with "val" from "java_class" template if lang = java

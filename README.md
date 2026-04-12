markers:
//-
-//
/*-
-*/
*///-

variable declaration $#var_name

right arrow var dec /*- var_name -> -*/ default
creates a variable in place

define template
/*- def def_name:
    template here
*///- endef:

file include
//- include file_path:

place
//- place def_name were var_name=var_value, var2=val2:
//- place def_name were body = "
    multiline variable 
//- ":

Inheritance is possible with def place
//- def a place b where c=d:
//- place a:
here "a" will be placed with c=d, however, you can call
//- place a were c=e:
this will place with c=e so it allows overrides



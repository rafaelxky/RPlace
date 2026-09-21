# rplace - Template Preprocessing Language

Purpose: Embed templates in any text file using comment-style markers

## Markers
Interchangeable: //-  -//  /*-  -*/  *///-

## Variables
Regular: $#var_name, $#var? (optional), $#var+suffix
Arrow (inline default): /*- $#name -> -*/ default
Modifiers: \snakecase \camelcase \pascalcase \screaming \regex \lua::"code" \def \var
Arrays: [(a,b),(c,d)] used in for loops

## Templates
Define:
//- def name [when condition=val] [where var=default]:
    content $#var
//- end:

Place:
//- place name [where var=value, var2=value2]:

Inheritance: //- def a place b where c=d:
Overloading: Use "when" for conditions; provide default def name: as fallback

## Control Flow
Match/Case:
//- match var:
    //- case val:
        content
    //- end:
//- end:

For Loops:
//- for a,b in array:
    $#a $#b
//- end:

## File Operations
Include: //- include path/file.txt:
Create: //- create folder/file.txt place name [where var=val]:
Parse: //- parse ./file.txt:
Derive (reverse template): //- derive path/file.txt where var="pattern"\regex:

## Config
File-level: //- $#output = "out.txt":
Project: rplace.toml (root file)
Settings: config.json (allow_lua, allow_import)

## CLI
rplace run <origin> [target]
rplace new <project_name>
package add <name>
package push

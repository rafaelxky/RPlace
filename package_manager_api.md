http://localhost:3000
# routes
## create package
- /package POST
- creates a new package 
- requires loggin
- receives:
- { "name": String }
- returns:
{
    "id": i32,
    "name": string,
    "created_at": timedate,
    "creator_id": i32
}
## create version
- /package/version POST
- creates a new package version
- requires loggin
- receives:
{
    "package_name": string,
    "version": i32
}
- returns:
{
    "id": i32,
    "version": string,
    "created_at": datetime,
    "package_id": i32
}
## create file 
- /file POST
- uploads file to package version
- requires loggin
- receives:
{
    "registry_id": i32,
    "version_header_id": i32,
    "code": string,
    "path": string
}
- returns:
{
    "path": string,
    "file_hash": string
}
## loggin
- /loggin POST
- takes email and password, returns JWT token
- receives: 
{
    "email": string,
    "password": string
}
- returns:
{
    "token": string
}
## create user
- /user POST
- creates new user
- input:
{
    "name": string,
    "email": string,
    "password": string
} 
- returns:
{
    "id": i32,
    "name": string
}
## get initial package no version
- package/{name} GET
- returns the rplace.toml file and the project data
- since no version in provided it gives you the latest
- returns:
{
    "repo_id": i32,
    "version": string,
    "header_id": i32,
    "file_hash": string,
    "file_path": string,
    "code": string
}

## get initial package version
- package/{name}/{version} GET
- returns the rplace.toml file and other project data 
- returns for the specified version
- returns:
{
    "repo_id": i32,
    "version": strign,
    "header_id": i32,
    "file_hash": string,
    "file_path": string,
    "code": string
}

## get package file
- /package/fetch_file/{version_header_id}/{path} GET 
- gets the code for the specific file of the specific version
- returns: 
{
    "header_id": i32,
    "file_path": string,
    "file_hash": string,
    "code": string
}


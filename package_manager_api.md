http://localhost:3000
# routes
- /packages/{name} GET
- gets the package with the latest version
- /package/{name}/{version} GET
- gets the package with the specific version
- /package/submit POST
- package sent by body
- { name: String, version: String, code: String }
- adds a new package

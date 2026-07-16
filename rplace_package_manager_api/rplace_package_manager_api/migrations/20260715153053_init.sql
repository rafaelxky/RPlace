-- Add migration script here
DROP TABLE IF EXISTS package_registry;
DROP TABLE IF EXISTS package_version_header;
DROP TABLE IF EXISTS links;
DROP TABLE IF EXISTS package_file;
DROP TABLE IF EXISTS users;


-- registers a package, just the name
-- id is package_version's foreign key 
CREATE TABLE IF NOT EXISTS package_registry (
    id INTEGER PRIMARY KEY,
    package_name TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    creator_id INTEGER NOT NULL,
    FOREIGN KEY (creator_id) REFERENCES users(id)
);

-- registers a package version
-- id is link's foreign key
CREATE TABLE IF NOT EXISTS package_version_header (
    id INTEGER PRIMARY KEY,
    version TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    package_id INTEGER NOT NULL,
    FOREIGN KEY (package_id) REFERENCES package_registry(id)
);

-- links a package version and path to a file
-- this allows diferent versions to use the same file if it didn't change
-- must be cheap to store
CREATE TABLE IF NOT EXISTS links (
    package_version_id INTEGER NOT NULL,
    file_hash TEXT NOT NULL,
    file_path TEXT NOT NULL,
    FOREIGN KEY (package_version_id) REFERENCES package_version(id),
    FOREIGN KEY (file_hash) REFERENCES package_file(file_hash)
);

-- contains the file code, path and hash
CREATE TABLE IF NOT EXISTS package_file (
    file_hash TEXT PRIMARY KEY,
    code TEXT NOT NULL
);

-- users for auth
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL
);


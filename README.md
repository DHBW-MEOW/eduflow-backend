# MEOW-backend
## Deployment
The recommended way of starting everything at once is via the [eduflow-deploy](https://github.com/DHBW-MEOW/eduflow-deploy) repository.

If you want to run the backend only, e.g. for testing purposes:

Make sure a data directory exists
(`mkdir data`)

Run `cargo run` to start the service. (adding the `--release` flag will make it run around 10 times faster)

Alternatively docker / podman can be used (for use with docker please rename the Containerfile to Dockerfile):

Build the container:

`podman build -t eduflow-backend .`

Start the container (make sure a data directory exists):

`podman run -p 3000:3000 -e RUST_LOG=INFO -v ./data:/app/data eduflow-backend`

## Usage
The following section has a quick and dirty description on how to communicate with the backend.
See the bruno test files (test/bruno) for further reference.

### Connecting
Will listen on http://0.0.0.0 port 3000 TCP

http://0.0.0.0:3000 will be shortened to "host" in the following sections

### Authentication
#### Registration / login:
Registration is only first time.
Login is every other time.

will return unauthorized on wrong user or passwd

will return conflict if username is taken

register:
GET host/auth/register

login:
GET host/auth/login

body:
```json
{
  "username": "user",
  "password": "pwd"
}
```

Both login and registration will return the following:
```json
{
  "token": "x_abc..."
}
```

This token is needed if you want to retrieve data, pass it as a Bearer token in the authentication header.

The token is valid for two weeks, it will get invalidated automatically.

#### logout:

Logout does not require a body, the token passed in the auth header will be invalidated.

Will return unauthorized if token is invalid.

Will not return any body data.

### Data
There are always the following methods for every object types (which are listet below)
- create
- edit
- get
- delete

Every action needs a authorization header with a valid Bearer token.

If the token is invalid, will return unauthorized.

#### create / edit
url:  POST host/data/(object-name)

The following has to be send to create or edit an object:
```json
{
  "id": int or null,
  ... (more fields)
}
```
Fields are object specific and will be listed below for every object.

All fields need to be not null and filled out (id is an exception).

If id is null, a new object will be created

If the id is not null, the object with its id will be edited, still all fields are to be filled out.

Will return a json object containing the id of the edited / new object:
```json
{
  "id": int
}
```

NOTE: If id is filled out (-> edit request) but invalid, nothing will be edited, however 200 success will be returned with the id as body (as usual). It is the responsibility of the client to verify that the id is vaild.

#### delete
url: DELETE host/data/(object-name)

The following needs to be send to delete an object:
```json
{
  "id": int
}
```

Object with id will be deleted.

Will return a json object containing the id of the deleted object:
```json
{
  "id": int
}
```

NOTE: will return 200 success, even if the ID is not valid. In this case nothing will happen, because the requested deletion is already deleted (or never existed). It is the responsibility of the client to verify that the id is vaild.

#### get data
url: GET host/data/(object-name)

Filters can be applied to only get some data objects. They have to passed as URL query parameters.

Filter fields are object specific and described below.

Filter fields will be checked on equality.

An array of objects (with the corresponding fields, as listed below) will be returned.
```json
[
  {
    "id": 1,
    ... (more fields)
  },
  ...
]
```

Will return an empty array if no objects match the filter fields.

### data objects

note: int is signed 32bit

#### course

Fields:
```json
{
  "id": int,
  "name": string,
}
```

Filter fields:
```json
{
  "id": int or null
}
```

#### topic

Fields:
```json
{
  "id": int,
  "course_id": int,
  "name": string,
  "details": string
}
```

Filter fields:
```json
{
  "id": int or null,
  "course_id": int or null
}
```

#### study_goal

Fields:
```json
{
  "id": int,
  "topic_id": int,
  "deadline": date // "yyyy-mm-dd" format, e.g: "2025-12-1"
}
```

Filter fields:
```json
{
  "id": int or null,
  "topic_id": int or null
}
```

#### exam

Fields:
```json
{
  "id": int,
  "course_id": int,
  "name": string,
  "date": date // "yyyy-mm-dd"
}
```

Filter fields:
```json
{
  "id": int or null,
  "course_id": int or null
}
```

#### todo

Fields:
```json
{
  "id": int,
  "name": string,
  "deadline": date, // "yyyy-mm-dd"
  "details": string,
  "completed": boolean
}
```

Filter fields:
```json
{
  "id": int or null,
  "completed": boolean or null
}
```

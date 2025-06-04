# MEOW-backend

## Usage

### Connecting
Will listen on http://0.0.0.0 port 3000 TCP

http://0.0.0.0:3000 will be shortened to "host" in the following sections

### Authentication
Both login and registration will return the following:
```json
{
  "token": "x_abc..."
}
```

This token is needed if you want to retrieve data, pass it as a Bearer token in the authentication header.

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

#### logout:

does not need body

just pass token in auth header

token will be invalidated

will return unauthorized if token is invalid

no body response

### Data
There are always the following methods for every object types (which are listet below)
- create
- edit
- get
- delete

Every action needs a authorization header with a valid Bearer token

#### create / edit
url:  POST host/data/<object-name>
the following has to be send to create or edit an object:
```json
{
  "id": int or null,
  ... (more fields)
}
```
fields are object specific and will be listet below for every object

all fields need to be not null and filled out

if id is null, a new object will be created

if the is is not null, the object with its id will be edited, still all fields are to be filled out.

#### delete
url: DELETE host/data/<object-name>

The following needs to be send to delete an object:
```json
{
  "id": int
}
```

object with id will be deleted

#### get data
url: GET host/data/<object-name>

The following needs to be send to get one or more objects:
```json
{
  "id": int or null,
  ... (more FILTER fields)
}
```

Filter fields are object specific and described below.

Filter fields can be null or have a value, they are used to filter out objects.

An array of objects (in the same format as create input body, but always with id) will be returned.
```json
[
  {
    "id": 1,
    ... (more fields)
  },
  ...
]
```
### data objects
#### course

Fields:
```json
{
  "id": int (32bit),
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

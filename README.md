# req.nvim

`req.nvim` is a minimal Neovim plugin for executing HTTP requests from buffers.

The plugin has a Lua integration layer and a Rust core. Lua integrates with
Neovim buffers, selections, commands, and local configuration. Rust is designed
to stay independent from Neovim: it receives the selected request as input,
resolves the request context, and executes the HTTP call.

## MVP goals

The initial file format for requests is `.req`.

- [x] Execute the request under the cursor from a `.req` buffer.
- [x] Execute a visually selected request from a `.req` buffer.
- [ ] Use environment groups for values such as `BASE_URL` and default headers.
- [ ] Use inline variables to override environment values for a single request.
- [ ] Combine multiple environment groups when running a request.
- [ ] Prevent execution when selected environment groups override the same variable.
- [ ] Import a `curl` command as a `.req` request.
- [ ] Export a `.req` request as a `curl` command.
- [ ] Import Postman collections.
- [ ] Export Postman collections.

## Local installation with lazy.nvim

During local development, point lazy.nvim to the plugin directory:

```lua
return {
  dir = "/PATH/TO/req.nvim",
  name = "req.nvim",
}
```

## Usage

Run the plugin command in Neovim:

```vim
:ReqRun
```

`:ReqRun` executes the request under the cursor or the visually selected request.

## Request format

Requests are written in `.req` files.

A minimal request contains an HTTP method and a URL:

```http
GET https://example.com
```

A request can have a name, environment groups, inline variables, headers, and a body:

```http
### Create user
@env dev
@env auth
@BASE_URL=https://staging.example.com

POST {{BASE_URL}}/users
Content-Type: application/json
Authorization: Bearer {{TOKEN}}

{
  "name": "John",
  "email": "john@example.com"
}
```

Multiple requests can live in the same file. Each request starts with `###`:

```http
### Healthcheck
@env dev

GET {{BASE_URL}}/health
Accept: application/json

### Get user
@env dev
@env auth
@USER_ID=1

GET {{BASE_URL}}/users/{{USER_ID}}
Accept: application/json
Authorization: Bearer {{TOKEN}}
```

Format rules:

- `### Name` defines the request name.
- `@env name` selects an environment group.
- `@NAME=value` defines an inline variable for the current request.
- `METHOD URL` defines the HTTP request line.
- `Header-Name: value` defines a header.
- The body starts after the first empty line following the request line and headers.
- Variables use `{{NAME}}` syntax.

Variable precedence:

1. Inline variables declared with `@NAME=value`.
2. Variables from selected environment groups declared with `@env`.
3. Default or global variables.

## Runtime context design

The Rust binary should be able to run outside Neovim. The selected `.req`
request is passed to Rust through stdin.

Lua is expected to pass an optional context to Rust. That context contains
environment groups and their values. The request selects which groups to use with
`@env`, and request-specific overrides can be declared with inline variables.

Example context:

```json
{
  "envs": {
    "dev": {
      "BASE_URL": "https://dev.example.com"
    },
    "auth": {
      "TOKEN": "secret-token"
    }
  },
  "defaults": {
    "BASE_URL": "https://api.example.com"
  }
}
```

The expected runtime flow is:

1. Lua sends the selected request to Rust.
2. Lua optionally sends a context with environment groups and default values.
3. Rust parses the request.
4. Rust loads the selected `@env` groups from the context.
5. Rust checks that selected environment groups do not override each other.
6. Rust applies inline variables as request-specific overrides.
7. Rust replaces `{{VARIABLES}}` in the URL, headers, and body.
8. Rust executes the HTTP request.

## Contributing and development

Clone the repository and build the Rust binary before running the plugin locally.

Build the Rust binary in debug mode:

```sh
cargo build
```

Generate a release binary:

```sh
cargo build --release
```

The Lua files are not compiled. Only the Rust binary is compiled by Cargo.

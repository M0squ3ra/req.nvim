# req.nvim

`req.nvim` is a minimal Neovim plugin with a Lua integration layer and a Rust core.

Lua loads the plugin inside Neovim, runs the Rust binary with `vim.system`, and shows the result with `vim.notify`. The Rust binary is the core executable.

## MVP goals

The initial file format for requests is `.req`.

- [x] Execute the request under the cursor from a `.req` buffer.
- [ ] Execute a visually selected request from a `.req` buffer.
- [ ] Show each request and response in a dedicated Neovim buffer.
- [ ] Use environment groups for values such as `BASE_URL` and default headers.
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
:ReqHello
```

`:ReqHello` executes the Rust binary and displays:

```text
hello from rust
```

## Request format

Requests are written in `.req` files.

A minimal request contains an HTTP method and a URL:

```http
GET https://example.com
```

A request can have a name, environment groups, headers, and a body:

```http
### Create user
@env dev
@env auth

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

GET {{BASE_URL}}/users/1
Accept: application/json
Authorization: Bearer {{TOKEN}}
```

Format rules:

- `### Name` defines the request name.
- `@env name` selects an environment group.
- `METHOD URL` defines the HTTP request line.
- `Header-Name: value` defines a header.
- The body starts after the first empty line following the request line and headers.
- Variables use `{{NAME}}` syntax.

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

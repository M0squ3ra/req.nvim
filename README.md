# req.nvim

`req.nvim` is a minimal Neovim plugin for executing HTTP requests from buffers.

The plugin has a Lua integration layer and a Rust core. Lua integrates with
Neovim buffers, selections, commands, and local configuration. Rust is designed
to stay independent from Neovim: it receives the selected request as input,
resolves the request context, and executes the HTTP call.

## Requirements

- Neovim with `vim.system` support (>= 0.10).
- Rust toolchain with Cargo.

## Installation

### lazy.nvim

Install from GitHub:

```lua
return {
  "m0squ3ra/req.nvim",
  build = "cargo build --release",
  cmd = "Req",
}
```

### vim.pack

Neovim 0.12 includes `vim.pack`. Add the plugin and build the Rust binary when
the package is installed or updated:

```lua
vim.api.nvim_create_autocmd("PackChanged", {
  callback = function(event)
    local spec = event.data and event.data.spec

    if not spec or spec.name ~= "req.nvim" then
      return
    end

    if event.data.kind == "install" or event.data.kind == "update" then
      vim.system({ "cargo", "build", "--release" }, { cwd = event.data.path })
    end
  end,
})

vim.pack.add({
  { src = "https://github.com/m0squ3ra/req.nvim", name = "req.nvim" },
})
```

### vim-plug

```vim
Plug 'm0squ3ra/req.nvim', { 'do': 'cargo build --release' }
```

### Native packages

Install the plugin as a native start package and build the Rust binary:

```sh
git clone https://github.com/m0squ3ra/req.nvim \
  ~/.local/share/nvim/site/pack/req/start/req.nvim

cd ~/.local/share/nvim/site/pack/req/start/req.nvim
cargo build --release
```

## Usage

Run the plugin command in Neovim:

```vim
:Req run
```

`:Req run` executes the request under the cursor or the visually selected request.
`:Req curl` exports the resolved request as a multiline `curl` command.

## Request format

Requests are written in `.http`-style files.

A minimal request contains an HTTP method and a URL:

```http
GET https://example.com
```

A request can have a name, environment groups, inline variables, headers, and a body:

```http
### Create user
# @env dev
# @env auth
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
# @env dev

GET {{BASE_URL}}/health
Accept: application/json

### Get user
# @env dev
# @env auth
@USER_ID=1

GET {{BASE_URL}}/users/{{USER_ID}}
Accept: application/json
Authorization: Bearer {{TOKEN}}
```

Format rules:

- `### Name` defines the request name.
- `# @env name` selects one environment group.
- `@NAME=value` defines an inline variable for the current request.
- `# comment` and `// comment` are ignored.
- `METHOD URL` defines the HTTP request line.
- `Header-Name: value` defines a header.
- The body starts after the first empty line following the request line and headers.
- Variables use `{{NAME}}` syntax.

Variable precedence:

1. Inline variables declared with `@NAME=value`.
2. Variables from selected environment groups declared with `# @env`.
3. Default or global variables.

## Environment context

Lua looks for an optional environment context at:

```text
.req/env.json
```

If the file exists, Lua reads it and passes it to Rust with `--context-json`.
Rust can also run outside Neovim by receiving the selected request through stdin
and an optional context with `--context-json`.

The context contains default variables and environment groups. Defaults are
always available. Environment groups are only available when selected with
`# @env`. Request-specific overrides can be declared with inline variables.

Example `.req/env.json`:

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

See `examples/requests.http` and `examples/.req/env.json` for a working example.

The expected runtime flow is:

1. Lua sends the selected request to Rust.
2. Lua optionally sends a context with environment groups and default values.
3. Rust parses the request.
4. Rust loads the selected `# @env` groups from the context.
5. Rust checks that selected environment groups do not override each other.
6. Rust applies inline variables as request-specific overrides.
7. Rust replaces `{{VARIABLES}}` in the URL, headers, and body.
8. Rust executes the HTTP request.

Example Rust invocation:

```sh
req-nvim --context-json '{"envs":{"dev":{"BASE_URL":"https://dev.example.com"}}}'
```

Export the resolved request as a multiline `curl` command instead of executing
it:

```sh
req-nvim --export-curl --context-json '{"envs":{"dev":{"BASE_URL":"https://dev.example.com"}}}'
```

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

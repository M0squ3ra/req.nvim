# req.nvim

`req.nvim` is a minimal Neovim plugin with a Lua integration layer and a Rust core.

Lua loads the plugin inside Neovim, runs the Rust binary with `vim.system`, and shows the result with `vim.notify`. The Rust binary is the core executable.

## MVP goals

The initial file format for requests is `.req`.

- [ ] Execute the request under the cursor from a `.req` buffer.
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

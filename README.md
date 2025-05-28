# Wonfy Tools

Simple collection of tools, mostly for personal usage.\
You can checkout deployed version with UI on https://tools.won.fyi

#### You can install is as a cli

```sh
cargo install wonfy-tools --features cli
```

then use it as 

```sh
wonfy-tools-cli -f ./0.png ./1.png ...
```

can use `--help` to list all possible inputs for the cli

```sh
wonfy-tools-cli --help
```

#### Or add it as a library

```sh
cargo add wonfy-tools
```

#### Or add it as an npm dependency 

```sh
npm install @wonfsy/wonfy-tools
```

Then initiating the wasm before using any of the functionality.

```ts
  import initWasm, { stitch } from "@wonfsy/wonfy-tools";

  function main() {
    initWasm().then(() => stitch(/* params */))
  }
```

If you're using a bundler, you're gonna have to tell the bundler to include the wasm file and then get a link to it.

##### Example for vite
```ts
  import initWasm, { stitch } from "@wonfsy/wonfy-tools";
  import wasmUrl from "@wonfsy/wonfy-tools/wonfy_tools_bg.wasm?url"

  function main() {
    initWasm({ module_or_path: wasmUrl }).then(() => stitch(/* params */))
  }
```
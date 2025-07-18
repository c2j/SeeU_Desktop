# web-view examples

## minimal
Just displays the wikipedia homepage.

## pageload
Loads a custom url-encoded html page (hello world).

## timer
Uses two-way communication with the web app to render the state of a timer and reset the timer on the click of a button. Shows basic usage of `userdata` and shared state between threads.

## todo
Uses picodom.js to render a basic Todo App. Demonstrates how to embed the frontend into the Rust executable and how to use `userdata` to store app state.

## todo-purescript
This is a port of the todo example to PureScript.
To be able to build this, first install purescript and bundling tools:
```
$ npm install -g purescript pulp psc-package parcel-bundler inline-assets
```
Next, install the dependencies:
```
$ psc-package update
```
Now build the frontend and bundle it into `dist/bundle.html`:
```
$ npm run prod
```
Finally use cargo to build the rust executable, which includes `bundle.html` using `include_str!()`.

## elm-counter

(This assumes you're using Elm 0.19.0)

```
$ npm install -g elm
$ cd elm-counter
$ elm make --optimize src/Main.elm
$ cargo run --example elm-counter
```

## actix

Uses [rust-embed](https://github.com/pyros2097/rust-embed) and [actix-web](https://github.com/actix/actix-web) to embed files directly in binary and serve them to web-view.

Unfortunately if you run this with the EdgeHTML backend (`edge` feature) it won't work by default due to webview sandbox restrictions.

In order for this to run on EdgeHTML, you need to run `CheckNetIsolation.exe LoopbackExempt -a -n="Microsoft.Win32WebViewHost_cw5n1h2txyewy"` from your administrator command prompt only once and everything works.

You can make this step for example as a part of your apps installer.

## todo-yew

Based of the code of the actix example (see above) this bundles/serves the yew [todo example](https://github.com/yewstack/yew/tree/master/examples/todomvc) app. That makes it the most `rust`y example and still only has a ~4mb binary size (90% of which is actix actually, see this example repo using hyper to reduce it to 2mb: https://github.com/Extrawurst/rust-webview-todomvc-yew).

Find the build instructions for the todomvc wasm source in `example/todo-yew/Makefile`.

## todo-elm

(This assumes you're using Elm 0.19.0).  
This example is functionally equivalent to `todo` and `todo-purescript` examples, but implemented in Elm.  
It showcases how to communicate from Elm to Rust and back through Elm's ports.  
You can run this example as is with `cargo run --example todo-elm`.  

If you want to edit the example's sources, you will first need to install Elm as described [here](https://guide.elm-lang.org/install/elm.html).  
Then run:  
```
elm make --optimize --output=elm.js src/Main.elm
cargo run --example todo-elm
```
The `--output=elm.js` parameter is very important, otherwise `elm make` would output `index.html`.
We include `elm.js` and js glue code (for Elm's ports) in `todo-elm.rs`, so we cannot use `index.html`.

## egui_embedded_webview

Demonstrates safe integration between egui 0.28.1 and web-view using system browser commands. Features a native egui interface with URL input, quick navigation buttons, and safe WebView management. Perfect for learning the fundamentals of hybrid GUI applications without threading issues.

```bash
cargo run --example egui_embedded_webview
```

## egui_webview_embedded_safe

Advanced embedded WebView example that attempts true WebView embedding using single-threaded operations. Falls back to system browser if embedding fails. Includes WebView controls, JavaScript evaluation, and manual lifecycle management.

```bash
cargo run --example egui_webview_embedded_safe
```

## egui_webview_advanced

Advanced example showcasing multiple WebView window management, menu bars, status monitoring, and advanced controls. Uses system browser commands for safety. Includes features like window lifecycle management, preset quick-create buttons, and real-time status updates.

```bash
cargo run --example egui_webview_advanced
```

## egui_webview_safe

Safe integration example that uses system browser commands instead of embedded WebView to avoid threading issues. Perfect for production use where stability is more important than tight integration.

```bash
cargo run --example egui_webview_safe
```

## egui_webview_simple

Simple embedded WebView example using the proven multi_window.rs pattern. Single WebView instance with safe step() operations and JavaScript evaluation support.

```bash
cargo run --example egui_webview_simple
```

## egui_webview_final

**🎯 RECOMMENDED FOR TRUE EMBEDDING**: Production-ready embedded WebView example with multiple WebView support, advanced controls, and JavaScript evaluation. Uses the exact same safe pattern as multi_window.rs.

```bash
cargo run --example egui_webview_final
```

## egui_webview_embedded_native

Demonstrates WebView embedding in egui using native window positioning. Creates a WebView window and shows how it could be positioned over an egui area.

```bash
cargo run --example egui_webview_embedded_native
```

## egui_webview_true_embed

Demonstrates the approach for true WebView embedding in egui using platform-specific window parenting. Shows the technical details and challenges.

```bash
cargo run --example egui_webview_true_embed
```

## egui_webview_html_embed

Demonstrates HTML content embedding in egui by fetching web content and rendering it directly within egui UI components.

```bash
cargo run --example egui_webview_html_embed
```

## egui_webview_truly_embedded

**🎯 DEEP REFACTOR RESULT**: Demonstrates truly embedded WebView using the deeply refactored web-view crate. This is the result of extensive modifications to the web-view crate to support real parent window embedding.

⚠️ **Note**: Currently has segmentation fault issues. Use `egui_webview_safe_embedded` instead.

```bash
cargo run --example egui_webview_truly_embedded
```

## egui_webview_safe_embedded

**🛡️ RECOMMENDED SAFE VERSION**: Safe implementation of embedded WebView that avoids segmentation faults while providing multiple embedding modes and comprehensive error handling.

```bash
cargo run --example egui_webview_safe_embedded
```

## egui_webview_iframe_embed

**🎯 TRUE EMBEDDING SOLUTION**: Creates a WebView with HTML content containing an iframe that loads the target URL. This provides true web content display within the WebView window, solving the "fake embedding" problem.

```bash
cargo run --example egui_webview_iframe_embed
```

## egui_webview_native_embed

**🔬 EXPERIMENTAL NATIVE EMBEDDING**: Attempts to get native window handles and embed WebView as a child window. Uses platform-specific code for true window hierarchy embedding.

⚠️ **Warning**: May cause foreign exceptions. Use safe version instead.

```bash
cargo run --example egui_webview_native_embed
```

## egui_webview_safe_native_embed

**🛡️ SAFE NATIVE EMBEDDING**: Safe version of native embedding that avoids foreign exceptions while demonstrating positioning concepts and providing frameless WebView with simulated embedding.

```bash
cargo run --example egui_webview_safe_native_embed
```

## egui_webview_positioning_demo

**📐 POSITIONING DEMO**: Completely safe demonstration of WebView positioning concepts without any risky operations. Shows position calculations, window movement simulation, and debug information.

```bash
cargo run --example egui_webview_positioning_demo
```

## egui_webview_ultra_safe

**🛡️ ULTRA SAFE SIMULATION**: The safest possible WebView demonstration that simulates all WebView operations without creating any actual WebView objects. Zero segfault risk, perfect for learning and development.

```bash
cargo run --example egui_webview_ultra_safe
```

## egui_webview_aligned_embed

**🎯 PRECISION ALIGNMENT**: Advanced WebView positioning with pixel-perfect alignment. Features scale factor detection, manual fine-tuning (1px precision), visual alignment indicators, and comprehensive position debugging.

```bash
cargo run --example egui_webview_aligned_embed
```

## egui_webview_manual_align

**🎮 MANUAL ALIGNMENT**: User-controlled WebView positioning solution. Provides direct position/size controls, multi-level adjustment (10px/1px precision), target area matching, and strong visual indicators. Perfect for solving alignment issues through manual control.

```bash
cargo run --example egui_webview_manual_align
```

## egui_webview_iframe_enhanced

**🔄 ENHANCED IFRAME**: Specifically addresses iframe reload issues with reliable force reload functionality, enhanced loading states, reload counter, better error handling, and URL change detection. Solves the "reload后web内容仍然为空" problem.

```bash
cargo run --example egui_webview_iframe_enhanced
```

## egui_webview_real_embed

**📋 EMBEDDING RESEARCH**: Research implementation for understanding the challenges and approaches to real WebView embedding within egui windows.

```bash
cargo run --example egui_webview_real_embed
```

For detailed documentation on the egui integration examples, see `EGUI_INTEGRATION_EXAMPLES.md`, `EGUI_WEBVIEW_EMBEDDING.md`, `DEEP_WEBVIEW_REFACTOR.md`, `SEGFAULT_ANALYSIS_AND_SOLUTION.md`, `TRUE_EMBEDDING_SOLUTIONS.md`, `NATIVE_EMBED_OPTIMIZATION.md`, `FOREIGN_EXCEPTION_ANALYSIS.md`, `SEGFAULT_RESOLUTION.md`, and `FINAL_SEGFAULT_SOLUTION.md`.

---

Note: For some reason (at least on Windows), if I try to `cargo run` the examples directly, they don't show the window, but it works with `cargo build --example <name> && target\debug\examples\<name>`

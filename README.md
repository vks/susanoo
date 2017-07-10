# `susanoo` [![Build Status](https://travis-ci.org/ubnt-intrepid/susanoo.svg?branch=master)](https://travis-ci.org/ubnt-intrepid/susanoo) [![Join the chat at https://gitter.im/ubnt-intrepid/susanoo](https://badges.gitter.im/ubnt-intrepid/susanoo.svg)](https://gitter.im/ubnt-intrepid/susanoo?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

__WARNING__:  
The development of this project is currently under development.

## Overview
`susanoo` is a micro Web framework for Rust, focused on asynchronous handling and easy of use.

The design of this project is highly inspired by other Web frameworks, e.g. Iron, Nickel.


## Example

```rust
extern crate susanoo;

use susanoo::{Susanoo, Router, Context, AsyncResult, Chain};
use susanoo::contrib::hyper::{Get, StatusCode};

fn hello(ctx: mut Context) -> AsyncResult {
    ctx.res.set_status(StatusCode::Ok);
    ctx.res.set_body("<html><head></head><body><h1>Hello</h1></body></html>");
    ctx.finish()
}

fn main() {
    let router = Router::default()
        .with_route(Get, "/", hello);
    let server = Susanoo::new(router).into_server("0.0.0.0:4000").unwrap();

    server.run().unwrap();
}
```

## Documentation
* [master](https://ubnt-intrepid/susanoo/susanoo/index.html)

## License
`susanoo` is primarily distributed under the terms of both the MIT license and
the Apache License (Version 2.0), with portions covered by various BSD-like licenses.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

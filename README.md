[![coverage](https://remcokranenburg.github.io/boxrs/coverage/badges/for_the_badge.svg)](https://remcokranenburg.github.io/boxrs/coverage/)

Welcome to boxrs, my attempt to create a Rust-only GUI library, in the style of a browser engine.

The idea is this:

 - HTML for markup
 - CSS for style
 - No JS engine: event handling done in Rust

The experiment: could we render enough of CSS to be able to load in GTK's stylesheets and have a
very basic GTK-facsimile in pure Rust?

Lots of inspiration is drawn from Matt Brubeck's series on creating a browser from scratch. You can
find the first article here: [Let's Build a Browser Engine!](https://limpet.net/mbrubeck/2014/08/08/toy-layout-engine-1.html)


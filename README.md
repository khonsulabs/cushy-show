# cushy-show

This is a proof-of-concept presentation tool written using [Cushy][cushy]. It is
inspired by [Reveal.js][reveal], which is what [@Ecton][ecton] would have
normally reached for to present a the [January 2024 Rust Gamedev
Meetup][meetup]. To see the repository at the time of the presentation, check
out the [`initial-demo`][initial-demo] tag.

**This project is not an active focus**, but it will likely be used and slowly
improved upon each time [@Ecton][ecton] needs a slide deck.

## Ideas for this Project's Future

* Hot-reloading slide DSL: A custom DSL for this presentation system could be an
  even more consise way of putting slides together, but also the ability to edit
  slides while viewing them would make editing slides easier. If we can allow
  the DSL to invoke Rust-defined helper functions, it could still be used to
  create

* Visual novel engine: By allowing slide elements to change the current slide,
  it would enable each slide to jump to any other slide, allowing for branching
  narrative paths to be created a-la the "choose your own adventure" books
  [@Ecton][ecton] remembers from his childhood.

[cushy]: https://github.com/khonsulabs/cushy
[ecton]: https://github.com/ecton
[reveal]: https://revealjs.com/
[meetup]: https://youtu.be/I_7AgjiE9RA?t=618
[initial-demo]: https://github.com/khonsulabs/cushy-show/tree/initial-demo

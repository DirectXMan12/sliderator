# Sliderator

## Yet another markdown slide tool [^1]

[^1]: press space to advance here

# How do I use this thing? {master=wide}

<span style="font-size: 200%;">🙋🏼‍♀️</span> [^3]

Run either of these (the first watches for changes to the base
presentation):

```shell-example
~/sliderator $ ./compiler/watch.sh README.md sample-template/template.html
~/sliderator $ ./compiler/run.sh README.md sample-template/template.html > README.md.out.html
```

Move with left and right arrow keys, or with space.

[^3]: Notice that emoji fonts are enabled

# What do I need installed?

* **Rust**
  ([rustup](https://github.com/rust-lang/rustup.rs/#other-installation-methods)
  is a quick way to get this)

* **pandoc** (`apt-get install pandoc`)

* inotify-wait (for `watch.sh` -- `apt-get install inotify-tools`)

# Some sample stuff

Try refreshing -- it'll remember your place w/o crowding your history!

This is a <dfn>definition</dfn> [^2]

This is a list:

* an item
* another item

[^2]: it looks nice, and is semantically valid

# Another Slide With a Wide Format {master=wide}

```shell-example
~ $ do --some $STUFF
~/foo $ echo "hi"
hi
```

# A slide with a flat list

* a flat list item
* another flat item

# A slide with a split view for code {master=split}

This is a description

::: figure

::: figcaption

This figure will appear to the side of the slide

:::

```go 
type SomeStuff struct {

}
```

:::


# A slide with a split view for lots of code code {master=split}

Notice the font is smaller.

::: figure

::: figcaption

This figure will appear to the side of the slide

:::

```{.go .small}
type SomeStuff struct {

}
```

:::

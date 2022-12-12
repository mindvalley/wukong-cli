<p align="center">
  <h1 align="center">
    Wukong CLI
  </h1>
</p>

<div align="center">
  <img alt="Wukong CLI" src="./.github/img/wukong_logo.png" width="200" height="200">
</div>

<p align="center">A Swiss-army Knife CLI For Mindvalley Developers</p>

<div align="center">
    <a href="https://github.com/mindvalley/wukong-cli/actions/workflows/ci.yml">
      <img alt="GitHub Actions status" src="https://github.com/mindvalley/wukong-cli/actions/workflows/ci.yml/badge.svg">
    </a>
    <a href="https://github.com/mindvalley/wukong-cli/actions/workflows/release.yml">
      <img alt="GitHub Actions status" src="https://github.com/mindvalley/wukong-cli/actions/workflows/release.yml/badge.svg">
    </a>
</div>

<div align="center">
  <img alt="Wukong CLI" src="./.github/img/wukong_help.png">
</div>

<p align="center">This is a <strong>Work In Progress 🚧</strong>.</p>

## The Idea

The Wukong CLI is a set of tools to manages Mindvalley DevOps resources. Its goal is to provide a one-stop shop for developers to interact with the Mindvalley DevOps ecosystem. By centralizing different tasks on different tools into a single CLI, It will solve these problems when it comes to adopting DevOps practices:

- Knowledge Gaps: Learning to use a new tool can be painful. Each tool has a unique UI & UX, as well as different workflows.
- Getting Lost in the ecosystem: It’s completely normal for a company to have 20+ different DevOps tools, so knowing which tool to use and remember where to access it can be a problem. After all developers are having enough trouble dealing with their day to day tasks.

> **Note**
> You can read more from [here](https://mindvalley.atlassian.net/wiki/spaces/PXP/pages/450396161/PD2+-+A+Swiss-army+Knife+CLI+For+Mindvalley+Developers)

## KNOWN ISSUE  

* Currently the Jenkins sometimes does not return the list of changes during build, so the CLI is not able to determine the CHANGELOG properly. We are working on a solution in the future release.  

## IMPORTANT !!

The latest release of Wukong CLI contains several breaking changes. Users must upgrade to the latest version, which is at least `0.0.4-beta1` in order to continue using the Wukong CLI. If you are using Wukong version < `0.0.4-beta1`, please follow the below instructions:

```
# Check the current version of Wukong CLI.  
$ wukong --version

# If your version is < 0.0.4-beta1.  
$ brew update  
$ brew upgrade wukong  

# Confirm that you're on version >= 0.0.4-beta1.  
$ wukong --version
wukong 0.0.4-beta1

# Delete the current config.  
$ rm ~/.config/wukong/config.toml

# Re-init the Wukong CLI.  
$ wukong init
```

## Installation

```bash
brew tap mindvalley/wukong
brew update
brew install wukong
```

## Get Started for Development

Make sure you have [Rust](https://www.rust-lang.org/) installed. The recommended way is to install [Rustup](https://www.rust-lang.org/learn/get-started), the Rust installer and version management tool, using:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then you should be able to check your rust version using:

```bash
rustc --version
# output
# rustc 1.63.0 (4b91a6ea7 2022-08-08)
```

> **Note** > `rustc` is the Rust compiler

Since this CLI is a binary program, you can start the program using:

```bash
# compile and run the cli program
cargo run

# compile and run the cli program with help flag
cargo run -- --help
```

To build the cli program, use:

```bash
# using --release flag will trigger the release build, optimized and no debug info
cargo build --release

# run the cli
./target/release/wukong --help
```

> **Note** > `cargo` is the Rust build tool and package manager

## Recommendation

Use [rust-analyzer](https://rust-analyzer.github.io/), a new implementation of the Language Server Protocol (LSP) for Rust.
It is now [officially a part of the wider Rust organization](https://blog.rust-lang.org/2022/02/21/rust-analyzer-joins-rust-org.html).

## Safety

This program uses `#![forbid(unsafe_code)]` to ensure everything is implemented in **100% Safe Rust**.

## License

Licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)

<p align="center">
  <h1 align="center">
    Wukong CLI
  </h1>
</p>

<p align="center">A Swiss-army Knife CLI For Mindvalley Developers</p>

<p align="center">This is a <strong>Work In Progress 🚧</strong>.</p>

## The Idea
The Wukong CLI is a set of tools to manages Mindvalley DevOps resources. Its goal is to provide a one-stop shop for developers to interact with the Mindvalley DevOps ecosystem. By centralizing different tasks on different tools into a single CLI, It will solve these problems when it comes to adopting DevOps practices:  
- Knowledge Gaps: Learning to use a new tool can be painful. Each tool has a unique UI & UX, as well as different workflows. 
- Getting Lost in the ecosystem: It’s completely normal for a company to have 20+ different DevOps tools, so knowing which tool to use and remember where to access it can be a problem. Afterall developers are having enough trouble dealing with their day to day tasks. 

> **Note**
> You can read more from [here](https://mindvalley.atlassian.net/wiki/spaces/PXP/pages/450396161/PD2+-+A+Swiss-army+Knife+CLI+For+Mindvalley+Developers)

## Get Started
Make sure you have [Rust](https://www.rust-lang.org/) installed. The recommanded way is to install [Rustup](https://www.rust-lang.org/learn/get-started), the Rust installer and version management tool, using:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Then you should be able to check your rust version using:
```bash
rustc --version
# output
# rustc 1.61.0 (fe5b13d68 2022-05-18)
```
> **Note**
> `rustc` is the Rust compiler

Since this CLI is a binary program, you can use `cargo run` to start the program
> **Note**
> `cargo` is the Rust build tool and package manager

## Recommendation
Use [rust-analyzer](https://rust-analyzer.github.io/), a new implementation of the Language Server Protocol (LSP) for Rust. 
It is now [officially a part of the wider Rust organization](https://blog.rust-lang.org/2022/02/21/rust-analyzer-joins-rust-org.html).

## Safety
This program uses `#![forbid(unsafe_code)]` to ensure everything is implemented in **100% Safe Rust**.

## License

Licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
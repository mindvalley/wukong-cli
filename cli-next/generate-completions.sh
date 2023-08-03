#!/bin/bash
# we always want to use latest wukong version to generate completions
if ! command -v ./target/release/wukong > /dev/null 2>&1; then
  echo "ERROR: wukong command is not available."
  exit 1
else
  wukong_command=./target/release/wukong
fi

cd "$(dirname "$0")"
mkdir -p completions

gen() {
  outdir="completions/$1"
  mkdir -p ./completions/$1
  # For non homebrew use case
  # if [ $1 = 'zsh' ]; then
    # replace the last line so the zsh autocompletion is source-able
    # https://github.com/starship/starship/issues/2806
    # $wukong_command completions $1 | sed '$s/_wukong "$@"/compdef _wukong wukong/' > ./completions/$1/$2
  # else
  $wukong_command completion $1 > ./completions/$1/$2
  # fi
  echo "$1 completions generated successfully at $( cd "$(dirname "$0")" ; pwd -P )/completions/$1/$2."
}

gen bash wukong.bash
gen zsh _wukong
gen fish wukong.fish

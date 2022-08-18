#!/bin/bash

if ! command -v wukong >/dev/null 2>&1; then
  if ! command -v ./target/release/wukong > /dev/null 2>&1; then
    echo "ERROR: wukong command is not available."
    exit 1
  else
    wukong_command=./target/release/wukong
  fi
else
  wukong_command=wukong
fi

cd "$(dirname "$0")"
mkdir -p completions

gen() {
  outdir="completions/$1"
  mkdir -p ./completions/$1
  $wukong_command completions $1 > ./completions/$1/$2
  echo "$1 completions generated successfully at $( cd "$(dirname "$0")" ; pwd -P )/completions/$1/$2."
}

gen bash wukong.bash
gen zsh _wukong
gen fish wukong.fish

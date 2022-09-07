# Documentation: https://docs.brew.sh/Formula-Cookbook
#                https://rubydoc.brew.sh/Formula

require_relative "lib/github_private_repo_download_strategy"

class Wukong < Formula
  desc "A Swiss-army Knife CLI For Mindvalley Developers"
  homepage "https://github.com/mindvalley/wukong-cli"
  version "0.0.1-dev"

  on_macos do
    on_arm do
      url "https://github.com/mindvalley/wukong-cli/releases/download/#{version}/wukong-v#{version}-macOS-arm.tar.gz", 
        :using => GitHubPrivateRepositoryReleaseDownloadStrategy
      sha256 "5212ca5ca39c402c832c068dde13a0892711f7699922976429d9250df0a21057"
    end

    on_intel do
      url "https://github.com/mindvalley/wukong-cli/releases/download/#{version}/wukong-v#{version}-macOS-x86.tar.gz",
        :using => GitHubPrivateRepositoryReleaseDownloadStrategy
      sha256 "c2c289fd9a8944bda929987bf30ccdbd08e5943d417f81d6168f46891b145850"
    end
  end

  on_linux do
    url "https://github.com/mindvalley/wukong-cli/releases/download/#{version}/wukong-v#{version}-linux-x86.tar.gz",
        :using => GitHubPrivateRepositoryReleaseDownloadStrategy
    sha256 "cdd712b7143f21c5a64dec2c096826c88c5826c4173e29f6ecf38381791e0160"
  end


  def install
    bin.install "wukong"

    bash_completion.install "completions/bash/wukong.bash"
    zsh_completion.install "completions/zsh/_wukong"
    fish_completion.install "completions/fish/wukong.fish"
  end
end


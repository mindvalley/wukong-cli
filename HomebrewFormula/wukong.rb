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
      sha256 "76c2c8697361aac1811545ac71d7055895cdc611eecef98202ed0a2d14878ef7"
    end

    on_intel do
      url "https://github.com/mindvalley/wukong-cli/releases/download/#{version}/wukong-v#{version}-macOS-x86.tar.gz",
        :using => GitHubPrivateRepositoryReleaseDownloadStrategy
      sha256 "eb4879a1f614b5cea6e7c1cbf6eaeff16a6d7603e0500038ab2d7f47bddda9ab"
    end
  end

  on_linux do
    url "https://github.com/mindvalley/wukong-cli/releases/download/#{version}/wukong-v#{version}-linux-x86.tar.gz",
        :using => GitHubPrivateRepositoryReleaseDownloadStrategy
    sha256 "3a79fb67d26104c817a1f8fb17a30ad37fa5d98d448e1ebf1ee8afa76b0ae067"
  end


  def install
    bin.install "wukong"

    bash_completion.install "completions/bash/wukong.bash"
    zsh_completion.install "completions/zsh/_wukong"
    fish_completion.install "completions/fish/wukong.fish"
  end
end


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
      sha256 "7855372e1a8a11704aeb1cf524d1d82098f2a660c8eae986c1810cd061486c4d"
    end

    on_intel do
      url "https://github.com/mindvalley/wukong-cli/releases/download/#{version}/wukong-v#{version}-macOS-x86.tar.gz",
        :using => GitHubPrivateRepositoryReleaseDownloadStrategy
      sha256 "a07399cabbe79da782fadfa3e81a4c6657871e0af452049b9254a87f3281d53f"
    end
  end

  on_linux do
    url "https://github.com/mindvalley/wukong-cli/releases/download/#{version}/wukong-v#{version}-linux-x86.tar.gz",
        :using => GitHubPrivateRepositoryReleaseDownloadStrategy
    sha256 "76af1fbbd96dcb7fd5eb23527860e02324003d580a109bf4c589e94a3d1ad4b3"
  end


  def install
    bin.install "wukong"

    bash_completion.install "completions/bash/wukong.bash"
    zsh_completion.install "completions/zsh/_wukong"
    fish_completion.install "completions/fish/wukong.fish"
  end
end


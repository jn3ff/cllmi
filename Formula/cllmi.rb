class Cllmi < Formula
  desc "CLI tool that uses Claude to correct failed shell commands"
  homepage "https://github.com/jn3ff/cllmi"
  url "https://github.com/jn3ff/cllmi/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"
  head "https://github.com/jn3ff/cllmi.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  def caveats
    <<~EOS
      cllmi requires a Claude API key. Set it in your environment:
        export CLAUDE_API_KEY="your-api-key"

      Your shell must have HISTFILE set (default for zsh/bash).
    EOS
  end

  test do
    assert_match "Usage:", shell_output("#{bin}/cllmi --help")
  end
end

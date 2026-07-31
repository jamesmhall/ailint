# typed: false
# frozen_string_literal: true

# url and sha256 are updated automatically by the release workflow.
class Ailint < Formula
  desc "Linter for AI agent guidance files (CLAUDE.md, AGENTS.md, Copilot/Cursor rules)"
  homepage "https://github.com/jamesmhall/ailint"
  url "https://github.com/jamesmhall/ailint/archive/refs/tags/v1.0.2.tar.gz"
  sha256 "98f411c156213271a7330df22a6c5f5f517e9e7915cdffb0cccd4c3496cad478"
  license any_of: ["MIT", "Apache-2.0"]
  head "https://github.com/jamesmhall/ailint.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/ailint-cli")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/ailint --version")
    (testpath/"AGENTS.md").write("# Test\n\nBe helpful.\n")
    system bin/"ailint", "check", testpath
  end
end

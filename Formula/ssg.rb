# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Homebrew formula for SSG — Static Site Generator
#
# Install:
#   brew install sebastienrousseau/tap/ssg
#
# Or tap-less (from this repo):
#   brew install --formula Formula/ssg.rb

class Ssg < Formula
  desc "Fast, SEO-optimised, WCAG-compliant static site generator"
  homepage "https://github.com/sebastienrousseau/shokunin"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    on_arm do
      url "https://github.com/sebastienrousseau/shokunin/releases/latest/download/ssg-latest-aarch64-apple-darwin.tar.gz"
    end
    on_intel do
      url "https://github.com/sebastienrousseau/shokunin/releases/latest/download/ssg-latest-x86_64-apple-darwin.tar.gz"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/sebastienrousseau/shokunin/releases/latest/download/ssg-latest-aarch64-unknown-linux-gnu.tar.gz"
    end
    on_intel do
      url "https://github.com/sebastienrousseau/shokunin/releases/latest/download/ssg-latest-x86_64-unknown-linux-musl.tar.gz"
    end
  end

  def install
    bin.install "ssg"
  end

  test do
    assert_match "SSG", shell_output("#{bin}/ssg --version")
  end
end

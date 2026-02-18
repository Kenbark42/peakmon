class Peakmon < Formula
  desc "Real-time terminal system monitor for macOS"
  homepage "https://github.com/kenbarker/peakmon"
  url "https://github.com/kenbarker/peakmon/releases/download/v#{version}/peakmon-v#{version}-aarch64-apple-darwin.tar.gz"
  sha256 "PLACEHOLDER"
  license "MIT"

  def install
    bin.install "peakmon"
  end

  test do
    assert_match "peakmon", shell_output("#{bin}/peakmon --version")
  end
end

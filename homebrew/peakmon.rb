class Peakmon < Formula
  desc "Real-time terminal system monitor for macOS"
  homepage "https://github.com/Kenbark42/peakmon"
  version "0.3.0"
  url "https://github.com/Kenbark42/peakmon/releases/download/v#{version}/peakmon-v#{version}-aarch64-apple-darwin.tar.gz"
  sha256 "fb85215bf23eaa0534e273e3b3c837ece92634db4ebdd87d46c686a995e433ee"
  license "MIT"

  def install
    bin.install "peakmon"
  end

  test do
    assert_match "peakmon", shell_output("#{bin}/peakmon --version")
  end
end

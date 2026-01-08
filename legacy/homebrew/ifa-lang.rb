# Homebrew Formula for Ifá-Lang
# The Yoruba Programming Language
# ═══════════════════════════════════════════════════════════════════════════
# To use this formula:
# 1. Create a GitHub repo: homebrew-ifa-lang
# 2. Add this file as Formula/ifa-lang.rb
# 3. Users install with: brew tap AAEO04/ifa-lang && brew install ifa-lang
# ═══════════════════════════════════════════════════════════════════════════

class IfaLang < Formula
  desc "The Yoruba Programming Language - 16 Odù domains, dual syntax"
  homepage "https://github.com/AAEO04/ifa-lang"
  url "https://github.com/AAEO04/ifa-lang/releases/download/v1.0.0/ifa-lang-1.0.0-linux.tar.gz"
  sha256 "PLACEHOLDER_SHA256"  # Update after first release
  license "MIT"
  version "1.0.0"

  depends_on "python@3.11"

  def install
    # Install all files to libexec (private directory)
    libexec.install Dir["*"]
    
    # Make the main script executable
    chmod 0755, libexec/"bin/ifa"
    
    # Create wrapper script in bin that sets up environment
    (bin/"ifa").write <<~EOS
      #!/bin/bash
      export PYTHONPATH="#{libexec}:$PYTHONPATH"
      exec "#{libexec}/bin/ifa" "$@"
    EOS
    chmod 0755, bin/"ifa"
  end

  def post_install
    # Install Python dependencies
    system "python3", "-m", "pip", "install", "--quiet", "-r", "#{libexec}/requirements.txt"
  end

  def caveats
    <<~EOS
      Ifá-Lang has been installed!
      
      Quick Start:
        ifa --help              Show all commands
        ifa run hello.ifa      Run an Ifá program
        ifa repl               Start interactive REPL
      
      VS Code Extension:
        Search "Ifá-Lang" in VS Code Extensions
      
      Documentation:
        #{libexec}/DOCS.md
        #{libexec}/TUTORIAL.md
      
      Àṣẹ! (It is done!)
    EOS
  end

  test do
    # Basic test to verify installation
    assert_match "Ifá-Lang", shell_output("#{bin}/ifa --version")
  end
end

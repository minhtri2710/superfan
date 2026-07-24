cask "superfan" do
  version "1.2.1"
  sha256 "baff77c98980c6bed136fe4290cb9ee51c6032f804299b7b09c102b0423054a3"

  url "https://github.com/minhtri2710/superfan/releases/download/v#{version}/SuperFan_#{version}_universal.dmg"
  name "SuperFan"
  desc "Control and monitor fan speed on macOS"
  homepage "https://github.com/minhtri2710/superfan"

  livecheck do
    url :url
    strategy :github_latest
  end

  app "SuperFan.app"

  zap trash: [
    "~/Library/Application Support/com.superfan.app",
    "~/Library/Caches/com.superfan.app",
    "~/Library/Preferences/com.superfan.app.plist",
  ]
end

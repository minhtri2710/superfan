cask "superfan" do
  version "1.2.3"
  sha256 "21db7de9245a4f88e517655847ac33f4a03764d62f2f2ccb849117c24961f85c"

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

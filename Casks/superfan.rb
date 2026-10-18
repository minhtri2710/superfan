cask "superfan" do
  version "1.2.6"
  sha256 "aa70980e88bda71da4a4a972c636a98c2a9c81bc9e05048155f20b460df39494"

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

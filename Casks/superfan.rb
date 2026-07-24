cask "superfan" do
  version "1.2.3"
  sha256 "b38a435fbbbbf55b1b248a61af6de7fedc8cb1e321ed62dfda4961ccd2ada730"

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

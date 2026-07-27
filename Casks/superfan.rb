cask "superfan" do
  version "1.2.5"
  sha256 "1f2998d87875ecc0bb826db64ea7807268a71953ec826fcffc8455af7fd5c718"

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

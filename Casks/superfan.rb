cask "superfan" do
  version "1.2.1"
  sha256 :no_check # Hoặc mã SHA256 của file SuperFan-macOS-Universal.dmg sau khi release

  url "https://github.com/minhtri2710/superfan/releases/download/v#{version}/SuperFan-macOS-Universal.dmg"
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

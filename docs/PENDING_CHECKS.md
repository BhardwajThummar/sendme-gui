# Pending checks

Things that have **not** been verified yet, and what each one needs. Tick them off
here when done. Everything not listed here has been tested on real hardware or the
simulator (see the changelog).

## Tested on a real iPhone (iPhone 12, iOS 27.0.1)

Done on 2026-10-24, folder transfers checked file by file against the originals:

- Mac to iPhone, scanning the sender's QR code with the in-app camera (4 files,
  200 MB, nested folders): identical.
- iPhone to Mac, folder picked with the in-app picker, ticket copied through the
  clipboard (4 files, 120 MB, nested folders): identical.

Still to check on the iPhone:

- [ ] Background behaviour: start a transfer, lock the phone or switch apps, and
      check what happens. iOS suspends background apps after a short while no matter
      what, so a long transfer can pause when the app isn't on screen. The app keeps
      the screen from auto-locking and asks iOS for extra background time while a
      transfer runs, but this has only been compiled, not run on a device.
- [ ] iCloud Drive and other providers as the iOS save location (only the app's own
      folder has been tried).
- [ ] The direct/relay line on the receive screen (added in the upstream sync).
- [ ] `UIBackgroundModes` (`fetch`, `processing`) in `Info.plist` aren't used by any
      feature. Remove them before an App Store submission, or Apple may question them.

### Building for a real iPhone

1. Sign in to Xcode (Xcode > Settings > Accounts). A free account works; its
   "Personal Team" ID (look in Accounts, or `defaults read com.apple.dt.Xcode
   IDEProvisioningTeamByIdentifier`) is your `APPLE_DEVELOPMENT_TEAM`. It is **not**
   the number in brackets in the certificate's name.
2. Connect the iPhone, unlock it, tap Trust, and turn on Developer Mode.
3. `APPLE_DEVELOPMENT_TEAM=<team id> yarn ios:build`. The first time it fails with
   "Your team has no devices": run `xcodebuild -workspace
   src-tauri/gen/apple/sendme-gui.xcodeproj/project.xcworkspace -scheme
   sendme-gui_iOS -destination id=<device id> -allowProvisioningUpdates
   -allowProvisioningDeviceRegistration build` once so Xcode registers the phone
   (it then fails at the Rust step; that is expected), then build again.
4. `xcrun devicectl device install app --device <id> <unzipped ipa>/Payload/*.app`.
   On the phone: Settings > General > VPN & Device Management > Trust. Free-account
   installs expire after 7 days.

Without the cable (same Wi-Fi, no VPN): newer Xcode versions have no "Connect via
network" checkbox, but it only flips a setting on the phone. With the cable still
plugged in, run `pip install pymobiledevice3` (in a virtualenv) and then
`pymobiledevice3 lockdown set True com.apple.mobile.wireless_lockdown
EnableWifiDebugging --udid <id>`. After unplugging, `xcrun devicectl list devices`
shows the phone as connected over `localNetwork`, and install, launch and
`devicectl device copy` all work (an install takes about 3 seconds).

## Needs Windows and Linux machines

- [ ] Build and run on Windows (the README says it's supported; it has never been
      built there).
- [ ] Build and run on Linux.
- [ ] `yarn build` runs macOS-only installer scripts after the build (`postbuild`);
      check that this doesn't break the build on Windows and Linux.
- [ ] A transfer between each desktop OS and a phone.

## Needs the desktop app driven by hand (or by a tool with screen access)

- [ ] macOS desktop with the latest changes: code field shown or hidden depending on
      the server, 8-character codes, automatic retry, friendly errors.
- [ ] Windows and Linux desktop: the same.
- [ ] Desktop app hidden or minimised during a large transfer (macOS App Nap may
      slow it).

## Before the first release

- [ ] Re-enable CI (`gh workflow enable ci.yml --repo BhardwajThummar/sendme-gui`),
      and add the CI badge back to the README.
- [ ] Add jobs that build the Android and iOS apps, since CI only builds desktop.
- [ ] Pass `BASE_URL` (the code server address) when building the mobile apps, or
      they will offer tickets and QR codes only.
- [ ] Decide the version and tag, build signed artifacts.

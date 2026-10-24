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

- Mac to iPhone over Wi-Fi, 1 GB, with the app restarted mid-transfer: it resumed
  from the 387 MB already stored (only the missing 101 MB went over the network in
  the first 8 seconds); the final file's SHA-256 matched. The receive screen showed
  "Connected directly to the sender".
- **Background behaviour (receiving):** with the app sent to the home screen and the
  screen on, the transfer ran at full speed (4.75 vs 4.80 MB/s). With the screen
  locked it kept going for about 30 seconds (the background time the app asks iOS
  for) and then iOS froze the app, so the transfer stopped. Before the stall fix it
  then stayed stuck at "Receiving..." forever, even after unlocking. Now, once the
  phone is unlocked, the app notices the dead connection within 30 seconds,
  reconnects and continues from where it stopped, without any tap (1 GB, SHA-256
  matched).

- **iPhone as the sender**, 400 MB folder picked in the app, received by the Mac:
  the iPhone kept sending for about 30 seconds after the screen locked, then froze.
  About 90 seconds after the lock the phone was unlocked, the Mac's retry found it
  again and finished the transfer without any action; all files' SHA-256 matched.
  The iPhone's sender screen must stay unlocked for a long share to keep going.

Still to check on the iPhone:

- [ ] The iPhone as the **sender** locked for longer than about two minutes: the
      receiver retries 5 times over roughly that long and then gives up with
      "Couldn't reach the sender". A resumed receive after that should continue from
      what was stored; not tried yet.
- [ ] A long lock (several minutes) while receiving.
- [ ] iCloud Drive and other providers as the iOS save location (only the app's own
      folder has been tried).
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

## Tested on a real Android phone (Galaxy Note10, Android 12), 2026-10-24

- Receive from the Mac over Wi-Fi with a code server configured (the receive screen
  shows the 8 code boxes, so HTTPS to the server works), 600 MB: the screen was
  locked for 50 seconds in the middle and the transfer kept running at full speed;
  every file's SHA-256 matched.
- This run caught a panic: reqwest 0.13's default certificate checker needs JNI
  set-up on Android. The code-server client now uses bundled root certificates.

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

### Driving the iPhone from the Mac

WebDriverAgent (Appium's UI-automation server) gives screenshots and taps on a real
iPhone, which `devicectl` cannot. It is third-party code that gets installed on the
phone, so only do it on a test phone you control. Build it with
`xcodebuild -project WebDriverAgent.xcodeproj -scheme WebDriverAgentRunner
-destination id=<device> DEVELOPMENT_TEAM=<team id>
PRODUCT_BUNDLE_IDENTIFIER=<your own id> -allowProvisioningUpdates
-allowProvisioningDeviceRegistration build-for-testing`, then run it with
`test-without-building` and keep that process running. On the phone, turn on
Settings > Developer > Enable UI Automation. It serves on `http://<phone ip>:8100`
(`/screenshot`, `/session/<id>/wda/tap`, `/wda/lock`, ...). It cannot unlock a phone
that has a passcode. If your shell exports `CC="zig cc"`, run `xcodebuild` with
`env -u CC -u CXX -u LD`.

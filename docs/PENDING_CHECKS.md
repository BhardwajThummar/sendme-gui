# Pending checks

Things that have **not** been verified yet, and what each one needs. Tick them off
here when done. Everything not listed here has been tested on real hardware or the
simulator (see the changelog).

## Needs a real iPhone

First, once: sign Xcode in to the Apple ID that owns the team (Xcode > Settings >
Accounts > +). Without it Xcode cannot create the provisioning profile and the
build fails with "No Accounts" / "No profiles for com.sendme-gui.app". Then run
`APPLE_DEVELOPMENT_TEAM=<team id> yarn ios:build` with the iPhone plugged in,
unlocked, and Developer Mode on.

- [ ] Send and receive a folder between an iPhone and another device.
- [ ] QR scanning on iOS (the simulator has no camera).
- [ ] iCloud Drive and other providers as the iOS save location (only the app's own
      folder and local folders have been tried).
- [ ] Background behaviour: start a transfer, lock the phone or switch apps, and
      check what happens. iOS suspends background apps after a short while no matter
      what, so a long transfer can pause when the app isn't on screen. The app keeps
      the screen from auto-locking and asks iOS for extra background time while a
      transfer runs, but this has only been compiled, not run on a device.
- [ ] `UIBackgroundModes` (`fetch`, `processing`) in `Info.plist` aren't used by any
      feature. Remove them before an App Store submission, or Apple may question them.

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

#!/usr/bin/env bash
# Checks that this machine can build sendme-gui, and says how to fix what's missing.
#   scripts/doctor.sh
# Desktop is required; Android and iOS are optional and reported separately.
cd "$(dirname "$0")/.."

fail=0
check() { # name, command that must succeed, hint
  if eval "$2" >/dev/null 2>&1; then printf '  \033[32mok\033[0m       %s\n' "$1"
  else printf '  \033[31mmissing\033[0m  %s  -> %s\n' "$1" "$3"; return 1; fi
}

echo "Desktop (required)"
check "node 18+"  'node -e "process.exit(+process.versions.node.split(\".\")[0] >= 18 ? 0 : 1)"' "install Node 18+ (https://nodejs.org)" || fail=1
check "yarn"      'yarn --version'  "npm install -g yarn" || fail=1
check "rust"      'cargo --version' "install rustup (https://rustup.rs)" || fail=1
check "C compiler" 'cc --version'   "install Xcode command line tools / build-essential" || fail=1
# A global CC like 'zig cc' breaks crates such as ring.
if [[ -n "${CC:-}" && "$CC" != "cc" ]]; then
  echo "  note     CC=$CC is set; if the Rust build fails in 'ring', run: CC=cc CXX=c++ yarn tauri dev"
fi

echo "Android (optional: yarn android:dev)"
check "Android SDK" '[[ -d "${ANDROID_HOME:-$HOME/Library/Android/sdk}" ]]' "install Android Studio, set ANDROID_HOME"
check "Android NDK" 'ls "${ANDROID_HOME:-$HOME/Library/Android/sdk}"/ndk/* '  "install the NDK in Android Studio's SDK Manager, set NDK_HOME"
check "adb"         'adb version'  "add platform-tools to PATH"
check "rust android targets" 'rustup target list --installed | grep -q aarch64-linux-android' "rustup target add aarch64-linux-android"

if [[ "$(uname)" == "Darwin" ]]; then
  echo "iOS (optional: yarn ios:dev)"
  check "Xcode"   'xcodebuild -version' "install Xcode from the App Store"
  check "simulator runtime matching Xcode" 'xcrun simctl list runtimes | grep -q "$(xcodebuild -version | sed -n "s/Xcode \([0-9]*\.[0-9]*\).*/\1/p")"' "xcodebuild -downloadPlatform iOS"
  check "rust iOS targets" 'rustup target list --installed | grep -q aarch64-apple-ios-sim' "rustup target add aarch64-apple-ios aarch64-apple-ios-sim"
  check "xcodegen" 'xcodegen --version' "brew install xcodegen"
fi

echo
if [[ $fail -eq 0 ]]; then echo "Desktop is ready: yarn install && yarn tauri dev"; else echo "Fix the required items above first."; fi
exit $fail

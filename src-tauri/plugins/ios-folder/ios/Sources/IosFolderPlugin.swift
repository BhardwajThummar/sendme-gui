import Foundation
import SwiftRs
import Tauri
import UIKit
import UniformTypeIdentifiers

struct PickedFolder: Encodable {
  let bookmark: String
  let name: String
}

struct PickResponse: Encodable {
  let folder: PickedFolder?
}

struct PathResponse: Encodable {
  let path: String
}

struct ExistingResponse: Encodable {
  let existing: [String]
}

struct CopyOutArgs: Decodable {
  let bookmark: String
  let dest: String
}

struct ListArgs: Decodable {
  let bookmark: String
  let paths: [String]
}

struct CopyInArgs: Decodable {
  let bookmark: String
  let src: String
  let overwrite: Bool
}

struct FolderError: LocalizedError {
  let message: String
  var errorDescription: String? { message }
}

class IosFolderPlugin: Plugin, UIDocumentPickerDelegate {
  private var pending: Invoke?

  // MARK: Picking

  @objc public func pickFolder(_ invoke: Invoke) throws {
    pending = invoke
    DispatchQueue.main.async {
      let picker = UIDocumentPickerViewController(forOpeningContentTypes: [UTType.folder])
      picker.allowsMultipleSelection = false
      picker.delegate = self
      self.manager.viewController?.present(picker, animated: true, completion: nil)
    }
  }

  public func documentPicker(
    _ controller: UIDocumentPickerViewController, didPickDocumentsAt urls: [URL]
  ) {
    guard let invoke = pending else { return }
    pending = nil
    guard let url = urls.first else {
      invoke.resolve(PickResponse(folder: nil))
      return
    }
    // The picked URL is security-scoped; a bookmark taken while it is open
    // lets us get the same access again later, even after a relaunch.
    let opened = url.startAccessingSecurityScopedResource()
    defer { if opened { url.stopAccessingSecurityScopedResource() } }
    do {
      let data = try url.bookmarkData(
        options: [], includingResourceValuesForKeys: nil, relativeTo: nil)
      invoke.resolve(
        PickResponse(
          folder: PickedFolder(bookmark: data.base64EncodedString(), name: url.lastPathComponent)))
    } catch {
      invoke.reject(error.localizedDescription)
    }
  }

  public func documentPickerWasCancelled(_ controller: UIDocumentPickerViewController) {
    guard let invoke = pending else { return }
    pending = nil
    invoke.resolve(PickResponse(folder: nil))
  }

  // MARK: Access

  /// Resolves a bookmark and keeps the folder open for the duration of `body`.
  private func withFolder<T>(_ bookmark: String, _ body: (URL) throws -> T) throws -> T {
    guard let data = Data(base64Encoded: bookmark) else {
      throw FolderError(message: "The saved folder is not valid")
    }
    var stale = false
    let url = try URL(
      resolvingBookmarkData: data, options: [], relativeTo: nil, bookmarkDataIsStale: &stale)
    let opened = url.startAccessingSecurityScopedResource()
    defer { if opened { url.stopAccessingSecurityScopedResource() } }
    return try body(url)
  }

  private func coordinatedRead(_ url: URL, _ body: (URL) throws -> Void) throws {
    var coordinationError: NSError?
    var inner: Error?
    NSFileCoordinator().coordinate(readingItemAt: url, options: [], error: &coordinationError) {
      readURL in
      do { try body(readURL) } catch { inner = error }
    }
    if let error = coordinationError ?? inner { throw error }
  }

  private func coordinatedWrite(_ url: URL, _ body: (URL) throws -> Void) throws {
    var coordinationError: NSError?
    var inner: Error?
    NSFileCoordinator().coordinate(writingItemAt: url, options: [], error: &coordinationError) {
      writeURL in
      do { try body(writeURL) } catch { inner = error }
    }
    if let error = coordinationError ?? inner { throw error }
  }

  // MARK: Commands

  /// Copies the picked folder to `dest/<folder name>` and returns that path.
  @objc public func copyOut(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(CopyOutArgs.self)
    DispatchQueue.global(qos: .userInitiated).async {
      do {
        let path: String = try self.withFolder(args.bookmark) { folder in
          let fm = FileManager.default
          let destRoot = URL(fileURLWithPath: args.dest, isDirectory: true)
          let target = destRoot.appendingPathComponent(folder.lastPathComponent, isDirectory: true)
          try fm.createDirectory(at: destRoot, withIntermediateDirectories: true)
          try? fm.removeItem(at: target)
          try self.coordinatedRead(folder) { readURL in
            try fm.copyItem(at: readURL, to: target)
          }
          return target.path
        }
        invoke.resolve(PathResponse(path: path))
      } catch {
        invoke.reject(error.localizedDescription)
      }
    }
  }

  /// Which of the relative paths already exist in the picked folder.
  @objc public func existing(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(ListArgs.self)
    DispatchQueue.global(qos: .userInitiated).async {
      do {
        let found: [String] = try self.withFolder(args.bookmark) { folder in
          args.paths.filter {
            FileManager.default.fileExists(atPath: folder.appendingPathComponent($0).path)
          }
        }
        invoke.resolve(ExistingResponse(existing: found))
      } catch {
        invoke.reject(error.localizedDescription)
      }
    }
  }

  /// Copies every file under `src` into the picked folder, keeping the layout.
  /// A taken name is overwritten, or numbered ("a (1).txt") unless `overwrite`.
  @objc public func copyIn(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(CopyInArgs.self)
    DispatchQueue.global(qos: .userInitiated).async {
      do {
        try self.withFolder(args.bookmark) { folder in
          let fm = FileManager.default
          let srcRoot = URL(fileURLWithPath: args.src, isDirectory: true)
          guard
            let walker = fm.enumerator(
              at: srcRoot, includingPropertiesForKeys: [.isRegularFileKey])
          else { return }
          for case let file as URL in walker {
            let isFile = (try? file.resourceValues(forKeys: [.isRegularFileKey]))?.isRegularFile
            if isFile != true { continue }
            let rel = String(file.path.dropFirst(srcRoot.path.count)).trimmingCharacters(
              in: CharacterSet(charactersIn: "/"))
            var dest = folder.appendingPathComponent(rel)
            try fm.createDirectory(
              at: dest.deletingLastPathComponent(), withIntermediateDirectories: true)
            if fm.fileExists(atPath: dest.path) {
              if args.overwrite {
                try fm.removeItem(at: dest)
              } else {
                dest = self.freeName(for: dest)
              }
            }
            try self.coordinatedWrite(dest) { writeURL in
              try fm.copyItem(at: file, to: writeURL)
            }
          }
        }
        invoke.resolve()
      } catch {
        invoke.reject(error.localizedDescription)
      }
    }
  }

  /// "name.ext" -> "name (1).ext", "name (2).ext", ... first one that is free.
  private func freeName(for url: URL) -> URL {
    let dir = url.deletingLastPathComponent()
    let ext = url.pathExtension
    let stem = url.deletingPathExtension().lastPathComponent
    var n = 1
    while true {
      let name = ext.isEmpty ? "\(stem) (\(n))" : "\(stem) (\(n)).\(ext)"
      let candidate = dir.appendingPathComponent(name)
      if !FileManager.default.fileExists(atPath: candidate.path) { return candidate }
      n += 1
    }
  }
}

@_cdecl("init_plugin_ios_folder")
public func initPlugin() -> Plugin {
  return IosFolderPlugin()
}

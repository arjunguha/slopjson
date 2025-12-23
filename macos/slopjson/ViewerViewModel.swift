import AppKit
import Combine
import Foundation
import UniformTypeIdentifiers

@MainActor
final class ViewerViewModel: ObservableObject {
    @Published private(set) var documents: [RustTreeNode] = []
    @Published var selectedNodeID: RustTreeNode.ID?
    @Published var searchText: String = "" {
        didSet { refreshSearchMatches() }
    }
    @Published var isCaseSensitive = false {
        didSet { refreshSearchMatches() }
    }
    @Published private(set) var searchMatches: [RustTreeNode] = []
    @Published private(set) var currentMatchIndex: Int?
    @Published var statusMessage: String? = "Open a JSON, JSONL, YAML, or Parquet file to begin."
    @Published var isLoading = false

    private let service = RustTreeService()
    private let clipboardName = "Clipboard"
    private let supportedTypes: [UTType]

    init() {
        var list: [UTType] = [.json, .plainText]
        if let yaml = UTType(filenameExtension: "yaml") {
            list.append(yaml)
        }
        if let yml = UTType(filenameExtension: "yml") {
            list.append(yml)
        }
        if let parquet = UTType(filenameExtension: "parquet") {
            list.append(parquet)
        }
        supportedTypes = list
    }

    var selectedNode: RustTreeNode? {
        guard let selectedNodeID else { return nil }
        return findNode(withID: selectedNodeID)
    }

    var currentMatchID: RustTreeNode.ID? {
        guard let index = currentMatchIndex,
              index >= 0,
              index < searchMatches.count else {
            return nil
        }
        return searchMatches[index].id
    }

    var hasMatches: Bool {
        !searchMatches.isEmpty
    }

    func presentOpenPanel() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = supportedTypes
        panel.allowsMultipleSelection = true
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.prompt = "Open"
        panel.title = "Open Files"

        if panel.runModal() == .OK {
            loadFiles(from: panel.urls)
        }
    }

    func loadFiles(from urls: [URL]) {
        guard !urls.isEmpty else { return }
        isLoading = true
        statusMessage = "Loading \(urls.count) file(s)..."
        Task {
            do {
                let nodes = try await service.loadFiles(urls)
                documents.append(contentsOf: nodes)
                selectedNodeID = nodes.last?.id ?? selectedNodeID
                statusMessage = nil
            } catch {
                statusMessage = error.localizedDescription
            }
            isLoading = false
            refreshSearchMatches()
        }
    }

    func pasteFromClipboard() {
        guard let content = NSPasteboard.general.string(forType: .string) else {
            statusMessage = "Clipboard does not contain text."
            return
        }

        isLoading = true
        statusMessage = "Parsing clipboard contents..."
        Task {
            do {
                let node = try await service.parseClipboard(content: content, name: clipboardName)
                documents.append(node)
                selectedNodeID = node.id
                statusMessage = nil
            } catch {
                statusMessage = error.localizedDescription
            }
            isLoading = false
            refreshSearchMatches()
        }
    }

    func removeSelectedRoot() {
        guard let selectedNodeID else { return }
        guard let index = documents.firstIndex(where: { $0.id == selectedNodeID }) else {
            return
        }
        documents.remove(at: index)
        self.selectedNodeID = documents.first?.id
        if documents.isEmpty {
            statusMessage = "Open a JSON, JSONL, YAML, or Parquet file to begin."
        }
        refreshSearchMatches()
    }

    func clearAll() {
        documents.removeAll()
        selectedNodeID = nil
        searchMatches = []
        currentMatchIndex = nil
        statusMessage = "Open a JSON, JSONL, YAML, or Parquet file to begin."
    }

    func goToNextMatch() {
        guard hasMatches else { return }
        let nextIndex = ((currentMatchIndex ?? -1) + 1) % searchMatches.count
        selectMatch(at: nextIndex)
    }

    func goToPreviousMatch() {
        guard hasMatches else { return }
        let total = searchMatches.count
        let previousIndex = ((currentMatchIndex ?? total) - 1 + total) % total
        selectMatch(at: previousIndex)
    }

    func copySelectedPath() {
        guard let path = selectedNode?.path else { return }
        writeToClipboard(path)
    }

    func copySelectedValue() {
        guard let value = selectedNode?.displayValue else { return }
        writeToClipboard(value)
    }

    var canRemoveSelectedRoot: Bool {
        guard let selectedNodeID else { return false }
        return documents.contains(where: { $0.id == selectedNodeID })
    }

    private func writeToClipboard(_ value: String) {
        let pasteboard = NSPasteboard.general
        pasteboard.clearContents()
        pasteboard.setString(value, forType: .string)
    }

    private func findNode(withID id: RustTreeNode.ID) -> RustTreeNode? {
        for document in documents {
            if let match = document.findNode(withID: id) {
                return match
            }
        }
        return nil
    }

    private func selectMatch(at index: Int) {
        guard index >= 0, index < searchMatches.count else { return }
        currentMatchIndex = index
        selectedNodeID = searchMatches[index].id
    }

    private func refreshSearchMatches() {
        let trimmed = searchText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            searchMatches.removeAll()
            currentMatchIndex = nil
            return
        }

        var matches: [RustTreeNode] = []
        for document in documents {
            document.collectMatches(query: trimmed, caseSensitive: isCaseSensitive, results: &matches)
        }

        searchMatches = matches
        if matches.isEmpty {
            currentMatchIndex = nil
        } else if let currentID = selectedNodeID,
                  let existingIndex = matches.firstIndex(where: { $0.id == currentID }) {
            selectMatch(at: existingIndex)
        } else {
            selectMatch(at: 0)
        }
    }
}

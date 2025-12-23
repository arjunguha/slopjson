import SwiftUI

struct ContentView: View {
    @ObservedObject var viewModel: ViewerViewModel

    var body: some View {
        NavigationSplitView {
            SidebarView(viewModel: viewModel)
                .frame(minWidth: 320)
        } detail: {
            DetailView(viewModel: viewModel)
                .frame(minWidth: 480)
        }
        .toolbar { toolbarContent }
        .overlay(alignment: .center, content: loadingOverlay)
    }

    @ToolbarContentBuilder
    private var toolbarContent: some ToolbarContent {
        ToolbarItemGroup(placement: .navigation) {
            Button {
                viewModel.presentOpenPanel()
            } label: {
                Label("Open", systemImage: "folder")
            }

            Button {
                viewModel.pasteFromClipboard()
            } label: {
                Label("Paste", systemImage: "doc.on.clipboard")
            }

            Button {
                viewModel.removeSelectedRoot()
            } label: {
                Label("Remove", systemImage: "trash")
            }
            .disabled(!viewModel.canRemoveSelectedRoot)
        }

        ToolbarItemGroup(placement: .automatic) {
            Button {
                viewModel.copySelectedPath()
            } label: {
                Label("Copy Path", systemImage: "arrow.right.doc.on.clipboard")
            }
            .disabled(viewModel.selectedNode == nil)

            Button {
                viewModel.copySelectedValue()
            } label: {
                Label("Copy Value", systemImage: "doc.on.doc")
            }
            .disabled(viewModel.selectedNode == nil)
        }
    }

    @ViewBuilder
    private func loadingOverlay() -> some View {
        if viewModel.isLoading {
            ZStack {
                Color.black.opacity(0.2)
                    .ignoresSafeArea()
                ProgressView("Parsing...")
                    .padding()
                    .background(.regularMaterial)
                    .clipShape(RoundedRectangle(cornerRadius: 12))
            }
        }
    }
}

struct SidebarView: View {
    @ObservedObject var viewModel: ViewerViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            SearchControls(viewModel: viewModel)
            TreeListView(viewModel: viewModel)
        }
        .padding(12)
    }
}

struct SearchControls: View {
    @ObservedObject var viewModel: ViewerViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            TextField("Search keys and values", text: $viewModel.searchText)
                .textFieldStyle(.roundedBorder)

            HStack(spacing: 12) {
                Toggle("Case sensitive", isOn: $viewModel.isCaseSensitive)
                    .toggleStyle(.switch)
                    .controlSize(.small)

                Spacer()

                if viewModel.hasMatches {
                    Text("Match \((viewModel.currentMatchIndex ?? 0) + 1) of \(viewModel.searchMatches.count)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                } else if !viewModel.searchText.isEmpty {
                    Text("No matches")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                HStack(spacing: 4) {
                    Button(action: viewModel.goToPreviousMatch) {
                        Image(systemName: "chevron.up")
                    }
                    .disabled(!viewModel.hasMatches)

                    Button(action: viewModel.goToNextMatch) {
                        Image(systemName: "chevron.down")
                    }
                    .disabled(!viewModel.hasMatches)
                }
                .controlSize(.small)
            }
        }
    }
}

struct TreeListView: View {
    @ObservedObject var viewModel: ViewerViewModel

    var body: some View {
        List(selection: selectionBinding) {
            if viewModel.documents.isEmpty {
                ContentUnavailableView(
                    "No Files",
                    systemImage: "doc",
                    description: Text("Open files or paste JSON to get started.")
                )
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                OutlineGroup(viewModel.documents, children: \.nonEmptyChildren) { node in
                    TreeNodeRow(
                        node: node,
                        isMatched: node.id == viewModel.currentMatchID
                    )
                    .tag(node.id)
                }
            }
        }
        .listStyle(.sidebar)
    }

    private var selectionBinding: Binding<RustTreeNode.ID?> {
        Binding(
            get: { viewModel.selectedNodeID },
            set: { viewModel.selectedNodeID = $0 }
        )
    }
}

struct TreeNodeRow: View {
    let node: RustTreeNode
    let isMatched: Bool

    var body: some View {
        HStack {
            Text(node.name)
                .fontWeight(isMatched ? .semibold : .regular)
            Spacer()
            Text(node.preview)
                .foregroundStyle(.secondary)
                .font(.caption)
        }
        .padding(.vertical, 2)
    }
}

struct DetailView: View {
    @ObservedObject var viewModel: ViewerViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            if let node = viewModel.selectedNode {
                DetailHeader(node: node)
                ValueViewer(value: node.displayValue)
            } else {
                ContentUnavailableView(
                    "Select an Item",
                    systemImage: "cursorarrow.rays",
                    description: Text(viewModel.statusMessage ?? "Select a value to view its JSON path and contents.")
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }

            if let message = viewModel.statusMessage, viewModel.selectedNode != nil {
                Text(message)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(24)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}

struct DetailHeader: View {
    let node: RustTreeNode

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("JSON Path")
                .font(.headline)
            Text(node.path)
                .font(.system(.body, design: .monospaced))
                .padding(8)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(.thinMaterial)
                .clipShape(RoundedRectangle(cornerRadius: 8))
            Divider()
            Text("Value")
                .font(.headline)
        }
    }
}

struct ValueViewer: View {
    let value: String

    var body: some View {
        ScrollView {
            Text(value)
                .font(.system(.body, design: .monospaced))
                .frame(maxWidth: .infinity, alignment: .leading)
                .textSelection(.enabled)
                .padding(.bottom, 8)
        }
    }
}

#Preview {
    ContentView(viewModel: ViewerViewModel())
}

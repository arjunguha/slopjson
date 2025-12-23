import Foundation

struct RustTreeNode: Codable, Identifiable, Hashable {
    let name: String
    let preview: String
    let path: String
    let fullValue: String
    let displayValue: String
    var children: [RustTreeNode]
    var nonEmptyChildren: [RustTreeNode]? {
        children.isEmpty ? nil : children
    }

    var id: String {
        "\(path)::\(name)"
    }

    var isLeaf: Bool {
        children.isEmpty
    }
}

private struct RustBridgeResponse: Decodable {
    enum Status: String, Decodable {
        case ok
        case error
    }

    let status: Status
    let root: RustTreeNode?
    let message: String?
}

enum RustBridgeError: LocalizedError {
    case invalidResponse
    case runtime(String)

    var errorDescription: String? {
        switch self {
        case .invalidResponse:
            return "Rust bridge returned an unexpected payload."
        case .runtime(let message):
            return message
        }
    }
}

final class RustTreeService {
    func loadFiles(_ urls: [URL]) async throws -> [RustTreeNode] {
        var nodes: [RustTreeNode] = []
        for url in urls {
            let node = try await Task.detached(priority: .userInitiated) {
                try self.parseFile(at: url)
            }.value
            nodes.append(node)
        }
        return nodes
    }

    func parseClipboard(content: String, name: String) async throws -> RustTreeNode {
        try await Task.detached(priority: .userInitiated) {
            try self.parseText(content: content, name: name)
        }.value
    }

    func parseFile(at url: URL) throws -> RustTreeNode {
        var path = url.path(percentEncoded: false)
        if path.isEmpty {
            path = url.path
        }
        return try path.withCString { pointer in
            guard let responsePtr = slopjson_parse_file(pointer) else {
                throw RustBridgeError.invalidResponse
            }
            return try decodeAndFree(pointer: responsePtr)
        }
    }

    private func parseText(content: String, name: String) throws -> RustTreeNode {
        try name.withCString { namePtr in
            try content.withCString { contentPtr in
                guard let responsePtr = slopjson_parse_text(contentPtr, namePtr) else {
                    throw RustBridgeError.invalidResponse
                }
                return try decodeAndFree(pointer: responsePtr)
            }
        }
    }

    private func decodeAndFree(pointer: UnsafeMutablePointer<CChar>) throws -> RustTreeNode {
        defer {
            slopjson_string_free(pointer)
        }

        let json = String(cString: pointer)
        guard let data = json.data(using: .utf8) else {
            throw RustBridgeError.invalidResponse
        }

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let response = try decoder.decode(RustBridgeResponse.self, from: data)

        switch response.status {
        case .ok:
            if let root = response.root {
                return root
            }
            throw RustBridgeError.invalidResponse
        case .error:
            throw RustBridgeError.runtime(response.message ?? "Unknown error")
        }
    }
}

extension RustTreeNode {
    func findNode(withID id: String) -> RustTreeNode? {
        if self.id == id {
            return self
        }

        for child in children {
            if let match = child.findNode(withID: id) {
                return match
            }
        }

        return nil
    }

    func containsNode(withID id: String) -> Bool {
        findNode(withID: id) != nil
    }

    func collectMatches(
        query: String,
        caseSensitive: Bool,
        results: inout [RustTreeNode]
    ) {
        if matches(query: query, caseSensitive: caseSensitive) {
            results.append(self)
        }

        for child in children {
            child.collectMatches(query: query, caseSensitive: caseSensitive, results: &results)
        }
    }

    private func matches(query: String, caseSensitive: Bool) -> Bool {
        guard !query.isEmpty else { return false }
        if contains(query: query, in: name, caseSensitive: caseSensitive) {
            return true
        }
        return contains(query: query, in: displayValue, caseSensitive: caseSensitive)
    }

    private func contains(query: String, in value: String, caseSensitive: Bool) -> Bool {
        if caseSensitive {
            return value.range(of: query) != nil
        }
        return value.range(of: query, options: [.caseInsensitive]) != nil
    }
}

import Foundation
import PDFKit
let args = CommandLine.arguments
guard args.count == 2, let doc = PDFDocument(url: URL(fileURLWithPath: args[1])) else { FileHandle.standardError.write("usage: pdftext <file.pdf>\n".data(using: .utf8)!); exit(2) }
FileHandle.standardError.write("pages=\(doc.pageCount)\n".data(using: .utf8)!)
for i in 0..<doc.pageCount { if let p = doc.page(at: i) { print("===== PAGE \(i+1) ====="); print(p.string ?? "") } }

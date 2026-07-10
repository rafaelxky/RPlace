import * as vscode from "vscode";

export function activate(context: vscode.ExtensionContext) {
	vscode.window.showInformationMessage("Activated!");

	const completionProvider = {
		provideCompletionItems(document: vscode.TextDocument, position: vscode.Position) {
			const line = document.lineAt(position).text;
			const before = line.slice(0, position.character);
			const after = line.slice(position.character);

			const start = before.lastIndexOf("//-");

			return [
				new vscode.CompletionItem("place"),
				new vscode.CompletionItem("def"),
			];
		}
	};

	context.subscriptions.push(
		vscode.languages.registerCompletionItemProvider(
			"*",
			completionProvider
		)
	);

	context.subscriptions.push(
		vscode.workspace.onDidChangeTextDocument(event => {
			const editor = vscode.window.activeTextEditor;
			if (!editor) return;

			const position = editor.selection.active;
			const line = editor.document.lineAt(position).text;
			const before = line.slice(0, position.character);

			// autocomplete sujestions
			vscode.commands.executeCommand(
				"editor.action.triggerSuggest"
			);
		})
	);
}

export function deactivate() { }
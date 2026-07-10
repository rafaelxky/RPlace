import * as vscode from "vscode";

export function activate(context: vscode.ExtensionContext) {
	vscode.window.showInformationMessage("Activated!");

	const completionProvider = {
		provideCompletionItems(document: vscode.TextDocument, position: vscode.Position) {
			const line = document.lineAt(position).text;
			const before = line.slice(0, position.character);
			const after = line.slice(position.character);

			const marker = before.lastIndexOf("//-");
			const colon = before.indexOf(":", marker);

			if (marker === -1 || colon !== -1) {
				return;
			}

			const range = new vscode.Range(
				position.line,
				marker + 4,
				position.line,
				position.character
			);
			const place = new vscode.CompletionItem("place");
			place.range = range;
			const def = new vscode.CompletionItem("def");
			def.range = range;
			return [place, def];

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


			const marker = before.lastIndexOf("//-");
			const colon = line.indexOf(":", marker);

			if (marker === -1 || colon !== -1) {
				return;
			}

			// autocomplete sujestions
			vscode.commands.executeCommand(
				"editor.action.triggerSuggest"
			);
		})
	);
}

export function deactivate() { }
import * as vscode from "vscode";

const directiveItems: Array<[string, string, string]> = [
	["def", "Define a reusable template", "def ${1:name}:\n\t$0\n//- end:"],
	["place", "Render a template", "place ${1:name}:"],
	["for", "Iterate over a value or range", "for ${1:item} in ${2:items}:\n\t$#${1:item}\n//- end:"],
	["match", "Select a template block by value", "match ${1:value}:\n\t//- case ${2:case}:\n\t\t$0\n\t//- end:\n//- end:"],
	["include", "Include definitions from another file", "include ${1:path}:"],
	["create", "Create a file", "create ${1:path} place ${2:name}:"],
	["derive", "Create a reverse template", "derive ${1:path} where ${2:name} = ${3:value}:"],
	["parse", "Parse another file", "parse ${1:path}:"],
	["mod", "Load a module", "mod ${1:path}:"],
	["here", "Insert piped text", "here:"],
	["end", "End a template block", "end:"],
];

const directiveMarkers = ["//-", "/*-", "*///-"];

export function provideRplaceCompletions(
	line: string,
	position: number,
	lineNumber = 0,
): vscode.CompletionItem[] | undefined {
	const beforeCursor = line.slice(0, position);
	const marker = findLastDirectiveMarker(beforeCursor);

	if (marker) {
		const directiveText = beforeCursor.slice(marker.end);
		if (directiveText.includes(":")) {
			return undefined;
		}

		return completeDirective(directiveText, position, lineNumber);
	}

	return completeVariable(beforeCursor, position, lineNumber);
}

function findLastDirectiveMarker(
	text: string,
): { start: number; end: number } | undefined {
	let lastMarker: { start: number; end: number } | undefined;

	for (const marker of directiveMarkers) {
		const start = text.lastIndexOf(marker);
		if (start !== -1 && (!lastMarker || start > lastMarker.start)) {
			lastMarker = { start, end: start + marker.length };
		}
	}

	return lastMarker;
}

function completeDirective(
	directiveText: string,
	position: number,
	lineNumber: number,
): vscode.CompletionItem[] {
	const trimmed = directiveText.trimStart();
	const firstSpace = trimmed.search(/\s/);
	const command = firstSpace === -1 ? trimmed : trimmed.slice(0, firstSpace);
	const rest = firstSpace === -1 ? "" : trimmed.slice(firstSpace);
	const currentWordMatch = /[^\s]*$/.exec(directiveText);
	const currentWord = currentWordMatch?.[0] ?? "";
	const wordStart = position - currentWord.length;
	const range = new vscode.Range(lineNumber, wordStart, lineNumber, position);

	if (command.length === 0 || (firstSpace === -1 && !directiveText.endsWith(" "))) {
		return directiveItems.map(([label, detail, snippet]) =>
			createItem(label, detail, snippet, range),
		);
	}

	const trimmedRest = rest.trimStart();
	const restHasTrailingSpace = /\s$/.test(rest);
	const words = trimmedRest.split(/\s+/).filter(Boolean);
	const lastWord = words.at(-1) ?? "";
	const suggestions: Array<[string, string, string]> = [];

	switch (command) {
		case "def":
			if (
				(words.length <= 1 && restHasTrailingSpace) ||
				(words.length === 2 && lastWord.startsWith("w"))
			) {
				suggestions.push(
					["when", "Add a template condition", "when ${1:variable} = ${2:value}"],
					["where", "Set default template values", "where ${1:variable} = ${2:value}"],
				);
			}
			break;
		case "place":
			if (
				(words.length <= 1 && restHasTrailingSpace) ||
				(words.length === 2 && lastWord.startsWith("w"))
			) {
				suggestions.push(["where", "Pass values to the template", "where ${1:name} = ${2:value}"]);
			}
			break;
		case "for":
			if (lastWord !== "in" && !trimmedRest.includes(" in ")) {
				suggestions.push(["in", "Choose the collection or range to iterate", "in "]);
			} else {
				suggestions.push(
					["$#", "Loop value or template variable", "$#${1:name}"],
					["$g#", "Global value", "$g#${1:name}"],
					["range", "Integer range (end exclusive)", "${1:0}..${2:10}"],
				);
			}
			break;
		case "match":
			if (words.length <= 1 && restHasTrailingSpace) {
				suggestions.push(["case", "Add a match arm", "case ${1:value}:\n\t$0\n//- end:"]);
			}
			break;
		default:
			break;
	}

	if (suggestions.length === 0) {
		return [];
	}

	return suggestions.map(([label, detail, snippet]) =>
		createItem(label, detail, snippet, range),
	);
}

function completeVariable(
	beforeCursor: string,
	position: number,
	lineNumber: number,
): vscode.CompletionItem[] | undefined {
	const variableMatch = /(?:\$#|\$g#)[\w]*$/.exec(beforeCursor);
	if (!variableMatch) {
		return undefined;
	}

	const start = position - variableMatch[0].length;
	const range = new vscode.Range(lineNumber, start, lineNumber, position);
	return [
		createItem("$#", "Insert a template variable", "$#${1:name}", range),
		createItem("$g#", "Insert a global variable", "$g#${1:name}", range),
	];
}

function createItem(
	label: string,
	detail: string,
	snippet: string,
	range: vscode.Range,
): vscode.CompletionItem {
	const item = new vscode.CompletionItem(label, vscode.CompletionItemKind.Keyword);
	item.detail = detail;
	item.insertText = new vscode.SnippetString(snippet);
	item.range = range;
	return item;
}

export function activate(context: vscode.ExtensionContext) {
	context.subscriptions.push(
		vscode.languages.registerCompletionItemProvider(
			"*",
			{
				provideCompletionItems(document, position) {
					return provideRplaceCompletions(
						document.lineAt(position).text,
						position.character,
						position.line,
					);
				},
			},
			"-",
			" ",
			"$",
			"#",
		),
	);
}

export function deactivate() {}

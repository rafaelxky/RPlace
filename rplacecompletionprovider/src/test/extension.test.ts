import * as assert from 'assert';
import * as vscode from 'vscode';
import { provideRplaceCompletions } from '../extension';

suite('RPlace Completion Provider', () => {
	test('offers directives and replaces the partial directive', () => {
		const line = 'text //- pla';
		const items = provideRplaceCompletions(line, line.length, 2);
		const place = items?.find(item => item.label === 'place');

		assert.ok(place);
		assert.strictEqual((place.range as vscode.Range).start.line, 2);
		assert.strictEqual((place.range as vscode.Range).start.character, line.length - 3);
	});

	test('offers where after a place name', () => {
		const line = '//- place welcome w';
		const items = provideRplaceCompletions(line, line.length);

		assert.ok(items?.some(item => item.label === 'where'));
	});

	test('offers range and global sources after for-in', () => {
		const line = '//- for i in ';
		const items = provideRplaceCompletions(line, line.length);
		const labels = items?.map(item => item.label);

		assert.ok(labels?.includes('range'));
		assert.ok(labels?.includes('$g#'));
	});

	test('offers variable forms in template text', () => {
		const line = 'Hello $#na';
		const items = provideRplaceCompletions(line, line.length);

		assert.deepStrictEqual(items?.map(item => item.label), ['$#', '$g#']);
	});

	test('does not offer directive completions after the body colon', () => {
		const line = '//- def welcome:';

		assert.strictEqual(provideRplaceCompletions(line, line.length), undefined);
	});
});

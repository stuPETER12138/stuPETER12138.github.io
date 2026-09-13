import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { parsePublications } from './publications.mjs';

// Self-contained runner: no `node --test` flags or child processes, so the same
// command works on every supported Node version and inside restricted sandboxes.
const tests = [];
const test = (name, fn) => tests.push({ name, fn });

test('provided bibliography renders both papers without inventing a DOI', async () => {
  const papers = parsePublications(await readFile(new URL('../papers.bib', import.meta.url), 'utf8'));
  assert.equal(papers.length, 2);
  const [sentimm, seeing] = papers;
  assert.equal(sentimm.key, '10.1007/978-981-95-5679-3_15');
  assert.ok(sentimm.title.startsWith('SentiMM:'));
  assert.equal(sentimm.year, 2026);
  assert.equal(sentimm.typeLabel, '会议论文');
  assert.equal(sentimm.pages, '208–221');
  assert.equal(sentimm.publisher, 'Springer Nature Singapore');
  assert.equal(sentimm.venue, 'Pattern Recognition and Computer Vision');
  assert.equal(sentimm.authorNames.length, 7);
  assert.equal(sentimm.authorNames[0], 'Xilai Xu');
  assert.equal(sentimm.authors, sentimm.authorNames.join(', '));
  assert.ok(sentimm.abstract);
  assert.equal(sentimm.url, '');
  assert.deepEqual(sentimm.links, []);
  assert.equal(seeing.key, 'feng2026seeingviewsbenchmarkingspatial');
  assert.equal(seeing.year, 2026);
  assert.equal(seeing.typeLabel, '预印本');
  assert.equal(seeing.authorNames.length, 19);
  assert.equal(seeing.url, 'https://arxiv.org/abs/2510.19400');
  assert.equal(seeing.pages, '');
  assert.equal(seeing.publisher, '');
  assert.ok(papers.every(p => p.title && Array.isArray(p.links)));
});

test('multiline authors, nested braces, accents, venue, DOI and stable year sorting', () => {
  const papers = parsePublications(String.raw`
@article{old, title={An {Example}}, author={Garc{\'i}a, Ana and
Xu, Xilai}, journal={A Journal}, year={2024}, doi={10.1234/example}}
@misc{new, title="New Paper", year={2026}}
@misc{same, title={Same Year}, year={2026}}
@misc{unknown, title={No Date}}
`);
  assert.deepEqual(papers.map(p => p.key), ['new', 'same', 'old', 'unknown']);
  assert.equal(papers[2].authors, 'Ana García, Xilai Xu');
  assert.equal(papers[2].venue, 'A Journal');
  assert.equal(papers[2].typeLabel, '期刊论文');
  assert.deepEqual(papers[2].authorNames, ['Ana García', 'Xilai Xu']);
  assert.equal(papers[2].url, 'https://doi.org/10.1234/example');
});

test('arXiv fallback preserves supplied year and unsafe URLs are discarded', () => {
  const [paper] = parsePublications('@misc{x,title={A},year={2026},eprint={2510.19400},archivePrefix={arXiv},url={javascript:alert(1)}}');
  assert.equal(paper.url, 'https://arxiv.org/abs/2510.19400');
  assert.equal(paper.year, 2026);
  assert.match(paper.venue, /预印本/);
});

test('DOI-looking keys do not become links; empty optional fields remain empty', () => {
  const [paper] = parsePublications('@misc{10.1234/example,title={A}}');
  assert.equal(paper.url, '');
  assert.equal(paper.authors, '');
  assert.deepEqual(paper.authorNames, []);
  assert.equal(paper.pages, '');
  assert.equal(paper.publisher, '');
  assert.equal(paper.typeLabel, '学术成果');
  assert.deepEqual(parsePublications(''), []);
});

test('published types take precedence over arXiv and literal authors remain intact', () => {
  const [paper] = parsePublications('@article{x,title={A},author={{Research Team}},journal={Journal},publisher={Publisher},pages={12--19},eprint={2510.19400},archiveprefix={arXiv}}');
  assert.equal(paper.typeLabel, '期刊论文');
  assert.equal(paper.venue, 'Journal');
  assert.equal(paper.publisher, 'Publisher');
  assert.equal(paper.pages, '12–19');
  assert.deepEqual(paper.authorNames, ['Research Team']);
  assert.equal(paper.authors, 'Research Team');
  assert.equal(paper.links[0].label, 'arXiv');
});

test('bibliography source follows a consistent format and retains original metadata', async () => {
  const source = await readFile(new URL('../papers.bib', import.meta.url), 'utf8');
  for (const line of source.split('\n').filter(Boolean)) {
    assert.match(line, /^(?:@[a-z]+\{[^,]+,|  [a-z]+ = \{.*\},|\})$/);
  }
  assert.match(source, /  editor = \{Kittler, Josef and Xiong, Hongkai/);
  assert.match(source, /  address = \{Singapore\},/);
  assert.match(source, /  isbn = \{978-981-95-5679-3\},/);
  assert.match(source, /  primaryclass = \{cs.CV\},/);
  assert.match(source, /  pages = \{208--221\},/);
  assert.doesNotMatch(source, /  doi =/);
});

test('invalid entries fail instead of disappearing silently', () => {
  assert.throws(() => parsePublications('@article{broken,title={Unclosed'));
  assert.throws(() => parsePublications('@misc{x,year={2026}}'), /Missing BibTeX title/);
  assert.throws(() => parsePublications('@misc{x,title={A}} @misc{x,title={B}}'), /Duplicate BibTeX key/);
});

let failed = 0;
for (const { name, fn } of tests) {
  try {
    await fn();
    console.log(`✔ ${name}`);
  } catch (error) {
    failed += 1;
    console.error(`✖ ${name}`);
    console.error(error);
  }
}
console.log(`\n${tests.length - failed} passed, ${failed} failed, ${tests.length} total`);
if (failed) process.exitCode = 1;

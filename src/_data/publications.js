import { readFile } from 'node:fs/promises';
import { parsePublications } from '../../scripts/publications.mjs';

export default async function () {
  try {
    return parsePublications(await readFile(new URL('../../papers.bib', import.meta.url), 'utf8'));
  } catch (error) {
    throw new Error(`无法加载 papers.bib，请检查文件及 BibTeX 格式：${error.message}`, { cause: error });
  }
}

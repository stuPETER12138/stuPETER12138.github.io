export default function (eleventyConfig) {
  eleventyConfig.addWatchTarget('./papers.bib');
  eleventyConfig.addPassthroughCopy({ 'papers.bib': 'papers.bib' });
  eleventyConfig.addWatchTarget('./scripts/publications.mjs');
  eleventyConfig.addPassthroughCopy({ "src/assets": "assets" });
  eleventyConfig.addPassthroughCopy("src/robots.txt");
  eleventyConfig.addFilter("year", () => new Date().getFullYear());
  eleventyConfig.addFilter("absoluteUrl", (path, base) => new URL(path, base).href);
  return {
    dir: { input: "src", includes: "_includes", data: "_data", output: "_site" },
    templateFormats: ["njk", "md"],
    markdownTemplateEngine: "njk",
    htmlTemplateEngine: "njk"
  };
}

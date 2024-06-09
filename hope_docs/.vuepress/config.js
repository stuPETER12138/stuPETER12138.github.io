import { viteBundler } from '@vuepress/bundler-vite';
import { hopeTheme } from "vuepress-theme-hope";
import { mdEnhancePlugin } from 'vuepress-plugin-md-enhance';
import { useDarkmode } from "vuepress-theme-hope/client";
const { isDarkmode } = useDarkmode();
console.log(isDarkmode.value); // get darkmode status

export default {
  bundler: viteBundler(), // 确定打包工具
  lang: "zh-CN",
  title: "魔法窝瓜",
  // description: "!",

  Plugins: [
    mdEnhancePlugin({
      katex: true,
      // mathjax: true,
    }),
  ],

  theme: hopeTheme({ 
    // 主题配置
    pure: true,
    darkmode: 'auto',
    logo: '/images/magicsquash.jpg',
    repo: 'stuPETER12138/stuPETER12138.github.io',
    repoLabel: 'GitHub',
    repoDisplay: true,
    navbar: [
      {
        text: '学习记录',
        link: '/studying/smd'
      },
    ],
    sidebar: 'structure',
  }),
};



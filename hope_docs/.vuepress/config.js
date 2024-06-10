import { viteBundler } from '@vuepress/bundler-vite';
import { hopeTheme } from "vuepress-theme-hope";
import { mdEnhancePlugin } from 'vuepress-plugin-md-enhance';
import { getDirname, path } from "vuepress/utils";

const __dirname = getDirname(import.meta.url);

export default {
  bundler: viteBundler(), // 确定打包工具
  lang: "zh-CN",
  title: "魔法窝瓜",
  // description: ""
  theme: hopeTheme({ 
    plugins: {
      mdEnhance: {
        katex: true, 
        sup: true,
        sub: true,
      },
      blog: {
        excerptLength: 0,
      },
    },
    pure: false,
    darkmode: 'toggle',
    logo: '/images/magicsquash.jpg',
    repo: 'stuPETER12138/stuPETER12138.github.io',
    repoLabel: 'GitHub',
    repoDisplay: true,
    navbar: [
      {
        text: '首页',
        link: '/'
      },
      {
        text: '学习记录',
        link: '/studying/smd'
      },
    ],
    sidebar: 'structure',
    // 博客相关
    blog: {
      avatar: "/images/transparent_me.png",
      name: "👋你好，我是魔法窝瓜",
      description: "一个淡淡的大学生",
      medias: {
        GitHub: "https://github.com/stuPETER12138",
      },
    },
  }),
};



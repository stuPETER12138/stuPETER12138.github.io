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
        katex: true, // mathjax: true,
        sup: true, // 启用上角标功能
        sub: true, // 启用下角标功能
        tasklist: true, // 任务列表
        figure: true, // 启用 figure
        imgLazyload: true, // 图片懒加载
        imgMark: true, // 启用图片标记
        imgSize: true,// 启用图片大小
        // footnote: true, 脚注
        // include: true,  支持导入文件
        tabs: true, // 支持选项卡
        alert: true, // GFM警告
        spoiler: true, // 添加剧透文字
        attrs: true, // 添加属性
        mark: true, // 标记
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



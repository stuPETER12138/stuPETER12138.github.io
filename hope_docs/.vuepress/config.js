import { viteBundler } from '@vuepress/bundler-vite';
import { hopeTheme } from "vuepress-theme-hope";
import { mdEnhancePlugin } from 'vuepress-plugin-md-enhance';
import { getDirname, path } from "vuepress/utils";
import { getCreatedTime } from '@vuepress/plugin-git';

const __dirname = getDirname(import.meta.url);

export default {
  bundler: viteBundler(), // 确定打包工具
  lang: 'zh-CN',
  author: '魔法窝瓜',
  theme: hopeTheme({ 
    plugins: {
      mdEnhance: {
        git: {
          CreatedTime: false,
        },
        include: true,
        footnote: true,
        tabs: true,
        align: true,
        mathjax: true,
        sup: true,
        sub: true,
        tasklist: true,
        figure: true,
        imgLazyload: true,
        imgMark: true,
        imgSize: true,
        alert: true,
        spoiler: true,
        attrs: true,
        mark: true,
      },
      blog: {
        excerptLength: 0,
      },
      components: {
        components: [
          'PDF',
          'Share',

        ],
      },
    },
    darkmode: 'toggle',
    logo: '/images/magicsquash.jpg',
    repo: 'stuPETER12138/stuPETER12138.github.io',
    repoLabel: 'GitHub',
    repoDisplay: true,
    displayFooter: true,
    copyright: 'MIT 协议 | 版权所有 © 2024 魔法窝瓜',
    navbar: [
      {
        text: '我学',
        icon: '/icons/line-md--coffee-loop.svg',
        prefix: '/studying/',
        children: [
          {
            text: '我的 markdown 学习',
            link: 'markdown/',
          },
          {
            text: '我的大学',
            link: 'college/',
          },
          {
            text: '广告位招租',
            link: 'weirdthing/',
          },
        ],
      },
      {
        text: '我思',
        icon: '/icons/line-md--speedometer-loop.svg',
        prefix: '/thinking/',
        children: [
          {
            text: '论文精读',
            link: 'papereading/',
          },
        ],
      },
      {
        text: '我做',
        icon: '/icons/line-md--cog-loop.svg',
        prefix: '/moving/',
        children: [
          {
            text: 'Datawhale AI 夏令营',
            link: 'DatawhaleAI/',
          }
        ],
      },
    ],
    sidebar: [
      {
        text: '带我回家',
        icon: '/icons/svg-spinners--blocks-scale.svg',
        link: '/',
      },
      {
        text: '我学',
        prefix: '/studying/',
        children: 'structure',
      },
      {
        text: '我思',
        prefix: '/thinking/',
        children: 'structure',
      },
      {
        text: '我做',
        prefix: '/moving/',
        children: 'structure',
      },
    ],
    blog: {
      avatar: "/images/but_inverted.jpg",
      name: "👋你好，我是魔法窝瓜",
      description: "一只淡淡的窝瓜",
      sidebarDisplay: 'none',
      articlePerPage: '6',
      timeline: '昨天也是努力的一天啊',
      medias: {
        GitHub: "https://github.com/stuPETER12138",
      },
    },
  }),
};



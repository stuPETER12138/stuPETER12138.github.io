import { viteBundler } from '@vuepress/bundler-vite'
import { defaultTheme } from '@vuepress/theme-default'
import { defineUserConfig } from 'vuepress'

export default defineUserConfig({
  bundler: viteBundler(),
 
  lang: 'zh-CN',
  title: '魔法窝瓜的markdown们',
  description: '窝的markdown笔记',
  theme: defaultTheme({
    logo: '/images/magicsquash.jpg',
    repo: 'https://github.com/stuPETER12138/stuPETER12138.github.io',

    // 导航栏设置
    navbar: [
      // NaverItem
      {
        text: '首页',
        link: '/',
      },
      // NaverGroup
      {
        text: '',
        prefix: '/name1/',
        children: ['pdf.md'],
      },
      {
        text: '名字二',
        link: '/name2/',
      },
    ],

    // 侧边栏设置
    sidebar: 'auto',
  }),
})

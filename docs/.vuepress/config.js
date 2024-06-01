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
        text: '学习记录',
        prefix: 'guide/studying/',
        children: ['test.md'],
      },
      {
        text: '学习记录',
        children: ['/studying/smd.md'],
      },
    ],

    // 侧边栏设置
    sidebar: 'auto',
  }),
})

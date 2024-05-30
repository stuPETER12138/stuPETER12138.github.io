import { viteBundler } from '@vuepress/bundler-vite'
import { defaultTheme } from '@vuepress/theme-default'
import { defineUserConfig } from 'vuepress'

export default defineUserConfig({
  bundler: viteBundler(),
 
  lang: 'zh-CN',
  title: '魔法窝瓜的markdown们',
  description: '窝的markdown笔记',
  theme: defaultTheme({
    // 主题配置

    // 导航栏设置
    navbar: [
      // NaverItem
      {
        text: '首页',
        link: '/',
      },
      // NaverGroup
      {
        text: '名字一',
        items: [
          {
            text: 'pdf',
            children: ['/name1/pdf.md'],
          }
        ]
      },
      {
        text: '名字二',
        link: '/name2/',
      },
    ],
    // 侧边栏设置
    sidebar: {
      '/name1/': [       
        {
          text: '名字一',
        },
      ],
      '/name2/': [
        {
          text: '名字二',
        },
      ],
    }
  }),
})

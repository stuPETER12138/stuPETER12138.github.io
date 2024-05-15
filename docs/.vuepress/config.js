import { viteBundler } from '@vuepress/bundler-vite'
import { defaultTheme } from '@vuepress/theme-default'
import { defineUserConfig } from 'vuepress'

export default defineUserConfig({
    bundler: viteBundler(),
    theme: defaultTheme(),

    lang: 'zh-CN',
    title: '欢迎来到我的博客世界',
    description: '这是我的第一个 VuePress 站点',
})

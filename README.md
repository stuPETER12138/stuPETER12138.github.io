![Who I am](https://img.shields.io/badge/%E4%BD%9C%E8%80%85-Magic_Squash-green) ![Static Badge](https://img.shields.io/badge/%E8%AE%B8%E5%8F%AF%E8%AF%81-MIT-blue) 


### 目录结构

```
.
├── docs
│   ├── .vuepress (用于存放全局的配置、组件、静态资源等)
│   │   ├── components (该目录中的 Vue 组件将会被自动注册为全局组件)
│   │   ├── theme (用于存放本地主题)
│   │   │   └── Layout.vue
│   │   ├── public (静态资源目录)
│   │   ├── styles (用于存放样式相关的文件)
│   │   │   ├── index.styl
│   │   │   └── palette.styl (用于重写默认颜色常量，或者设置新的 stylus 颜色常量)
│   │   ├── templates (可选的, 谨慎配置!存储 HTML 模板文件)
│   │   │   ├── dev.html (用于开发环境的 HTML 模板文件)
│   │   │   └── ssr.html (构建时基于 Vue SSR 的 HTML 模板文件)
│   │   ├── config.js (配置文件的入口文件)
│   │   └── client.js (客服端配置文件)
│   │ 
│   ├── README.md
│   ├── guide
│   │   └── README.md
│   ├── name1
│   ├── name2
│   └── config.md (暂无)
│ 
├── package.json
│ 
├── pnpm-lock.yaml
│ 
└── README.md
```


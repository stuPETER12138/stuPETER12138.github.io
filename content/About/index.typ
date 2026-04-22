#import "../index.typ": template, tufted
// 如需生成 RSS feed，必须填写 title、description 和 date 元数据
#show: template.with(
  title: "Iterators vs Generators in Python",
  description: "Understanding the differences between iterators and generators in Python, and when to use each.",
  date: datetime(year: 2024, month: 10, day: 4),
  lang: "en",
)

= 魔法窝瓜 / Magic Squash

#tufted.margin-note({
  image("../imgs/magicsquash.webp")
})
#tufted.margin-note[
  魔法窝瓜这个名字源于我丑萌丑萌的头像
]

哈喽哈喽小伙伴，欢迎来到我的个人网站！🤗
